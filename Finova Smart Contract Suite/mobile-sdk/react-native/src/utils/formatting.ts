/**
 * Finova Network - React Native SDK Formatting Utilities
 * Advanced formatting for tokens, time, stats, addresses, and achievements
 * 
 * @version 3.0.0
 * @author Finova Network Team
 * @license MIT
 */

import { PublicKey } from '@solana/web3.js';

// ============================================================================
// TYPE DEFINITIONS
// ============================================================================

export interface TokenAmount {
  amount: number | string;
  decimals?: number;
  symbol?: string;
}

export interface UserStats {
  xp: number;
  level: number;
  rpTier: string;
  miningRate: number;
  totalMined: number;
  networkSize: number;
}

export interface MiningStats {
  currentRate: number;
  dailyMined: number;
  totalMined: number;
  efficiency: number;
  phase: number;
  timeActive: number;
}

export interface SocialStats {
  platform: string;
  followers: number;
  engagement: number;
  posts: number;
  views: number;
  qualityScore: number;
}

export interface Achievement {
  id: string;
  title: string;
  description: string;
  rarity: 'common' | 'uncommon' | 'rare' | 'epic' | 'legendary';
  progress: number;
  maxProgress: number;
  unlocked: boolean;
  unlockedAt?: Date;
}

// ============================================================================
// TOKEN FORMATTING
// ============================================================================

export class TokenFormatter {
  private static readonly DECIMALS = {
    FIN: 9,
    sFIN: 9,
    USDfin: 6,
    sUSDfin: 6,
    SOL: 9,
    USDC: 6
  } as const;

  /**
   * Format token amount with proper decimals and symbol
   */
  static formatToken(token: TokenAmount): string {
    const { amount, decimals = 9, symbol = 'FIN' } = token;
    const numAmount = typeof amount === 'string' ? parseFloat(amount) : amount;
    
    if (isNaN(numAmount) || numAmount < 0) return '0 ' + symbol;
    
    // Handle very small amounts
    if (numAmount < 0.000001) {
      return `<0.000001 ${symbol}`;
    }
    
    // Format based on amount size
    let formatted: string;
    if (numAmount >= 1e9) {
      formatted = (numAmount / 1e9).toFixed(2) + 'B';
    } else if (numAmount >= 1e6) {
      formatted = (numAmount / 1e6).toFixed(2) + 'M';
    } else if (numAmount >= 1e3) {
      formatted = (numAmount / 1e3).toFixed(2) + 'K';
    } else if (numAmount >= 1) {
      formatted = numAmount.toFixed(6).replace(/\.?0+$/, '');
    } else {
      formatted = numAmount.toFixed(Math.min(decimals, 9)).replace(/\.?0+$/, '');
    }
    
    return `${formatted} ${symbol}`;
  }

  /**
   * Format mining rate with time unit
   */
  static formatMiningRate(ratePerHour: number): string {
    if (ratePerHour >= 1) {
      return `${ratePerHour.toFixed(3)} FIN/hr`;
    } else if (ratePerHour >= 0.001) {
      return `${(ratePerHour * 1000).toFixed(2)} mFIN/hr`;
    } else {
      return `${(ratePerHour * 1000000).toFixed(0)} μFIN/hr`;
    }
  }

  /**
   * Format percentage with precision
   */
  static formatPercentage(value: number, precision: number = 2): string {
    if (isNaN(value)) return '0%';
    return `${(value * 100).toFixed(precision)}%`;
  }

  /**
   * Format APY with color coding
   */
  static formatAPY(apy: number): { formatted: string; color: string } {
    const formatted = `${apy.toFixed(1)}% APY`;
    let color = '#10B981'; // green
    
    if (apy < 5) color = '#EF4444'; // red
    else if (apy < 10) color = '#F59E0B'; // yellow
    else if (apy >= 15) color = '#8B5CF6'; // purple
    
    return { formatted, color };
  }
}

// ============================================================================
// TIME FORMATTING
// ============================================================================

