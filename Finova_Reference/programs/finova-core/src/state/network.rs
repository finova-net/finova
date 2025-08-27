// programs/finova-core/src/state/network.rs

use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::FinovaError;

/// Network-wide statistics and configurations
/// Tracks global network metrics for mining calculations and economic balance
#[account]
#[derive(Default)]
pub struct NetworkState {
    /// Network configuration authority
    pub authority: Pubkey,
    
    /// Current network phase (1-4 based on user count)
    pub current_phase: u8,
    
    /// Total registered users across the network
    pub total_users: u64,
    
    /// Total active users (active in last 30 days)
    pub active_users: u64,
    
    /// Total verified KYC users
    pub kyc_verified_users: u64,
    
    /// Total $FIN tokens in circulation
    pub total_fin_supply: u64,
    
    /// Total $FIN tokens staked
    pub total_staked_fin: u64,
    
    /// Current base mining rate (in micro-FIN per hour)
    pub base_mining_rate: u64,
    
    /// Finizen bonus multiplier (scaled by 1000)
    pub finizen_bonus: u64,
    
    /// Network quality score (0-1000, affects various calculations)
    pub network_quality_score: u64,
    
    /// Total XP earned across all users
    pub total_xp_earned: u128,
    
    /// Total referral points in the system
    pub total_rp_earned: u128,
    
    /// Number of active referral networks
    pub active_referral_networks: u64,
    
    /// Total NFTs minted
    pub total_nfts_minted: u64,
    
    /// Total special cards used
    pub total_cards_used: u64,
    
    /// Guild statistics
    pub total_guilds: u32,
    pub active_guilds: u32,
    
    /// Economic metrics
    pub total_trading_volume: u64,
    pub total_fees_collected: u64,
    pub total_tokens_burned: u64,
    
    /// Network health indicators
    pub average_session_duration: u64, // in seconds
    pub daily_active_users: u64,
    pub monthly_active_users: u64,
    
    /// Anti-bot metrics
    pub bot_detection_score: u64, // 0-1000
    pub suspicious_accounts_flagged: u64,
    pub accounts_verified_human: u64,
    
    /// Platform integration stats
    pub instagram_connections: u64,
    pub tiktok_connections: u64,
    pub youtube_connections: u64,
    pub facebook_connections: u64,
    pub twitter_connections: u64,
    
    /// Revenue tracking
    pub brand_partnership_revenue: u64,
    pub advertising_revenue: u64,
    pub nft_trading_fees: u64,
    pub premium_subscriptions: u64,
    
    /// Last update timestamp
    pub last_updated: i64,
    
    /// Reserved space for future upgrades
    pub reserved: [u64; 32],
}

impl NetworkState {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        1 + // current_phase
        8 + // total_users
        8 + // active_users
        8 + // kyc_verified_users
        8 + // total_fin_supply
        8 + // total_staked_fin
        8 + // base_mining_rate
        8 + // finizen_bonus
        8 + // network_quality_score
        16 + // total_xp_earned
        16 + // total_rp_earned
        8 + // active_referral_networks
        8 + // total_nfts_minted
        8 + // total_cards_used
        4 + // total_guilds
        4 + // active_guilds
        8 + // total_trading_volume
        8 + // total_fees_collected
        8 + // total_tokens_burned
        8 + // average_session_duration
        8 + // daily_active_users
        8 + // monthly_active_users
        8 + // bot_detection_score
        8 + // suspicious_accounts_flagged
        8 + // accounts_verified_human
        8 + // instagram_connections
        8 + // tiktok_connections
        8 + // youtube_connections
        8 + // facebook_connections
        8 + // twitter_connections
        8 + // brand_partnership_revenue
        8 + // advertising_revenue
        8 + // nft_trading_fees
        8 + // premium_subscriptions
        8 + // last_updated
        (32 * 8); // reserved

