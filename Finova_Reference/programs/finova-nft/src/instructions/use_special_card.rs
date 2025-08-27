// programs/finova-nft/src/instructions/use_special_card.rs

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Burn};
use crate::state::*;
use crate::errors::*;
use crate::constants::*;
use crate::utils::*;

/// Use a special card NFT to apply its effects
#[derive(Accounts)]
#[instruction(card_type: u8, target_user: Option<Pubkey>)]
pub struct UseSpecialCard<'info> {
    /// User who owns and uses the card
    #[account(mut)]
    pub user: Signer<'info>,

    /// User account state
    #[account(
        mut,
        seeds = [SEED_USER_ACCOUNT, user.key().as_ref()],
        bump = user_account.bump,
    )]
    pub user_account: Account<'info, UserAccount>,

    /// Special card NFT metadata
    #[account(
        mut,
        seeds = [
            SEED_SPECIAL_CARD,
            special_card.mint.as_ref(),
        ],
        bump = special_card.bump,
        constraint = special_card.owner == user.key() @ NftError::NotCardOwner,
        constraint = !special_card.is_used @ NftError::CardAlreadyUsed,
        constraint = special_card.card_type == card_type @ NftError::InvalidCardType,
    )]
    pub special_card: Account<'info, SpecialCard>,

    /// NFT mint account
    #[account(
        mut,
        constraint = nft_mint.key() == special_card.mint @ NftError::InvalidMint,
    )]
    pub nft_mint: Account<'info, Mint>,

    /// User's token account holding the NFT
    #[account(
        mut,
        constraint = user_token_account.mint == nft_mint.key() @ NftError::InvalidTokenAccount,
        constraint = user_token_account.owner == user.key() @ NftError::InvalidTokenAccount,
        constraint = user_token_account.amount >= 1 @ NftError::InsufficientNftBalance,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    /// Target user account (for cards that affect other users)
    #[account(
        mut,
        seeds = [SEED_USER_ACCOUNT, target_user.unwrap_or(user.key()).as_ref()],
        bump,
    )]
    pub target_user_account: Option<Account<'info, UserAccount>>,

    /// Card usage record
    #[account(
        init,
        payer = user,
        space = CardUsage::SIZE,
        seeds = [
            SEED_CARD_USAGE,
            special_card.mint.as_ref(),
            &Clock::get()?.unix_timestamp.to_le_bytes(),
        ],
        bump,
    )]
    pub card_usage: Account<'info, CardUsage>,

    /// Global network state for network-wide effects
    #[account(
        mut,
        seeds = [SEED_NETWORK_STATE],
        bump = network_state.bump,
    )]
    pub network_state: Account<'info, NetworkState>,

    /// Token program
    pub token_program: Program<'info, Token>,
    /// System program
    pub system_program: Program<'info, System>,
    /// Rent sysvar
    pub rent: Sysvar<'info, Rent>,
    /// Clock sysvar
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> UseSpecialCard<'info> {
    /// Validate card usage conditions
    fn validate_card_usage(&self, card_type: u8) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;

        // Check if card has expired
        if let Some(expiry) = self.special_card.expiry_timestamp {
            require!(current_time <= expiry, NftError::CardExpired);
        }

        // Check cooldown period for user
        if let Some(last_card_use) = self.user_account.last_card_use_timestamp {
            let cooldown_period = match self.special_card.rarity {
                CardRarity::Common => CARD_COOLDOWN_COMMON,
                CardRarity::Uncommon => CARD_COOLDOWN_UNCOMMON,
                CardRarity::Rare => CARD_COOLDOWN_RARE,
                CardRarity::Epic => CARD_COOLDOWN_EPIC,
                CardRarity::Legendary => CARD_COOLDOWN_LEGENDARY,
            };
            
            require!(
                current_time >= last_card_use + cooldown_period,
                NftError::CardOnCooldown
            );
        }

        // Validate card type specific conditions
        match card_type {
            CARD_TYPE_MINING_BOOST => {
                require!(
                    self.user_account.mining_state.is_active,
                    NftError::MiningNotActive
                );
            },
            CARD_TYPE_XP_MULTIPLIER => {
                // XP cards can always be used
            },
            CARD_TYPE_REFERRAL_BOOST => {
                require!(
                    self.user_account.referral_state.total_referrals > 0,
                    NftError::NoReferralsToBoost
                );
            },
            CARD_TYPE_NETWORK_AMPLIFIER => {
                require!(
                    self.user_account.referral_state.network_size >= MIN_NETWORK_SIZE_FOR_AMPLIFIER,
                    NftError::NetworkTooSmall
                );
            },
            CARD_TYPE_GUILD_POWER => {
                require!(
                    self.user_account.guild_id.is_some(),
                    NftError::NotInGuild
                );
            },
            _ => return Err(NftError::InvalidCardType.into()),
        }

        Ok(())
    }

    /// Apply card effects based on card type
    fn apply_card_effects(&mut self, card_type: u8) -> Result<CardEffects> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        
        let mut effects = CardEffects::default();

        match card_type {
            CARD_TYPE_MINING_BOOST => {
                effects = self.apply_mining_boost_card(current_time)?;
            },
            CARD_TYPE_XP_MULTIPLIER => {
                effects = self.apply_xp_multiplier_card(current_time)?;
            },
            CARD_TYPE_REFERRAL_BOOST => {
                effects = self.apply_referral_boost_card(current_time)?;
            },
            CARD_TYPE_NETWORK_AMPLIFIER => {
                effects = self.apply_network_amplifier_card(current_time)?;
            },
            CARD_TYPE_GUILD_POWER => {
                effects = self.apply_guild_power_card(current_time)?;
            },
            CARD_TYPE_STREAK_SAVER => {
                effects = self.apply_streak_saver_card(current_time)?;
            },
            CARD_TYPE_LEVEL_RUSH => {
                effects = self.apply_level_rush_card(current_time)?;
            },
            _ => return Err(NftError::InvalidCardType.into()),
        }

        // Apply synergy bonuses if multiple cards are active
        self.apply_synergy_bonuses(&mut effects)?;

        Ok(effects)
    }

    /// Apply mining boost card effects
    fn apply_mining_boost_card(&mut self, current_time: i64) -> Result<CardEffects> {
        let multiplier = match self.special_card.rarity {
            CardRarity::Common => 200,    // +100% (2.0x)
            CardRarity::Uncommon => 250,  // +150% (2.5x)
            CardRarity::Rare => 300,      // +200% (3.0x)
            CardRarity::Epic => 500,      // +400% (5.0x)
            CardRarity::Legendary => 600, // +500% (6.0x)
        };

        let duration = match self.special_card.rarity {
            CardRarity::Common => 86400,      // 24 hours
            CardRarity::Uncommon => 64800,    // 18 hours
            CardRarity::Rare => 43200,        // 12 hours
            CardRarity::Epic => 14400,        // 4 hours
            CardRarity::Legendary => 10800,   // 3 hours
        };

        // Apply mining boost
        self.user_account.mining_state.boost_multiplier = multiplier;
        self.user_account.mining_state.boost_end_timestamp = Some(current_time + duration);
        
        // Update card usage tracking
        self.user_account.cards_used_count += 1;
        self.user_account.last_card_use_timestamp = Some(current_time);

        Ok(CardEffects {
            mining_multiplier: Some(multiplier),
            duration_seconds: duration,
            effect_type: EffectType::MiningBoost,
            ..Default::default()
        })
    }

    /// Apply XP multiplier card effects  
    fn apply_xp_multiplier_card(&mut self, current_time: i64) -> Result<CardEffects> {
        let multiplier = match self.special_card.rarity {
            CardRarity::Common => 200,    // +100% XP
            CardRarity::Uncommon => 250,  // +150% XP
            CardRarity::Rare => 300,      // +200% XP
            CardRarity::Epic => 400,      // +300% XP
            CardRarity::Legendary => 500, // +400% XP
        };

        let duration = match self.special_card.rarity {
            CardRarity::Common => 86400,   // 24 hours
            CardRarity::Uncommon => 129600, // 36 hours
            CardRarity::Rare => 172800,    // 48 hours
            CardRarity::Epic => 259200,    // 72 hours
            CardRarity::Legendary => 604800, // 168 hours (1 week)
        };

        // Apply XP boost
        self.user_account.xp_state.boost_multiplier = multiplier;
        self.user_account.xp_state.boost_end_timestamp = Some(current_time + duration);

        Ok(CardEffects {
            xp_multiplier: Some(multiplier),
            duration_seconds: duration,
            effect_type: EffectType::XpBoost,
            ..Default::default()
        })
    }

    /// Apply referral boost card effects
    fn apply_referral_boost_card(&mut self, current_time: i64) -> Result<CardEffects> {
        let multiplier = match self.special_card.rarity {
            CardRarity::Common => 150,    // +50% referral rewards
            CardRarity::Uncommon => 175,  // +75% referral rewards
            CardRarity::Rare => 200,      // +100% referral rewards
            CardRarity::Epic => 250,      // +150% referral rewards
            CardRarity::Legendary => 300, // +200% referral rewards
        };

        let duration = 604800; // 7 days for all rarities

        // Apply referral boost
        self.user_account.referral_state.boost_multiplier = multiplier;
        self.user_account.referral_state.boost_end_timestamp = Some(current_time + duration);

        Ok(CardEffects {
            referral_multiplier: Some(multiplier),
            duration_seconds: duration,
            effect_type: EffectType::ReferralBoost,
            ..Default::default()
        })
    }

    /// Apply network amplifier card effects
    fn apply_network_amplifier_card(&mut self, current_time: i64) -> Result<CardEffects> {
        let tier_boost = match self.special_card.rarity {
            CardRarity::Common => 1,      // +1 tier
            CardRarity::Uncommon => 1,    // +1 tier  
            CardRarity::Rare => 2,        // +2 tiers
            CardRarity::Epic => 2,        // +2 tiers
            CardRarity::Legendary => 3,   // +3 tiers
        };

        let duration = match self.special_card.rarity {
            CardRarity::Common => 86400,   // 24 hours
            CardRarity::Uncommon => 129600, // 36 hours
            CardRarity::Rare => 172800,    // 48 hours
            CardRarity::Epic => 259200,    // 72 hours
            CardRarity::Legendary => 432000, // 120 hours (5 days)
        };

        // Temporarily boost RP tier
        let current_tier = calculate_rp_tier(self.user_account.referral_state.total_rp);
        let boosted_tier = (current_tier + tier_boost).min(RP_TIER_AMBASSADOR);
        
        self.user_account.referral_state.temporary_tier_boost = Some(boosted_tier);
        self.user_account.referral_state.tier_boost_end_timestamp = Some(current_time + duration);

        Ok(CardEffects {
            rp_tier_boost: Some(tier_boost),
            duration_seconds: duration,
            effect_type: EffectType::NetworkAmplifier,
            ..Default::default()
        })
    }

    /// Apply guild power card effects
    fn apply_guild_power_card(&mut self, current_time: i64) -> Result<CardEffects> {
        let power_boost = match self.special_card.rarity {
            CardRarity::Common => 1000,    // +1000 guild power
            CardRarity::Uncommon => 2500,  // +2500 guild power
            CardRarity::Rare => 5000,      // +5000 guild power
            CardRarity::Epic => 10000,     // +10000 guild power
            CardRarity::Legendary => 25000, // +25000 guild power
        };

        let duration = 259200; // 72 hours for all rarities

        // Apply guild power boost
        self.user_account.guild_power_boost = Some(power_boost);
        self.user_account.guild_power_boost_end_timestamp = Some(current_time + duration);

        Ok(CardEffects {
            guild_power_boost: Some(power_boost),
            duration_seconds: duration,
            effect_type: EffectType::GuildPower,
            ..Default::default()
        })
    }

    /// Apply streak saver card effects
    fn apply_streak_saver_card(&mut self, current_time: i64) -> Result<CardEffects> {
        let protection_days = match self.special_card.rarity {
            CardRarity::Common => 3,      // 3 days protection
            CardRarity::Uncommon => 5,    // 5 days protection
            CardRarity::Rare => 7,        // 7 days protection
            CardRarity::Epic => 10,       // 10 days protection
            CardRarity::Legendary => 14,  // 14 days protection
        };

        let duration = protection_days * 86400; // Convert days to seconds

        // Apply streak protection
        self.user_account.xp_state.streak_protection_end_timestamp = Some(current_time + duration);
        self.user_account.xp_state.streak_protection_uses = protection_days as u32;

        Ok(CardEffects {
            streak_protection_days: Some(protection_days),
            duration_seconds: duration,
            effect_type: EffectType::StreakSaver,
            ..Default::default()
        })
    }

    /// Apply level rush card effects (instant XP boost)
    fn apply_level_rush_card(&mut self, current_time: i64) -> Result<CardEffects> {
        let instant_xp = match self.special_card.rarity {
            CardRarity::Common => 500,     // +500 XP
            CardRarity::Uncommon => 1000,  // +1000 XP
            CardRarity::Rare => 2500,      // +2500 XP
            CardRarity::Epic => 5000,      // +5000 XP
            CardRarity::Legendary => 10000, // +10000 XP
        };

        // Apply instant XP
        self.user_account.xp_state.total_xp = self.user_account.xp_state.total_xp
            .checked_add(instant_xp)
            .ok_or(NftError::ArithmeticOverflow)?;

        // Recalculate level after XP gain
        let new_level = calculate_level_from_xp(self.user_account.xp_state.total_xp);
        if new_level > self.user_account.xp_state.current_level {
            self.user_account.xp_state.current_level = new_level;
            self.user_account.xp_state.last_level_up_timestamp = current_time;
        }

        Ok(CardEffects {
            instant_xp: Some(instant_xp),
            duration_seconds: 0, // Instant effect
            effect_type: EffectType::LevelRush,
            ..Default::default()
        })
    }

    /// Apply synergy bonuses for multiple active cards
    fn apply_synergy_bonuses(&mut self, effects: &mut CardEffects) -> Result<()> {
        let active_card_count = self.count_active_card_effects();
        
        if active_card_count >= 2 {
            let synergy_multiplier = 100 + (active_card_count as u16 * 10); // +10% per additional card
            
            // Apply synergy to all active effects
            if let Some(ref mut mining_mult) = effects.mining_multiplier {
                *mining_mult = (*mining_mult * synergy_multiplier) / 100;
            }
            
            if let Some(ref mut xp_mult) = effects.xp_multiplier {
                *xp_mult = (*xp_mult * synergy_multiplier) / 100;
            }
            
            if let Some(ref mut ref_mult) = effects.referral_multiplier {
                *ref_mult = (*ref_mult * synergy_multiplier) / 100;
            }

            effects.synergy_bonus = Some(synergy_multiplier);
        }

        Ok(())
    }

    /// Count currently active card effects
    fn count_active_card_effects(&self) -> u8 {
        let current_time = Clock::get().unwrap().unix_timestamp;
        let mut count = 0u8;

        // Check mining boost
        if let Some(end_time) = self.user_account.mining_state.boost_end_timestamp {
            if current_time < end_time {
                count += 1;
            }
        }

        // Check XP boost
        if let Some(end_time) = self.user_account.xp_state.boost_end_timestamp {
            if current_time < end_time {
                count += 1;
            }
        }

        // Check referral boost
        if let Some(end_time) = self.user_account.referral_state.boost_end_timestamp {
            if current_time < end_time {
                count += 1;
            }
        }

        // Check tier boost
        if let Some(end_time) = self.user_account.referral_state.tier_boost_end_timestamp {
            if current_time < end_time {
                count += 1;
            }
        }

        // Check guild power boost
        if let Some(end_time) = self.user_account.guild_power_boost_end_timestamp {
            if current_time < end_time {
                count += 1;
            }
        }

        count
    }

    /// Burn single-use cards after usage
    fn burn_single_use_card(&mut self) -> Result<()> {
        // Only burn if it's a single-use card
        if self.special_card.is_single_use {
            let burn_ctx = CpiContext::new(
                self.token_program.to_account_info(),
                Burn {
                    mint: self.nft_mint.to_account_info(),
                    from: self.user_token_account.to_account_info(),
                    authority: self.user.to_account_info(),
                }
            );
            
            token::burn(burn_ctx, 1)?;
            
            // Mark card as used
            self.special_card.is_used = true;
            self.special_card.use_timestamp = Some(Clock::get()?.unix_timestamp);
        }

        Ok(())
    }
}

