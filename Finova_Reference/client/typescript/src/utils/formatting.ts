// finova-net/finova/client/typescript/src/utils/formatting.ts

/**
 * Finova Network - Formatting Utilities
 * Enterprise-grade formatting functions for tokens, XP, RP, and display values
 * 
 * @version 3.0.0
 * @author Finova Network Team
 * @license MIT
 */

import { BN } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';

// ============================================================================
// CONSTANTS
// ============================================================================

export const LAMPORTS_PER_SOL = 1_000_000_000;
export const DECIMALS = {
  FIN: 9,
  sFIN: 9,
  USDfin: 6,
  sUSDfin: 6,
  XP: 0,
  RP: 0,
} as const;

export const DISPLAY_DECIMALS = {
  FIN: 4,
  sFIN: 4,
  USDfin: 2,
  sUSDfin: 2,
  XP: 0,
  RP: 0,
} as const;

// ============================================================================
// TYPES
// ============================================================================

export type TokenType = keyof typeof DECIMALS;
export type NumberFormatStyle = 'decimal' | 'currency' | 'percent' | 'compact';

export interface FormatOptions {
  decimals?: number;
  style?: NumberFormatStyle;
  currency?: string;
  locale?: string;
  showSymbol?: boolean;
  compact?: boolean;
  prefix?: string;
  suffix?: string;
}

export interface UserStats {
  totalFin: number;
  stakedFin: number;
  xpPoints: number;
  rpPoints: number;
  level: number;
  tier: string;
  miningRate: number;
  networkSize: number;
}

// ============================================================================
// TOKEN FORMATTING
// ============================================================================

/**
 * Format token amounts with proper decimals and display
 */
export function formatTokenAmount(
  amount: number | string | BN,
  tokenType: TokenType,
  options: FormatOptions = {}
): string {
  const {
    decimals = DISPLAY_DECIMALS[tokenType],
    style = 'decimal',
    currency = 'USD',
    locale = 'en-US',
    showSymbol = true,
    compact = false,
    prefix = '',
    suffix = '',
  } = options;

  let numValue: number;

  // Convert different input types to number
  if (amount instanceof BN) {
    numValue = amount.toNumber() / Math.pow(10, DECIMALS[tokenType]);
  } else if (typeof amount === 'string') {
    numValue = parseFloat(amount);
  } else {
    numValue = amount;
  }

  // Handle zero or invalid values
  if (!numValue || isNaN(numValue)) {
    return showSymbol ? `0 ${tokenType}` : '0';
  }

  // Format based on style
  let formatted: string;
  
  if (compact && numValue >= 1000) {
    formatted = formatCompactNumber(numValue, decimals);
  } else {
    const formatter = new Intl.NumberFormat(locale, {
      style,
      currency: style === 'currency' ? currency : undefined,
      minimumFractionDigits: Math.min(decimals, 2),
      maximumFractionDigits: decimals,
    });
    
    formatted = formatter.format(numValue);
  }

  // Add token symbol if requested
  if (showSymbol && style !== 'currency') {
    formatted += ` ${tokenType}`;
  }

  return `${prefix}${formatted}${suffix}`;
}

/**
 * Format compact numbers (1K, 1M, 1B, etc.)
 */
export function formatCompactNumber(value: number, decimals: number = 2): string {
  const units = ['', 'K', 'M', 'B', 'T'];
  let unitIndex = 0;
  let compactValue = value;

  while (compactValue >= 1000 && unitIndex < units.length - 1) {
    compactValue /= 1000;
    unitIndex++;
  }

  return `${compactValue.toFixed(decimals).replace(/\.?0+$/, '')}${units[unitIndex]}`;
}

// ============================================================================
// XP & RP FORMATTING
// ============================================================================

/**
 * Format XP with level progression
 */
export function formatXP(
  currentXP: number,
  nextLevelXP?: number,
  options: FormatOptions = {}
): string {
  const { compact = false, showProgress = false } = options as FormatOptions & { showProgress?: boolean };
  
  const formatted = compact ? formatCompactNumber(currentXP, 0) : currentXP.toLocaleString();
  
  if (showProgress && nextLevelXP) {
    const progress = ((currentXP % nextLevelXP) / nextLevelXP * 100).toFixed(1);
    return `${formatted} XP (${progress}% to next level)`;
  }
  
  return `${formatted} XP`;
}