    /// Initialize network state with default values
    pub fn initialize(&mut self, authority: Pubkey) -> Result<()> {
        self.authority = authority;
        self.current_phase = 1; // Start in Finizen phase
        self.base_mining_rate = INITIAL_MINING_RATE;
        self.finizen_bonus = 2000; // 2.0x scaled by 1000
        self.network_quality_score = 800; // Start with good quality
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Calculate current mining phase based on user count
    pub fn calculate_current_phase(&self) -> u8 {
        match self.total_users {
            0..=100_000 => 1,      // Finizen phase
            100_001..=1_000_000 => 2,    // Growth phase
            1_000_001..=10_000_000 => 3, // Maturity phase
            _ => 4,                       // Stability phase
        }
    }

    /// Update mining rate based on current phase
    pub fn update_mining_rate(&mut self) -> Result<()> {
        let new_phase = self.calculate_current_phase();
        
        if new_phase != self.current_phase {
            self.current_phase = new_phase;
            
            // Update base mining rate according to phase
            self.base_mining_rate = match new_phase {
                1 => 100_000, // 0.1 FIN/hour in micro-FIN
                2 => 50_000,  // 0.05 FIN/hour
                3 => 25_000,  // 0.025 FIN/hour
                4 => 10_000,  // 0.01 FIN/hour
                _ => 10_000,
            };
            
            // Update Finizen bonus
            self.finizen_bonus = match new_phase {
                1 => 2000, // 2.0x
                2 => 1500, // 1.5x
                3 => 1200, // 1.2x
                4 => 1000, // 1.0x
                _ => 1000,
            };
        }
        
        Ok(())
    }

    /// Calculate Finizen bonus based on current user count
    pub fn calculate_finizen_bonus(&self) -> u64 {
        let bonus = 2000_u64.saturating_sub(
            (self.total_users * 1000) / 1_000_000
        );
        bonus.max(1000) // Minimum 1.0x
    }

    /// Update network quality score based on various metrics
    pub fn update_network_quality(&mut self) -> Result<()> {
        let mut quality_factors = Vec::new();
        
        // User activity ratio (0-250 points)
        if self.total_users > 0 {
            let activity_ratio = (self.active_users * 250) / self.total_users;
            quality_factors.push(activity_ratio.min(250));
        }
        
        // KYC verification ratio (0-200 points)
        if self.total_users > 0 {
            let kyc_ratio = (self.kyc_verified_users * 200) / self.total_users;
            quality_factors.push(kyc_ratio.min(200));
        }
        
        // Bot detection score (0-200 points)
        quality_factors.push((1000 - self.bot_detection_score) / 5);
        
        // Guild participation (0-150 points)
        if self.total_guilds > 0 {
            let guild_activity = (self.active_guilds * 150) / (self.total_guilds as u64);
            quality_factors.push(guild_activity.min(150));
        }
        
        // Platform diversity (0-200 points)
        let connected_platforms = [
            self.instagram_connections > 0,
            self.tiktok_connections > 0,
            self.youtube_connections > 0,
            self.facebook_connections > 0,
            self.twitter_connections > 0,
        ].iter().filter(|&&x| x).count() as u64;
        
        quality_factors.push((connected_platforms * 40).min(200));
        
        // Calculate weighted average
        let total_score = quality_factors.iter().sum::<u64>();
        self.network_quality_score = total_score.min(1000);
        
        Ok(())
    }

    /// Update user statistics
    pub fn update_user_stats(
        &mut self,
        new_users: u64,
        new_active_users: i64,
        new_kyc_users: u64,
    ) -> Result<()> {
        self.total_users = self.total_users.saturating_add(new_users);
        
        if new_active_users >= 0 {
            self.active_users = self.active_users.saturating_add(new_active_users as u64);
        } else {
            self.active_users = self.active_users.saturating_sub((-new_active_users) as u64);
        }
        
        self.kyc_verified_users = self.kyc_verified_users.saturating_add(new_kyc_users);
        
        // Update mining rate based on new user count
        self.update_mining_rate()?;
        self.update_network_quality()?;
        self.last_updated = Clock::get()?.unix_timestamp;
        
        Ok(())
    }

    /// Update economic metrics
    pub fn update_economic_metrics(
        &mut self,
        trading_volume: u64,
        fees_collected: u64,
        tokens_burned: u64,
    ) -> Result<()> {
        self.total_trading_volume = self.total_trading_volume.saturating_add(trading_volume);
        self.total_fees_collected = self.total_fees_collected.saturating_add(fees_collected);
        self.total_tokens_burned = self.total_tokens_burned.saturating_add(tokens_burned);
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update revenue metrics
    pub fn update_revenue_metrics(
        &mut self,
        brand_revenue: u64,
        ad_revenue: u64,
        nft_fees: u64,
        premium_subs: u64,
    ) -> Result<()> {
        self.brand_partnership_revenue = self.brand_partnership_revenue.saturating_add(brand_revenue);
        self.advertising_revenue = self.advertising_revenue.saturating_add(ad_revenue);
        self.nft_trading_fees = self.nft_trading_fees.saturating_add(nft_fees);
        self.premium_subscriptions = self.premium_subscriptions.saturating_add(premium_subs);
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update social platform connections
    pub fn update_platform_connections(
        &mut self,
        platform: &str,
        new_connections: u64,
    ) -> Result<()> {
        match platform.to_lowercase().as_str() {
            "instagram" => self.instagram_connections = self.instagram_connections.saturating_add(new_connections),
            "tiktok" => self.tiktok_connections = self.tiktok_connections.saturating_add(new_connections),
            "youtube" => self.youtube_connections = self.youtube_connections.saturating_add(new_connections),
            "facebook" => self.facebook_connections = self.facebook_connections.saturating_add(new_connections),
            "twitter" | "x" => self.twitter_connections = self.twitter_connections.saturating_add(new_connections),
            _ => return Err(FinovaError::InvalidPlatform.into()),
        }
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update bot detection metrics
    pub fn update_bot_metrics(
        &mut self,
        flagged_accounts: u64,
        verified_humans: u64,
        new_detection_score: u64,
    ) -> Result<()> {
        self.suspicious_accounts_flagged = self.suspicious_accounts_flagged.saturating_add(flagged_accounts);
        self.accounts_verified_human = self.accounts_verified_human.saturating_add(verified_humans);
        
        // Update bot detection score (weighted average)
        let current_weight = 80; // 80% current score
        let new_weight = 20;     // 20% new score
        
        self.bot_detection_score = (
            (self.bot_detection_score * current_weight) + 
            (new_detection_score.min(1000) * new_weight)
        ) / 100;
        
        self.update_network_quality()?;
        self.last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Get current network health score (0-100)
    pub fn get_network_health_score(&self) -> u8 {
        let mut health_factors = Vec::new();
        
        // User growth (0-25 points)
        let growth_score = if self.total_users < 1000 {
            (self.total_users / 40) as u8 // Up to 25 points for first 1000 users
        } else {
            25
        };
        health_factors.push(growth_score);
        
        // Activity ratio (0-25 points)
        let activity_score = if self.total_users > 0 {
            ((self.active_users * 25) / self.total_users) as u8
        } else {
            0
        };
        health_factors.push(activity_score.min(25));
        
        // Network quality (0-25 points)
        let quality_score = (self.network_quality_score / 40) as u8;
        health_factors.push(quality_score.min(25));
        
        // Economic activity (0-25 points)
        let economic_score = if self.total_trading_volume > 1_000_000 {
            25
        } else {
            (self.total_trading_volume / 40_000) as u8
        };
        health_factors.push(economic_score.min(25));
        
        health_factors.iter().sum::<u8>().min(100)
    }

    /// Check if network upgrade is needed
    pub fn needs_upgrade(&self) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        let time_since_update = current_time - self.last_updated;
        
        // Upgrade needed if:
        // 1. Not updated in last 24 hours
        // 2. Phase change detected
        // 3. Network health is below 50
        time_since_update > 86400 || // 24 hours
        self.calculate_current_phase() != self.current_phase ||
        self.get_network_health_score() < 50
    }

    /// Validate network state consistency
    pub fn validate_state(&self) -> Result<()> {
        require!(
            self.active_users <= self.total_users,
            FinovaError::InvalidNetworkState
        );
        
        require!(
            self.kyc_verified_users <= self.total_users,
            FinovaError::InvalidNetworkState
        );
        
        require!(
            self.current_phase >= 1 && self.current_phase <= 4,
            FinovaError::InvalidNetworkState
        );
        
        require!(
            self.network_quality_score <= 1000,
            FinovaError::InvalidNetworkState
        );
        
        require!(
            self.bot_detection_score <= 1000,
            FinovaError::InvalidNetworkState
        );
        
        Ok(())
    }
}

/// Network configuration parameters that can be updated by governance
#[account]
#[derive(Default)]
pub struct NetworkConfig {
    /// Configuration authority (governance program)
    pub authority: Pubkey,
    
    /// Minimum mining rate (cannot go below this)
    pub min_mining_rate: u64,
    
    /// Maximum mining rate (cannot go above this)
    pub max_mining_rate: u64,
    
    /// Quality score thresholds for various bonuses
    pub quality_thresholds: [u64; 5], // 5 different quality levels
    
    /// Bot detection sensitivity (0-1000)
    pub bot_detection_sensitivity: u64,
    
    /// KYC requirement threshold (user count after which KYC becomes mandatory)
    pub kyc_requirement_threshold: u64,
    
    /// Maximum referral depth for rewards
    pub max_referral_depth: u8,
    
    /// Guild size limits
    pub min_guild_size: u32,
    pub max_guild_size: u32,
    
    /// Fee structure (scaled by 10000 for precision)
    pub trading_fee: u64,     // e.g., 30 = 0.30%
    pub withdrawal_fee: u64,
    pub nft_creation_fee: u64,
    
    /// Staking parameters
    pub min_staking_amount: u64,
    pub max_staking_duration: u64, // in seconds
    pub base_staking_apy: u64,     // scaled by 10000
    
    /// Emergency controls
    pub emergency_pause: bool,
    pub maintenance_mode: bool,
    
    /// Feature flags
    pub social_mining_enabled: bool,
    pub nft_trading_enabled: bool,
    pub guild_system_enabled: bool,
    pub cross_chain_enabled: bool,
    
    /// Update timestamp
    pub last_config_update: i64,
    
    /// Reserved for future parameters
    pub reserved: [u64; 16],
}

impl NetworkConfig {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 + // min_mining_rate
        8 + // max_mining_rate
        (5 * 8) + // quality_thresholds
        8 + // bot_detection_sensitivity
        8 + // kyc_requirement_threshold
        1 + // max_referral_depth
        4 + // min_guild_size
        4 + // max_guild_size
        8 + // trading_fee
        8 + // withdrawal_fee
        8 + // nft_creation_fee
        8 + // min_staking_amount
        8 + // max_staking_duration
        8 + // base_staking_apy
        1 + // emergency_pause
        1 + // maintenance_mode
        1 + // social_mining_enabled
        1 + // nft_trading_enabled
        1 + // guild_system_enabled
        1 + // cross_chain_enabled
        8 + // last_config_update
        (16 * 8); // reserved

    pub fn initialize(&mut self, authority: Pubkey) -> Result<()> {
        self.authority = authority;
        self.min_mining_rate = 1_000; // 0.001 FIN/hour minimum
        self.max_mining_rate = 500_000; // 0.5 FIN/hour maximum
        self.quality_thresholds = [200, 400, 600, 800, 900]; // Quality level thresholds
        self.bot_detection_sensitivity = 700; // 70% sensitivity
        self.kyc_requirement_threshold = 10_000; // KYC required after 10k users
        self.max_referral_depth = 3; // 3 levels of referral rewards
        self.min_guild_size = 10;
        self.max_guild_size = 50;
        self.trading_fee = 30; // 0.30%
        self.withdrawal_fee = 10; // 0.10%
        self.nft_creation_fee = 5; // 0.05%
        self.min_staking_amount = 100_000_000; // 100 FIN minimum stake
        self.max_staking_duration = 31_536_000; // 1 year maximum
        self.base_staking_apy = 800; // 8% base APY
        self.social_mining_enabled = true;
        self.nft_trading_enabled = true;
        self.guild_system_enabled = true;
        self.cross_chain_enabled = false; // Start disabled
        self.last_config_update = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn update_config(
        &mut self,
        authority: Pubkey,
        updates: NetworkConfigUpdate,
    ) -> Result<()> {
        require!(
            self.authority == authority,
            FinovaError::UnauthorizedAccess
        );

        if let Some(min_rate) = updates.min_mining_rate {
            self.min_mining_rate = min_rate;
        }
        if let Some(max_rate) = updates.max_mining_rate {
            self.max_mining_rate = max_rate;
        }
        if let Some(sensitivity) = updates.bot_detection_sensitivity {
            self.bot_detection_sensitivity = sensitivity.min(1000);
        }
        if let Some(threshold) = updates.kyc_requirement_threshold {
            self.kyc_requirement_threshold = threshold;
        }

        self.last_config_update = Clock::get()?.unix_timestamp;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NetworkConfigUpdate {
    pub min_mining_rate: Option<u64>,
    pub max_mining_rate: Option<u64>,
    pub bot_detection_sensitivity: Option<u64>,
    pub kyc_requirement_threshold: Option<u64>,
    pub trading_fee: Option<u64>,
    pub withdrawal_fee: Option<u64>,
    pub emergency_pause: Option<bool>,
    pub maintenance_mode: Option<bool>,
}