/// Execute special card usage
pub fn use_special_card(
    ctx: Context<UseSpecialCard>,
    card_type: u8,
    target_user: Option<Pubkey>,
) -> Result<()> {
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    // Validate card usage conditions
    ctx.accounts.validate_card_usage(card_type)?;

    // Apply card effects
    let effects = ctx.accounts.apply_card_effects(card_type)?;

    // Record card usage
    let card_usage = &mut ctx.accounts.card_usage;
    card_usage.user = ctx.accounts.user.key();
    card_usage.card_mint = ctx.accounts.special_card.mint;
    card_usage.card_type = card_type;
    card_usage.target_user = target_user;
    card_usage.use_timestamp = current_time;
    card_usage.effects = effects.clone();
    card_usage.bump = ctx.bumps.card_usage;

    // Update network statistics
    let network_state = &mut ctx.accounts.network_state;
    network_state.total_cards_used = network_state.total_cards_used
        .checked_add(1)
        .ok_or(NftError::ArithmeticOverflow)?;
    
    network_state.cards_used_by_type[card_type as usize] = network_state.cards_used_by_type[card_type as usize]
        .checked_add(1)
        .ok_or(NftError::ArithmeticOverflow)?;

    // Burn single-use cards
    ctx.accounts.burn_single_use_card()?;

    // Emit usage event
    emit!(CardUsedEvent {
        user: ctx.accounts.user.key(),
        card_mint: ctx.accounts.special_card.mint,
        card_type,
        target_user,
        effects: effects.clone(),
        timestamp: current_time,
        synergy_bonus: effects.synergy_bonus,
        network_effect: calculate_network_effect(&effects, network_state.total_users),
    });

    msg!("Special card used successfully");
    msg!("Card type: {}", card_type);
    msg!("Effects duration: {} seconds", effects.duration_seconds);
    if let Some(synergy) = effects.synergy_bonus {
        msg!("Synergy bonus: {}%", synergy - 100);
    }

    Ok(())
}