export class TimeFormatter {
  /**
   * Format time duration to human readable string
   */
  static formatDuration(seconds: number): string {
    if (seconds < 0) return 'Invalid time';
    
    const units = [
      { label: 'd', value: 86400 },
      { label: 'h', value: 3600 },
      { label: 'm', value: 60 },
      { label: 's', value: 1 }
    ];
    
    const parts: string[] = [];
    let remaining = Math.floor(seconds);
    
    for (const unit of units) {
      const count = Math.floor(remaining / unit.value);
      if (count > 0) {
        parts.push(`${count}${unit.label}`);
        remaining %= unit.value;
      }
      if (parts.length >= 2) break; // Show max 2 units
    }
    
    return parts.length > 0 ? parts.join(' ') : '0s';
  }

  /**
   * Format relative time (e.g., "2 hours ago")
   */
  static formatRelativeTime(date: Date): string {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    
    if (diffSec < 60) return 'just now';
    if (diffSec < 3600) return `${Math.floor(diffSec / 60)}m ago`;
    if (diffSec < 86400) return `${Math.floor(diffSec / 3600)}h ago`;
    if (diffSec < 604800) return `${Math.floor(diffSec / 86400)}d ago`;
    if (diffSec < 2592000) return `${Math.floor(diffSec / 604800)}w ago`;
    
    return date.toLocaleDateString();
  }

  /**
   * Format countdown timer
   */
  static formatCountdown(endTime: Date): string {
    const now = new Date();
    const remaining = Math.max(0, endTime.getTime() - now.getTime());
    
    if (remaining === 0) return 'Expired';
    
    return this.formatDuration(remaining / 1000);
  }

  /**
   * Format streak with fire emoji intensity
   */
  static formatStreak(days: number): string {
    let fire = '🔥';
    if (days >= 100) fire = '🔥🔥🔥';
    else if (days >= 30) fire = '🔥🔥';
    
    return `${days} ${fire}`;
  }
}

// ============================================================================
// USER STATS FORMATTING
// ============================================================================

export class UserStatsFormatter {
  /**
   * Format XP with level progression
   */
  static formatXP(xp: number, level: number): string {
    if (xp >= 1e6) {
      return `${(xp / 1e6).toFixed(1)}M XP (Lv.${level})`;
    } else if (xp >= 1e3) {
      return `${(xp / 1e3).toFixed(1)}K XP (Lv.${level})`;
    }
    return `${xp.toLocaleString()} XP (Lv.${level})`;
  }

  /**
   * Format level with tier badge
   */
  static formatLevel(level: number): { badge: string; tier: string; color: string } {
    if (level >= 101) return { badge: '👑', tier: 'Mythic', color: '#FF6B9D' };
    if (level >= 76) return { badge: '💎', tier: 'Diamond', color: '#60A5FA' };
    if (level >= 51) return { badge: '🏆', tier: 'Platinum', color: '#A78BFA' };
    if (level >= 26) return { badge: '🥇', tier: 'Gold', color: '#FBBF24' };
    if (level >= 11) return { badge: '🥈', tier: 'Silver', color: '#E5E7EB' };
    return { badge: '🥉', tier: 'Bronze', color: '#CD7F32' };
  }

  /**
   * Format RP tier with benefits
   */
  static formatRPTier(rp: number): { tier: string; bonus: string; color: string } {
    if (rp >= 50000) return { tier: 'Ambassador', bonus: '+200%', color: '#8B5CF6' };
    if (rp >= 15000) return { tier: 'Leader', bonus: '+100%', color: '#F59E0B' };
    if (rp >= 5000) return { tier: 'Influencer', bonus: '+50%', color: '#10B981' };
    if (rp >= 1000) return { tier: 'Connector', bonus: '+20%', color: '#3B82F6' };
    return { tier: 'Explorer', bonus: '+0%', color: '#6B7280' };
  }

  /**
   * Format complete user stats
   */
  static formatUserCard(stats: UserStats): {
    xpDisplay: string;
    levelInfo: ReturnType<typeof this.formatLevel>;
    rpInfo: ReturnType<typeof this.formatRPTier>;
    miningInfo: string;
    networkInfo: string;
  } {
    return {
      xpDisplay: this.formatXP(stats.xp, stats.level),
      levelInfo: this.formatLevel(stats.level),
      rpInfo: this.formatRPTier(parseInt(stats.rpTier) || 0),
      miningInfo: TokenFormatter.formatMiningRate(stats.miningRate),
      networkInfo: `${stats.networkSize.toLocaleString()} referrals`
    };
  }
}