/**
 * Format RP with tier information
 */
export function formatRP(
  currentRP: number,
  tier?: string,
  options: FormatOptions = {}
): string {
  const { compact = false, showTier = true } = options as FormatOptions & { showTier?: boolean };
  
  const formatted = compact ? formatCompactNumber(currentRP, 0) : currentRP.toLocaleString();
  
  if (showTier && tier) {
    return `${formatted} RP (${tier})`;
  }
  
  return `${formatted} RP`;
}

// ============================================================================
// MINING & REWARDS FORMATTING
// ============================================================================

/**
 * Format mining rate per hour/day
 */
export function formatMiningRate(
  ratePerHour: number,
  period: 'hour' | 'day' = 'hour',
  options: FormatOptions = {}
): string {
  const rate = period === 'day' ? ratePerHour * 24 : ratePerHour;
  const periodText = period === 'day' ? '/day' : '/hour';
  
  return `${formatTokenAmount(rate, 'FIN', { ...options, decimals: 4 })}${periodText}`;
}

/**
 * Format staking APY percentage
 */
export function formatAPY(apy: number, options: FormatOptions = {}): string {
  const { decimals = 2, locale = 'en-US' } = options;
  
  const formatter = new Intl.NumberFormat(locale, {
    style: 'percent',
    minimumFractionDigits: decimals,
    maximumFractionDigits: decimals,
  });
  
  return formatter.format(apy / 100);
}

/**
 * Format reward multipliers
 */
export function formatMultiplier(multiplier: number, options: FormatOptions = {}): string {
  const { decimals = 2, prefix = '', suffix = 'x' } = options;
  
  return `${prefix}${multiplier.toFixed(decimals)}${suffix}`;
}

// ============================================================================
// ADDRESS & HASH FORMATTING
// ============================================================================

/**
 * Format Solana public key with ellipsis
 */
export function formatAddress(
  address: string | PublicKey,
  startChars: number = 4,
  endChars: number = 4
): string {
  const addressStr = typeof address === 'string' ? address : address.toString();
  
  if (addressStr.length <= startChars + endChars) {
    return addressStr;
  }
  
  return `${addressStr.slice(0, startChars)}...${addressStr.slice(-endChars)}`;
}

/**
 * Format transaction signature
 */
export function formatTxSignature(signature: string, length: number = 8): string {
  return `${signature.slice(0, length)}...${signature.slice(-length)}`;
}

// ============================================================================
// TIME & DATE FORMATTING
// ============================================================================

/**
 * Format time duration (seconds to human readable)
 */
export function formatDuration(seconds: number, short: boolean = false): string {
  const units = short 
    ? [
        { label: 'y', seconds: 31536000 },
        { label: 'mo', seconds: 2592000 },
        { label: 'd', seconds: 86400 },
        { label: 'h', seconds: 3600 },
        { label: 'm', seconds: 60 },
        { label: 's', seconds: 1 },
      ]
    : [
        { label: 'year', seconds: 31536000 },
        { label: 'month', seconds: 2592000 },
        { label: 'day', seconds: 86400 },
        { label: 'hour', seconds: 3600 },
        { label: 'minute', seconds: 60 },
        { label: 'second', seconds: 1 },
      ];

  for (const unit of units) {
    const value = Math.floor(seconds / unit.seconds);
    if (value > 0) {
      const plural = !short && value !== 1 ? 's' : '';
      return `${value} ${unit.label}${plural}`;
    }
  }
  
  return short ? '0s' : '0 seconds';
}

/**
 * Format relative time (time ago)
 */
export function formatTimeAgo(timestamp: number | Date, locale: string = 'en-US'): string {
  const now = new Date();
  const date = typeof timestamp === 'number' ? new Date(timestamp) : timestamp;
  const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);
  
  if (diffInSeconds < 60) return 'just now';
  if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`;
  if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}h ago`;
  if (diffInSeconds < 2592000) return `${Math.floor(diffInSeconds / 86400)}d ago`;
  
  return new Intl.DateTimeFormat(locale, {
    year: 'numeric',
    month: 'short',
    day: 'numeric'
  }).format(date);
}

// ============================================================================
// LEVEL & PROGRESSION FORMATTING
// ============================================================================

/**
 * Calculate and format user level from XP
 */
