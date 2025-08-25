// programs/finova-nft/src/events/use_card.rs

use anchor_lang::prelude::*;
use crate::state::{SpecialCard, CardRarity, CardType, CardEffect};

/// Event emitted when a special card is used by a user
#[event]
pub struct CardUsed {
    /// The public key of the user who used the card
    pub user: Pubkey,
    /// The mint address of the NFT card that was used
    pub card_mint: Pubkey,
    /// The type of card that was used
    pub card_type: CardType,
    /// The rarity of the card
    pub card_rarity: CardRarity,
    /// The effect that was applied
    pub card_effect: CardEffect,
    /// The duration of the effect in seconds
    pub effect_duration: u64,
    /// The multiplier value applied
    pub multiplier_value: u64,
    /// Whether the card was consumed (single-use cards)
    pub card_consumed: bool,
    /// Timestamp when the card was used
    pub timestamp: i64,
    /// The slot when the card was used
    pub slot: u64,
    /// Additional metadata about the card usage
    pub metadata: CardUsageMetadata,
}

/// Event emitted when a card effect expires
#[event]
pub struct CardEffectExpired {
    /// The public key of the user whose card effect expired
    pub user: Pubkey,
    /// The mint address of the NFT card whose effect expired
    pub card_mint: Pubkey,
    /// The type of card whose effect expired
    pub card_type: CardType,
    /// The effect that expired
    pub card_effect: CardEffect,
    /// Timestamp when the effect expired
    pub timestamp: i64,
    /// The slot when the effect expired
    pub slot: u64,
    /// Whether the effect was manually removed or naturally expired
    pub manual_removal: bool,
}

/// Event emitted when multiple cards are used simultaneously (combo)
#[event]
pub struct CardComboUsed {
    /// The public key of the user who used the combo
    pub user: Pubkey,
    /// Array of card mints used in the combo
    pub card_mints: Vec<Pubkey>,
    /// Array of card types in the combo
    pub card_types: Vec<CardType>,
    /// Array of card rarities in the combo
    pub card_rarities: Vec<CardRarity>,
    /// The combined synergy multiplier
    pub synergy_multiplier: u64,
    /// Base combo bonus percentage (in basis points)
    pub combo_bonus_bps: u16,
    /// Total number of cards in the combo
    pub combo_size: u8,
    /// Whether this achieved a perfect combo (all rarities)
    pub perfect_combo: bool,
    /// Timestamp when the combo was used
    pub timestamp: i64,
    /// The slot when the combo was used
    pub slot: u64,
}

/// Event emitted when a card is upgraded or evolved
#[event]
pub struct CardEvolved {
    /// The public key of the user who evolved the card
    pub user: Pubkey,
    /// The mint address of the original card
    pub original_card_mint: Pubkey,
    /// The mint address of the new evolved card
    pub evolved_card_mint: Pubkey,
    /// The original card type
    pub original_card_type: CardType,
    /// The new evolved card type
    pub evolved_card_type: CardType,
    /// The original rarity
    pub original_rarity: CardRarity,
    /// The new evolved rarity
    pub evolved_rarity: CardRarity,
    /// Materials consumed in the evolution
    pub materials_consumed: Vec<EvolutionMaterial>,
    /// Cost paid for the evolution
    pub evolution_cost: u64,
    /// Success probability of the evolution
    pub success_probability: u8,
    /// Whether evolution was successful
    pub evolution_successful: bool,
    /// Timestamp when the evolution occurred
    pub timestamp: i64,
    /// The slot when the evolution occurred
    pub slot: u64,
}