// ============================================================================
// MINING STATS FORMATTING
// ============================================================================

export class MiningStatsFormatter {
  /**
   * Format mining efficiency with visual indicator
   */
  static formatEfficiency(efficiency: number): { 
    percentage: string; 
    indicator: string; 
    color: string 
  } {
    const percentage = `${(efficiency * 100).toFixed(1)}%`;
    let indicator: string;
    let color: string;
    
    if (efficiency >= 0.9) {
      indicator = '🚀';
      color = '#10B981';
    } else if (efficiency >= 0.7) {
      indicator = '⚡';
      color = '#F59E0B';
    } else if (efficiency >= 0.5) {
      indicator = '⚠️';
      color = '#EF4444';
    } else {
      indicator = '🐌';
      color = '#6B7280';
    }
    
    return { percentage, indicator, color };
  }

  /**
   * Format mining phase with progress
   */
  static formatPhase(phase: number, totalUsers: number): string {
    const phases = [
      { name: 'Finizen', max: 100000, emoji: '🌱' },
      { name: 'Growth', max: 1000000, emoji: '🌿' },
      { name: 'Maturity', max: 10000000, emoji: '🌳' },
      { name: 'Stability', max: Infinity, emoji: '🏔️' }
    ];
    
    const currentPhase = phases[Math.min(phase - 1, phases.length - 1)];
    return `${currentPhase.emoji} ${currentPhase.name} Phase`;
  }

  /**
   * Format comprehensive mining stats
   */
  static formatMiningDashboard(stats: MiningStats): {
    rateDisplay: string;
    dailyDisplay: string;
    totalDisplay: string;
    efficiencyDisplay: ReturnType<typeof this.formatEfficiency>;
    phaseDisplay: string;
    activeTimeDisplay: string;
  } {
    return {
      rateDisplay: TokenFormatter.formatMiningRate(stats.currentRate),
      dailyDisplay: TokenFormatter.formatToken({ amount: stats.dailyMined, symbol: 'FIN' }),
      totalDisplay: TokenFormatter.formatToken({ amount: stats.totalMined, symbol: 'FIN' }),
      efficiencyDisplay: this.formatEfficiency(stats.efficiency),
      phaseDisplay: this.formatPhase(stats.phase, 0),
      activeTimeDisplay: TimeFormatter.formatDuration(stats.timeActive)
    };
  }
}

// ============================================================================
// SOCIAL MEDIA STATS FORMATTING
// ============================================================================

export class SocialStatsFormatter {
  /**
   * Format follower count with abbreviations
   */
  static formatFollowers(count: number): string {
    if (count >= 1e9) return `${(count / 1e9).toFixed(1)}B`;
    if (count >= 1e6) return `${(count / 1e6).toFixed(1)}M`;
    if (count >= 1e3) return `${(count / 1e3).toFixed(1)}K`;
    return count.toLocaleString();
  }

  /**
   * Format engagement rate with quality indicator
   */
  static formatEngagement(rate: number): { 
    percentage: string; 
    quality: string; 
    color: string 
  } {
    const percentage = `${(rate * 100).toFixed(2)}%`;
    let quality: string;
    let color: string;
    
    if (rate >= 0.1) {
      quality = 'Excellent';
      color = '#10B981';
    } else if (rate >= 0.05) {
      quality = 'Good';
      color = '#F59E0B';
    } else if (rate >= 0.02) {
      quality = 'Average';
      color = '#6B7280';
    } else {
      quality = 'Low';
      color = '#EF4444';
    }
    
    return { percentage, quality, color };
  }