export function calculateLevel(xp: number): { level: number; progress: number; nextLevelXP: number } {
  // Level formula: level = floor(sqrt(XP / 100))
  const level = Math.floor(Math.sqrt(xp / 100)) + 1;
  const currentLevelXP = (level - 1) ** 2 * 100;
  const nextLevelXP = level ** 2 * 100;
  const progress = (xp - currentLevelXP) / (nextLevelXP - currentLevelXP);
  
  return {
    level,
    progress: Math.max(0, Math.min(1, progress)),
    nextLevelXP: nextLevelXP - xp,
  };
}

/**
 * Get tier name from RP points
 */
export function getRPTier(rp: number): string {
  if (rp >= 50000) return 'Ambassador';
  if (rp >= 15000) return 'Leader';
  if (rp >= 5000) return 'Influencer';
  if (rp >= 1000) return 'Connector';
  return 'Explorer';
}

/**
 * Get badge tier from level
 */
export function getBadgeTier(level: number): string {
  if (level >= 101) return 'Mythic';
  if (level >= 76) return 'Diamond';
  if (level >= 51) return 'Platinum';
  if (level >= 26) return 'Gold';
  if (level >= 11) return 'Silver';
  return 'Bronze';
}

// ============================================================================
// COMPREHENSIVE USER STATS FORMATTING
// ============================================================================

/**
 * Format complete user statistics display
 */
export function formatUserStats(stats: UserStats): {
  tokens: string;
  staking: string;
  experience: string;
  referrals: string;
  mining: string;
  level: string;
} {
  const levelInfo = calculateLevel(stats.xpPoints);
  const rpTier = getRPTier(stats.rpPoints);
  const badgeTier = getBadgeTier(levelInfo.level);
  
  return {
    tokens: `${formatTokenAmount(stats.totalFin, 'FIN', { compact: true })} | ${formatTokenAmount(stats.stakedFin, 'sFIN', { compact: true })}`,
    staking: `${formatTokenAmount(stats.stakedFin, 'sFIN')} (${formatAPY(12)})`,
    experience: `${formatXP(stats.xpPoints, undefined, { compact: true })} | Level ${levelInfo.level} (${badgeTier})`,
    referrals: `${formatRP(stats.rpPoints, rpTier, { compact: true })} | Network: ${formatCompactNumber(stats.networkSize, 0)}`,
    mining: `${formatMiningRate(stats.miningRate)} | Total: ${formatTokenAmount(stats.totalFin, 'FIN', { compact: true })}`,
    level: `${levelInfo.level} | ${(levelInfo.progress * 100).toFixed(1)}% to ${levelInfo.level + 1}`,
  };
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/**
 * Validate and sanitize numeric input
 */
export function sanitizeNumber(input: string | number): number {
  if (typeof input === 'number') {
    return isFinite(input) ? input : 0;
  }
  
  const cleaned = input.replace(/[^0-9.-]/g, '');
  const parsed = parseFloat(cleaned);
  
  return isFinite(parsed) ? parsed : 0;
}

/**
 * Format input for display while typing
 */
export function formatInputNumber(
  value: string,
  tokenType?: TokenType,
  maxDecimals?: number
): string {
  if (!value) return '';
  
  const decimals = maxDecimals || (tokenType ? DISPLAY_DECIMALS[tokenType] : 2);
  const regex = new RegExp(`^\\d*\\.?\\d{0,${decimals}}$`);
  
  if (regex.test(value)) {
    return value;
  }
  
  // Truncate excess decimals
  const parts = value.split('.');
  if (parts.length === 2) {
    return `${parts[0]}.${parts[1].slice(0, decimals)}`;
  }
  
  return parts[0] || '';
}

// ============================================================================
// EXPORT DEFAULT OBJECT
// ============================================================================

export default {
  // Token formatting
  formatTokenAmount,
  formatCompactNumber,
  
  // XP & RP formatting
  formatXP,
  formatRP,
  
  // Mining & rewards
  formatMiningRate,
  formatAPY,
  formatMultiplier,
  
  // Address formatting
  formatAddress,
  formatTxSignature,
  
  // Time formatting
  formatDuration,
  formatTimeAgo,
  
  // Level & progression
  calculateLevel,
  getRPTier,
  getBadgeTier,
  
  // Comprehensive formatting
  formatUserStats,
  
  // Validation
  sanitizeNumber,
  formatInputNumber,
  
  // Constants
  DECIMALS,
  DISPLAY_DECIMALS,
};