/// Event emitted when a card effect is stacked with another
#[event]
pub struct CardEffectStacked {
    /// The public key of the user stacking effects
    pub user: Pubkey,
    /// The mint address of the new card being stacked
    pub new_card_mint: Pubkey,
    /// The mint address of the existing card being stacked with
    pub existing_card_mint: Pubkey,
    /// The card type being stacked
    pub card_type: CardType,
    /// The combined effect multiplier
    pub combined_multiplier: u64,
    /// Maximum stack count for this card type
    pub max_stack_count: u8,
    /// Current stack count after this operation
    pub current_stack_count: u8,
    /// Whether maximum stack was reached
    pub max_stack_reached: bool,
    /// Stack efficiency percentage (diminishing returns)
    pub stack_efficiency_pct: u8,
    /// Timestamp when the stacking occurred
    pub timestamp: i64,
    /// The slot when the stacking occurred
    pub slot: u64,
}

/// Event emitted when a rare card triggers special effects
#[event]
pub struct RareCardTriggered {
    /// The public key of the user whose card triggered
    pub user: Pubkey,
    /// The mint address of the rare card
    pub card_mint: Pubkey,
    /// The type of rare card
    pub card_type: CardType,
    /// The rarity level that triggered
    pub card_rarity: CardRarity,
    /// Special effect that was triggered
    pub special_effect: RareCardSpecialEffect,
    /// Bonus value applied
    pub bonus_value: u64,
    /// Whether this was a critical trigger (double effect)
    pub critical_trigger: bool,
    /// Trigger probability that was beaten
    pub trigger_probability: u16,
    /// Network-wide notification sent
    pub network_notification: bool,
    /// Timestamp when the trigger occurred
    pub timestamp: i64,
    /// The slot when the trigger occurred
    pub slot: u64,
}

/// Event emitted when card trading occurs in the marketplace
#[event]
pub struct CardTraded {
    /// The seller of the card
    pub seller: Pubkey,
    /// The buyer of the card
    pub buyer: Pubkey,
    /// The mint address of the traded card
    pub card_mint: Pubkey,
    /// Price paid for the card
    pub price: u64,
    /// Currency used for payment (FIN, SOL, etc.)
    pub currency_mint: Pubkey,
    /// Platform fee charged
    pub platform_fee: u64,
    /// Royalty fee paid to creator
    pub royalty_fee: u64,
    /// Card type that was traded
    pub card_type: CardType,
    /// Card rarity that was traded
    pub card_rarity: CardRarity,
    /// Whether this was an auction or direct sale
    pub auction_sale: bool,
    /// Market cap impact of this trade
    pub market_impact_bps: u16,
    /// Timestamp when the trade occurred
    pub timestamp: i64,
    /// The slot when the trade occurred
    pub slot: u64,
}

/// Event emitted for card usage analytics and tracking
#[event]
pub struct CardAnalytics {
    /// The public key of the user
    pub user: Pubkey,
    /// The mint address of the card
    pub card_mint: Pubkey,
    /// Usage count for this card
    pub usage_count: u32,
    /// Total value generated by this card
    pub total_value_generated: u64,
    /// Average effect duration
    pub avg_effect_duration: u64,
    /// Best combo this card was part of
    pub best_combo_multiplier: u64,
    /// ROI of this card (return on investment)
    pub card_roi_bps: u16,
    /// User's total card collection value
    pub total_collection_value: u64,
    /// User's card usage efficiency score
    pub usage_efficiency_score: u8,
    /// Timestamp of the analytics snapshot
    pub timestamp: i64,
    /// The slot of the analytics snapshot
    pub slot: u64,
}

/// Metadata for card usage tracking
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct CardUsageMetadata {
    /// Current mining rate before card usage
    pub pre_usage_mining_rate: u64,
    /// New mining rate after card usage
    pub post_usage_mining_rate: u64,
    /// Current XP multiplier before card usage
    pub pre_usage_xp_multiplier: u64,
    /// New XP multiplier after card usage
    pub post_usage_xp_multiplier: u64,
    /// Current referral bonus before card usage
    pub pre_usage_referral_bonus: u64,
    /// New referral bonus after card usage
    pub post_usage_referral_bonus: u64,
    /// User's current level
    pub user_level: u16,
    /// User's current XP
    pub user_xp: u64,
    /// User's total cards owned
    pub total_cards_owned: u32,
    /// User's card usage streak
    pub card_usage_streak: u16,
    /// Network congestion at time of usage
    pub network_congestion_score: u8,
}