  /**
   * Format platform-specific stats
   */
  static formatPlatformStats(stats: SocialStats): {
    platformEmoji: string;
    followersDisplay: string;
    engagementDisplay: ReturnType<typeof this.formatEngagement>;
    postsDisplay: string;
    viewsDisplay: string;
    qualityDisplay: { score: string; stars: string };
  } {
    const platformEmojis: Record<string, string> = {
      instagram: '📷',
      tiktok: '🎵',
      youtube: '📺',
      facebook: '👥',
      twitter: '🐦',
      'x': '❌'
    };
    
    const qualityStars = '⭐'.repeat(Math.min(5, Math.floor(stats.qualityScore * 5)));
    
    return {
      platformEmoji: platformEmojis[stats.platform.toLowerCase()] || '🌐',
      followersDisplay: this.formatFollowers(stats.followers),
      engagementDisplay: this.formatEngagement(stats.engagement),
      postsDisplay: stats.posts.toLocaleString(),
      viewsDisplay: this.formatFollowers(stats.views),
      qualityDisplay: {
        score: `${(stats.qualityScore * 100).toFixed(0)}/100`,
        stars: qualityStars || '⭐'
      }
    };
  }
}

// ============================================================================
// ADDRESS FORMATTING
// ============================================================================

export class AddressFormatter {
  /**
   * Format Solana address with truncation
   */
  static formatAddress(
    address: string | PublicKey, 
    startChars: number = 4, 
    endChars: number = 4
  ): string {
    const addressStr = address.toString();
    
    if (addressStr.length <= startChars + endChars + 3) {
      return addressStr;
    }
    
    return `${addressStr.slice(0, startChars)}...${addressStr.slice(-endChars)}`;
  }