/// Card effects structure
#[derive(Clone, AnchorSerialize, AnchorDeserialize, Default)]
pub struct CardEffects {
    pub mining_multiplier: Option<u16>,
    pub xp_multiplier: Option<u16>,
    pub referral_multiplier: Option<u16>,
    pub rp_tier_boost: Option<u8>,
    pub guild_power_boost: Option<u64>,
    pub instant_xp: Option<u64>,
    pub streak_protection_days: Option<u8>,
    pub duration_seconds: i64,
    pub effect_type: EffectType,
    pub synergy_bonus: Option<u16>,
}

/// Effect types enumeration
#[derive(Clone, AnchorSerialize, AnchorDeserialize, PartialEq)]
pub enum EffectType {
    MiningBoost,
    XpBoost,
    ReferralBoost,
    NetworkAmplifier,
    GuildPower,
    StreakSaver,
    LevelRush,
}

impl Default for EffectType {
    fn default() -> Self {
        EffectType::MiningBoost
    }
}

/// Card usage record
#[account]
pub struct CardUsage {
    pub user: Pubkey,
    pub card_mint: Pubkey,
    pub card_type: u8,
    pub target_user: Option<Pubkey>,
    pub use_timestamp: i64,
    pub effects: CardEffects,
    pub bump: u8,
}