/// Materials used in card evolution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct EvolutionMaterial {
    /// Material type identifier
    pub material_type: u8,
    /// Mint address of the material token
    pub material_mint: Pubkey,
    /// Quantity of material consumed
    pub quantity: u64,
    /// Rarity of the material
    pub material_rarity: u8,
}

/// Special effects for rare cards
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum RareCardSpecialEffect {
    /// Double all rewards for duration
    DoubleRewards {
        duration_hours: u8,
    },
    /// Grant temporary VIP status
    VipStatus {
        duration_days: u8,
    },
    /// Unlock exclusive mining pool
    ExclusiveMiningPool {
        pool_id: u32,
        access_duration_hours: u16,
    },
    /// Network-wide bonus distribution
    NetworkBonus {
        bonus_amount: u64,
        distribution_type: u8,
    },
    /// Rare NFT airdrop
    NftAirdrop {
        collection_id: u32,
        nft_rarity: u8,
    },
    /// Guild leadership privileges
    GuildLeadership {
        privilege_level: u8,
        duration_days: u16,
    },
    /// Custom effect defined by card metadata
    CustomEffect {
        effect_id: u32,
        parameters: Vec<u64>,
    },
}

/// Event emitted when a card effect is boosted by external factors
#[event]
pub struct CardEffectBoosted {
    /// The public key of the user whose card was boosted
    pub user: Pubkey,
    /// The mint address of the boosted card
    pub card_mint: Pubkey,
    /// Source of the boost
    pub boost_source: CardBoostSource,
    /// Original effect multiplier
    pub original_multiplier: u64,
    /// Boosted effect multiplier
    pub boosted_multiplier: u64,
    /// Boost percentage applied
    pub boost_percentage: u16,
    /// Duration of the boost
    pub boost_duration: u64,
    /// Whether the boost is permanent
    pub permanent_boost: bool,
    /// Timestamp when the boost was applied
    pub timestamp: i64,
    /// The slot when the boost was applied
    pub slot: u64,
}

/// Sources of card effect boosts
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum CardBoostSource {
    /// Boost from guild membership
    GuildMembership {
        guild_id: u64,
        guild_level: u8,
    },
    /// Boost from staking tokens
    TokenStaking {
        staked_amount: u64,
        staking_tier: u8,
    },
    /// Boost from achieving milestones
    Milestone {
        milestone_id: u32,
        achievement_level: u8,
    },
    /// Boost from network events
    NetworkEvent {
        event_id: u32,
        participation_score: u16,
    },
    /// Boost from other cards in inventory
    CardSynergy {
        synergy_cards: Vec<Pubkey>,
        synergy_type: u8,
    },
    /// Boost from premium subscription
    PremiumSubscription {
        subscription_tier: u8,
        remaining_days: u16,
    },
    /// Boost from social media engagement
    SocialEngagement {
        platform_id: u8,
        engagement_score: u32,
    },
}

/// Event emitted when card durability is affected
#[event]
pub struct CardDurabilityChanged {
    /// The public key of the user whose card durability changed
    pub user: Pubkey,
    /// The mint address of the card
    pub card_mint: Pubkey,
    /// Previous durability value
    pub previous_durability: u16,
    /// New durability value
    pub new_durability: u16,
    /// Reason for durability change
    pub change_reason: DurabilityChangeReason,
    /// Whether the card is now broken/unusable
    pub card_broken: bool,
    /// Repair cost if applicable
    pub repair_cost: Option<u64>,
    /// Timestamp when durability changed
    pub timestamp: i64,
    /// The slot when durability changed
    pub slot: u64,
}

/// Reasons for card durability changes
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum DurabilityChangeReason {
    /// Normal usage wear
    NormalUsage,
    /// Overuse penalty
    Overuse,
    /// Repair operation
    Repair,
    /// Upgrade/enhancement
    Enhancement,
    /// Decay over time
    TimeDecay,
    /// Battle/competition damage
    BattleDamage,
    /// Environmental effects
    Environmental,
}