  /**
   * Validate Solana address format
   */
  static isValidAddress(address: string): boolean {
    try {
      new PublicKey(address);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Format with copy-friendly display
   */
  static formatCopyableAddress(address: string): {
    short: string;
    full: string;
    isValid: boolean;
  } {
    return {
      short: this.formatAddress(address),
      full: address,
      isValid: this.isValidAddress(address)
    };
  }
}

// ============================================================================
// QUALITY SCORES FORMATTING
// ============================================================================

export class QualityScoreFormatter {
  /**
   * Format AI quality score with visual representation
   */
  static formatQualityScore(score: number): {
    percentage: string;
    grade: string;
    color: string;
    bars: string;
    multiplier: string;
  } {
    const percentage = `${(score * 100).toFixed(0)}%`;
    let grade: string;
    let color: string;
    
    if (score >= 0.9) {
      grade = 'A+';
      color = '#10B981';
    } else if (score >= 0.8) {
      grade = 'A';
      color = '#34D399';
    } else if (score >= 0.7) {
      grade = 'B+';
      color = '#FBBF24';
    } else if (score >= 0.6) {
      grade = 'B';
      color = '#F59E0B';
    } else if (score >= 0.5) {
      grade = 'C';
      color = '#EF4444';
    } else {
      grade = 'F';
      color = '#7F1D1D';
    }
    
    const filledBars = Math.floor(score * 10);
    const bars = '█'.repeat(filledBars) + '░'.repeat(10 - filledBars);
    const multiplier = `${score.toFixed(2)}x`;
    
    return { percentage, grade, color, bars, multiplier };
  }

  /**
   * Format originality score
   */
  static formatOriginality(score: number): {
    label: string;
    emoji: string;
    color: string;
  } {
    if (score >= 0.9) return { label: 'Highly Original', emoji: '💎', color: '#10B981' };
    if (score >= 0.7) return { label: 'Original', emoji: '✨', color: '#F59E0B' };
    if (score >= 0.5) return { label: 'Somewhat Original', emoji: '📝', color: '#6B7280' };
    return { label: 'Low Originality', emoji: '⚠️', color: '#EF4444' };
  }
}

// ============================================================================
// ACHIEVEMENTS FORMATTING
// ============================================================================

export class AchievementFormatter {
  /**
   * Format achievement with rarity styling
   */
  static formatAchievement(achievement: Achievement): {
    title: string;
    description: string;
    progress: string;
    rarityEmoji: string;
    rarityColor: string;
    unlockStatus: string;
  } {
    const rarityConfig = {
      common: { emoji: '🥉', color: '#CD7F32' },
      uncommon: { emoji: '🥈', color: '#C0C0C0' },
      rare: { emoji: '🥇', color: '#FFD700' },
      epic: { emoji: '💎', color: '#8B5CF6' },
      legendary: { emoji: '👑', color: '#FF6B9D' }
    };
    
    const config = rarityConfig[achievement.rarity];
    const progressPercentage = Math.min(100, (achievement.progress / achievement.maxProgress) * 100);
    const progress = `${achievement.progress}/${achievement.maxProgress} (${progressPercentage.toFixed(0)}%)`;
    
    const unlockStatus = achievement.unlocked 
      ? `🔓 Unlocked ${achievement.unlockedAt ? TimeFormatter.formatRelativeTime(achievement.unlockedAt) : ''}`
      : '🔒 Locked';
    
    return {
      title: achievement.title,
      description: achievement.description,
      progress,
      rarityEmoji: config.emoji,
      rarityColor: config.color,
      unlockStatus
    };
  }

  /**
   * Format achievement progress bar
   */
  static formatProgressBar(
    current: number, 
    max: number, 
    width: number = 20
  ): string {
    const percentage = Math.min(1, current / max);
    const filledWidth = Math.floor(percentage * width);
    const emptyWidth = width - filledWidth;
    
    return '█'.repeat(filledWidth) + '░'.repeat(emptyWidth);
  }

  /**
   * Format achievement collection stats
   */
  static formatCollectionStats(achievements: Achievement[]): {
    total: number;
    unlocked: number;
    percentage: string;
    rarityBreakdown: Record<string, { count: number; unlocked: number }>;
  } {
    const total = achievements.length;
    const unlocked = achievements.filter(a => a.unlocked).length;
    const percentage = total > 0 ? `${((unlocked / total) * 100).toFixed(1)}%` : '0%';
    
    const rarityBreakdown = achievements.reduce((acc, achievement) => {
      if (!acc[achievement.rarity]) {
        acc[achievement.rarity] = { count: 0, unlocked: 0 };
      }
      acc[achievement.rarity].count++;
      if (achievement.unlocked) {
        acc[achievement.rarity].unlocked++;
      }
      return acc;
    }, {} as Record<string, { count: number; unlocked: number }>);
    
    return { total, unlocked, percentage, rarityBreakdown };
  }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

export class UtilityFormatter {
  /**
   * Format large numbers with appropriate abbreviations
   */
  static formatLargeNumber(num: number, precision: number = 1): string {
    if (num >= 1e12) return `${(num / 1e12).toFixed(precision)}T`;
    if (num >= 1e9) return `${(num / 1e9).toFixed(precision)}B`;
    if (num >= 1e6) return `${(num / 1e6).toFixed(precision)}M`;
    if (num >= 1e3) return `${(num / 1e3).toFixed(precision)}K`;
    return num.toLocaleString();
  }

  /**
   * Format decimal precision based on value
   */
  static formatSmartPrecision(value: number): string {
    if (value === 0) return '0';
    if (value >= 1000) return value.toFixed(0);
    if (value >= 100) return value.toFixed(1);
    if (value >= 10) return value.toFixed(2);
    if (value >= 1) return value.toFixed(3);
    if (value >= 0.01) return value.toFixed(4);
    if (value >= 0.0001) return value.toFixed(6);
    return value.toExponential(2);
  }

  /**
   * Sanitize user input for display
   */
  static sanitizeDisplay(input: string, maxLength: number = 100): string {
    return input
      .replace(/[<>]/g, '') // Remove potential HTML
      .trim()
      .substring(0, maxLength)
      + (input.length > maxLength ? '...' : '');
  }

  /**
   * Format file size
   */
  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`;
  }
}

// ============================================================================
// EXPORT ALL FORMATTERS
// ============================================================================

export default {
  TokenFormatter,
  TimeFormatter,
  UserStatsFormatter,
  MiningStatsFormatter,
  SocialStatsFormatter,
  AddressFormatter,
  QualityScoreFormatter,
  AchievementFormatter,
  UtilityFormatter
};