impl CardUsage {
    pub const SIZE: usize = 8 + // discriminator
        32 + // user
        32 + // card_mint
        1 +  // card_type
        1 + 32 + // target_user (Option<Pubkey>)
        8 +  // use_timestamp
        200 +  // effects (estimated)
        1;   // bump
}

/// Card used event
#[event]
pub struct CardUsedEvent {
    pub user: Pubkey,
    pub card_mint: Pubkey,
    pub card_type: u8,
    pub target_user: Option<Pubkey>,
    pub effects: CardEffects,
    pub timestamp: i64,
    pub synergy_bonus: Option<u16>,
    pub network_effect: u64,
}

/// Helper function to calculate network effect multiplier
fn calculate_network_effect(effects: &CardEffects, total_users: u64) -> u64 {
    let base_effect = match effects.effect_type {
        EffectType::MiningBoost => effects.mining_multiplier.unwrap_or(100) as u64,
        EffectType::XpBoost => effects.xp_multiplier.unwrap_or(100) as u64,
        EffectType::ReferralBoost => effects.referral_multiplier.unwrap_or(100) as u64,
        EffectType::NetworkAmplifier => (effects.rp_tier_boost.unwrap_or(0) as u64) * 100,
        EffectType::GuildPower => effects.guild_power_boost.unwrap_or(0) / 100,
        EffectType::StreakSaver => (effects.streak_protection_days.unwrap_or(0) as u64) * 100,
        EffectType::LevelRush => effects.instant_xp.unwrap_or(0) / 100,
    };

    // Network effect scales with user base (diminishing returns)
    let network_multiplier = (total_users as f64).ln().max(1.0) as u64;
    base_effect * network_multiplier / 100
}