/// Helper functions for card events
impl CardUsed {
    /// Create a new CardUsed event
    pub fn new(
        user: Pubkey,
        card_mint: Pubkey,
        special_card: &SpecialCard,
        effect_duration: u64,
        metadata: CardUsageMetadata,
    ) -> Self {
        Self {
            user,
            card_mint,
            card_type: special_card.card_type,
            card_rarity: special_card.rarity,
            card_effect: special_card.effect,
            effect_duration,
            multiplier_value: special_card.effect_value,
            card_consumed: special_card.single_use,
            timestamp: Clock::get().unwrap().unix_timestamp,
            slot: Clock::get().unwrap().slot,
            metadata,
        }
    }
}

impl CardComboUsed {
    /// Calculate synergy multiplier for a combo
    pub fn calculate_synergy_multiplier(
        card_types: &[CardType],
        card_rarities: &[CardRarity],
    ) -> (u64, u16, bool) {
        let base_multiplier = 1000; // 1.0x in basis points
        let combo_size = card_types.len();
        
        // Base combo bonus
        let mut combo_bonus_bps = match combo_size {
            2 => 150,  // 15% bonus
            3 => 300,  // 30% bonus
            4 => 500,  // 50% bonus
            5 => 750,  // 75% bonus
            _ => 1000, // 100% bonus for 6+
        };
        
        // Type diversity bonus
        let unique_types: std::collections::HashSet<_> = card_types.iter().collect();
        if unique_types.len() == card_types.len() {
            combo_bonus_bps += 200; // 20% bonus for all different types
        }
        
        // Rarity bonus
        let mut rarity_bonus = 0;
        let mut has_all_rarities = true;
        let expected_rarities = [
            CardRarity::Common,
            CardRarity::Uncommon,
            CardRarity::Rare,
            CardRarity::Epic,
            CardRarity::Legendary,
        ];
        
        for rarity in card_rarities {
            rarity_bonus += match rarity {
                CardRarity::Common => 50,
                CardRarity::Uncommon => 100,
                CardRarity::Rare => 200,
                CardRarity::Epic => 400,
                CardRarity::Legendary => 800,
            };
        }
        
        // Check for perfect combo (all rarities present)
        for expected_rarity in &expected_rarities {
            if !card_rarities.contains(expected_rarity) {
                has_all_rarities = false;
                break;
            }
        }
        
        if has_all_rarities {
            combo_bonus_bps += 1000; // 100% bonus for perfect combo
        }
        
        let total_multiplier = base_multiplier + combo_bonus_bps + rarity_bonus;
        
        (total_multiplier as u64, combo_bonus_bps, has_all_rarities)
    }
}

/// Event processing utilities
pub mod event_utils {
    use super::*;
    
    /// Emit card used event with proper validation
    pub fn emit_card_used_event(
        user: Pubkey,
        card_mint: Pubkey,
        special_card: &SpecialCard,
        effect_duration: u64,
        metadata: CardUsageMetadata,
    ) -> Result<()> {
        // Validate card can be used
        require!(
            special_card.is_active(),
            crate::errors::FinovaNftError::CardNotActive
        );
        
        // Create and emit event
        let event = CardUsed::new(
            user,
            card_mint,
            special_card,
            effect_duration,
            metadata,
        );
        
        emit!(event);
        Ok(())
    }
    
    /// Emit combo event with validation
    pub fn emit_combo_event(
        user: Pubkey,
        cards: &[(Pubkey, &SpecialCard)],
    ) -> Result<()> {
        require!(
            cards.len() >= 2,
            crate::errors::FinovaNftError::InsufficientCardsForCombo
        );
        
        let card_mints: Vec<Pubkey> = cards.iter().map(|(mint, _)| *mint).collect();
        let card_types: Vec<CardType> = cards.iter().map(|(_, card)| card.card_type).collect();
        let card_rarities: Vec<CardRarity> = cards.iter().map(|(_, card)| card.rarity).collect();
        
        let (synergy_multiplier, combo_bonus_bps, perfect_combo) = 
            CardComboUsed::calculate_synergy_multiplier(&card_types, &card_rarities);
        
        let event = CardComboUsed {
            user,
            card_mints,
            card_types,
            card_rarities,
            synergy_multiplier,
            combo_bonus_bps,
            combo_size: cards.len() as u8,
            perfect_combo,
            timestamp: Clock::get().unwrap().unix_timestamp,
            slot: Clock::get().unwrap().slot,
        };
        
        emit!(event);
        Ok(())
    }
    
    /// Calculate usage efficiency score
    pub fn calculate_usage_efficiency(
        usage_count: u32,
        total_value_generated: u64,
        card_cost: u64,
    ) -> u8 {
        if card_cost == 0 {
            return 100;
        }
        
        let roi = (total_value_generated * 100) / card_cost;
        let usage_factor = std::cmp::min(usage_count, 100) as u64;
        
        let efficiency = (roi + usage_factor) / 2;
        std::cmp::min(efficiency, 100) as u8
    }
    
    /// Validate card evolution requirements
    pub fn validate_evolution_requirements(
        original_card: &SpecialCard,
        materials: &[EvolutionMaterial],
        user_balance: u64,
        evolution_cost: u64,
    ) -> Result<u8> {
        // Check user has enough balance
        require!(
            user_balance >= evolution_cost,
            crate::errors::FinovaNftError::InsufficientFunds
        );
        
        // Check card can be evolved
        require!(
            original_card.can_evolve(),
            crate::errors::FinovaNftError::CardCannotEvolve
        );
        
        // Validate materials
        require!(
            !materials.is_empty(),
            crate::errors::FinovaNftError::NoEvolutionMaterials
        );
        
        // Calculate success probability based on materials and card rarity
        let mut success_prob = match original_card.rarity {
            CardRarity::Common => 95,
            CardRarity::Uncommon => 85,
            CardRarity::Rare => 70,
            CardRarity::Epic => 50,
            CardRarity::Legendary => 25,
        };
        
        // Boost probability with high-quality materials
        for material in materials {
            success_prob += std::cmp::min(material.material_rarity * 5, 20);
        }
        
        Ok(std::cmp::min(success_prob, 98))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_synergy_multiplier_calculation() {
        let card_types = vec![
            CardType::MiningBoost,
            CardType::XpAccelerator,
            CardType::ReferralPower,
        ];
        let card_rarities = vec![
            CardRarity::Rare,
            CardRarity::Epic,
            CardRarity::Legendary,
        ];
        
        let (multiplier, bonus, perfect) = CardComboUsed::calculate_synergy_multiplier(
            &card_types,
            &card_rarities,
        );
        
        assert!(multiplier > 1000); // Should be greater than base
        assert!(bonus > 0);
        assert!(!perfect); // Not all rarities present
    }
    
    #[test]
    fn test_perfect_combo_detection() {
        let card_types = vec![
            CardType::MiningBoost,
            CardType::XpAccelerator,
            CardType::ReferralPower,
            CardType::MiningBoost,
            CardType::XpAccelerator,
        ];
        let card_rarities = vec![
            CardRarity::Common,
            CardRarity::Uncommon,
            CardRarity::Rare,
            CardRarity::Epic,
            CardRarity::Legendary,
        ];
        
        let (multiplier, bonus, perfect) = CardComboUsed::calculate_synergy_multiplier(
            &card_types,
            &card_rarities,
        );
        
        assert!(perfect); // All rarities present
        assert!(multiplier > 2000); // Should have significant bonus
    }
    
    #[test]
    fn test_usage_efficiency_calculation() {
        let efficiency = event_utils::calculate_usage_efficiency(
            50,    // usage_count
            10000, // total_value_generated
            5000,  // card_cost
        );
        
        assert!(efficiency > 0);
        assert!(efficiency <= 100);
    }
}
