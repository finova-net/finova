/**
 * Finova Network API - Formatting Utilities
 * Enterprise-grade formatting functions for consistent data presentation
 * 
 * @version 1.0.0
 * @author Finova Network Development Team
 * @license MIT
 */

import { BigNumber } from 'bignumber.js';
import { format, parseISO, formatDistanceToNow, differenceInDays } from 'date-fns';
import { id, enUS } from 'date-fns/locale';

// Constants for Finova ecosystem
const FINOVA_CONSTANTS = {
  TOKEN_DECIMALS: 9,
  MAX_SUPPLY: new BigNumber('100000000000'), // 100B tokens
  PHASES: {
    FINIZEN: { min: 0, max: 100000, name: 'Finizen' },
    GROWTH: { min: 100000, max: 1000000, name: 'Growth' },
    MATURITY: { min: 1000000, max: 10000000, name: 'Maturity' },
    STABILITY: { min: 10000000, max: Infinity, name: 'Stability' }
  },
  XP_LEVELS: {
    BRONZE: { min: 0, max: 999, tier: 'Bronze' },
    SILVER: { min: 1000, max: 4999, tier: 'Silver' },
    GOLD: { min: 5000, max: 19999, tier: 'Gold' },
    PLATINUM: { min: 20000, max: 49999, tier: 'Platinum' },
    DIAMOND: { min: 50000, max: 99999, tier: 'Diamond' },
    MYTHIC: { min: 100000, max: Infinity, tier: 'Mythic' }
  },
  RP_TIERS: {
    EXPLORER: { min: 0, max: 999, name: 'Explorer' },
    CONNECTOR: { min: 1000, max: 4999, name: 'Connector' },
    INFLUENCER: { min: 5000, max: 14999, name: 'Influencer' },
    LEADER: { min: 15000, max: 49999, name: 'Leader' },
    AMBASSADOR: { min: 50000, max: Infinity, name: 'Ambassador' }
  }
} as const;

// Types
interface FormatNumberOptions {
  decimals?: number;
  showSymbol?: boolean;
  compact?: boolean;
  locale?: 'id' | 'en';
}

interface FormatCurrencyOptions extends FormatNumberOptions {
  currency?: 'IDR' | 'USD';
  hideZeros?: boolean;
}

interface UserStats {
  xp: number;
  rp: number;
  finBalance: number;
  level: number;
  tier: string;
}

/**
 * Format $FIN token amounts with proper decimals and symbols
 */
export const formatFIN = (
  amount: string | number | BigNumber,
  options: FormatNumberOptions = {}
): string => {
  const {
    decimals = 4,
    showSymbol = true,
    compact = false,
    locale = 'en'
  } = options;

  try {
    const num = new BigNumber(amount || 0);
    
    if (num.isNaN() || !num.isFinite()) {
      return showSymbol ? '0.0000 $FIN' : '0.0000';
    }

    let formatted: string;

    if (compact && num.gte(1000000)) {
      const millions = num.div(1000000);
      formatted = `${millions.toFixed(2)}M`;
    } else if (compact && num.gte(1000)) {
      const thousands = num.div(1000);
      formatted = `${thousands.toFixed(2)}K`;
    } else {
      formatted = num.toFixed(decimals);
    }

    // Add thousand separators based on locale
    const parts = formatted.split('.');
    parts[0] = parts[0].replace(/\B(?=(\d{3})+(?!\d))/g, locale === 'id' ? '.' : ',');
    formatted = parts.join(locale === 'id' ? ',' : '.');

    return showSymbol ? `${formatted} $FIN` : formatted;
  } catch (error) {
    console.error('Error formatting FIN amount:', error);
    return showSymbol ? '0.0000 $FIN' : '0.0000';
  }
};

/**
 * Format fiat currency (IDR/USD) with proper locale support
 */
export const formatCurrency = (
  amount: string | number,
  options: FormatCurrencyOptions = {}
): string => {
  const {
    currency = 'IDR',
    decimals = currency === 'IDR' ? 0 : 2,
    locale = 'id',
    hideZeros = true
  } = options;

  try {
    const num = new BigNumber(amount || 0);
    
    if (num.isNaN() || !num.isFinite()) {
      return currency === 'IDR' ? 'Rp 0' : '$0.00';
    }

    if (hideZeros && num.isZero()) {
      return currency === 'IDR' ? 'Rp 0' : '$0';
    }

    const localeCode = locale === 'id' ? 'id-ID' : 'en-US';
    const currencyCode = currency;

    return new Intl.NumberFormat(localeCode, {
      style: 'currency',
      currency: currencyCode,
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals
    }).format(num.toNumber());
  } catch (error) {
    console.error('Error formatting currency:', error);
    return currency === 'IDR' ? 'Rp 0' : '$0.00';
  }
};

/**
 * Format XP with level badges and progression
 */
export const formatXP = (xp: number, showBadge = true): string => {
  try {
    const level = calculateXPLevel(xp);
    const tier = getXPTier(xp);
    const formatted = new Intl.NumberFormat('en-US').format(xp);
    
    if (showBadge) {
      const badge = getXPBadgeEmoji(tier.tier);
      return `${badge} ${formatted} XP (Level ${level})`;
    }
    
    return `${formatted} XP`;
  } catch (error) {
    console.error('Error formatting XP:', error);
    return '0 XP';
  }
};

/**
 * Format RP with tier information
 */
export const formatRP = (rp: number, showTier = true): string => {
  try {
    const tier = getRPTier(rp);
    const formatted = new Intl.NumberFormat('en-US').format(rp);
    
    if (showTier) {
      const emoji = getRPTierEmoji(tier.name);
      return `${emoji} ${formatted} RP (${tier.name})`;
    }
    
    return `${formatted} RP`;
  } catch (error) {
    console.error('Error formatting RP:', error);
    return '0 RP';
  }
};

/**
 * Format mining rate with proper units
 */
export const formatMiningRate = (rate: number, unit: 'hour' | 'day' = 'hour'): string => {
  try {
    const multiplier = unit === 'day' ? 24 : 1;
    const amount = new BigNumber(rate).times(multiplier);
    
    return `${formatFIN(amount)} per ${unit}`;
  } catch (error) {
    console.error('Error formatting mining rate:', error);
    return `0.0000 $FIN per ${unit}`;
  }
};

/**
 * Format percentage with proper decimals
 */
export const formatPercentage = (
  value: number,
  decimals = 2,
  showPlus = false
): string => {
  try {
    const num = new BigNumber(value || 0);
    
    if (num.isNaN() || !num.isFinite()) {
      return '0%';
    }

    const formatted = num.toFixed(decimals);
    const sign = showPlus && num.gt(0) ? '+' : '';
    
    return `${sign}${formatted}%`;
  } catch (error) {
    console.error('Error formatting percentage:', error);
    return '0%';
  }
};

/**
 * Format dates with Indonesian locale support
 */
export const formatDate = (
  date: string | Date,
  formatStr = 'PPP',
  locale: 'id' | 'en' = 'en'
): string => {
  try {
    const dateObj = typeof date === 'string' ? parseISO(date) : date;
    const localeObj = locale === 'id' ? id : enUS;
    
    return format(dateObj, formatStr, { locale: localeObj });
  } catch (error) {
    console.error('Error formatting date:', error);
    return 'Invalid date';
  }
};

/**
 * Format time distance (e.g., "2 hours ago")
 */
export const formatTimeDistance = (
  date: string | Date,
  locale: 'id' | 'en' = 'en'
): string => {
  try {
    const dateObj = typeof date === 'string' ? parseISO(date) : date;
    const localeObj = locale === 'id' ? id : enUS;
    
    return formatDistanceToNow(dateObj, { addSuffix: true, locale: localeObj });
  } catch (error) {
    console.error('Error formatting time distance:', error);
    return 'Unknown time';
  }
};

/**
 * Format duration in human-readable format
 */
export const formatDuration = (seconds: number): string => {
  try {
    const days = Math.floor(seconds / 86400);
    const hours = Math.floor((seconds % 86400) / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remainingSeconds = seconds % 60;

    const parts: string[] = [];
    
    if (days > 0) parts.push(`${days}d`);
    if (hours > 0) parts.push(`${hours}h`);
    if (minutes > 0) parts.push(`${minutes}m`);
    if (remainingSeconds > 0 || parts.length === 0) parts.push(`${remainingSeconds}s`);

    return parts.slice(0, 2).join(' ');
  } catch (error) {
    console.error('Error formatting duration:', error);
    return '0s';
  }
};

/**
 * Format wallet address with ellipsis
 */
export const formatWalletAddress = (
  address: string,
  startChars = 6,
  endChars = 4
): string => {
  try {
    if (!address || address.length <= startChars + endChars) {
      return address || '';
    }

    return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
  } catch (error) {
    console.error('Error formatting wallet address:', error);
    return '';
  }
};

/**
 * Format transaction hash
 */
export const formatTxHash = (hash: string): string => {
  return formatWalletAddress(hash, 8, 8);
};

/**
 * Format network phase based on user count
 */
export const formatNetworkPhase = (userCount: number): string => {
  try {
    for (const [, phase] of Object.entries(FINOVA_CONSTANTS.PHASES)) {
      if (userCount >= phase.min && userCount < phase.max) {
        return `${phase.name} Phase (${new Intl.NumberFormat().format(userCount)} users)`;
      }
    }
    return `Unknown Phase (${new Intl.NumberFormat().format(userCount)} users)`;
  } catch (error) {
    console.error('Error formatting network phase:', error);
    return 'Unknown Phase';
  }
};

/**
 * Format user stats summary
 */
export const formatUserStats = (stats: UserStats): Record<string, string> => {
  try {
    return {
      finBalance: formatFIN(stats.finBalance),
      xp: formatXP(stats.xp),
      rp: formatRP(stats.rp),
      level: `Level ${stats.level}`,
      tier: stats.tier,
      combined: `Level ${stats.level} • ${formatFIN(stats.finBalance, { showSymbol: false })} $FIN`
    };
  } catch (error) {
    console.error('Error formatting user stats:', error);
    return {
      finBalance: '0 $FIN',
      xp: '0 XP',
      rp: '0 RP',
      level: 'Level 1',
      tier: 'Bronze I',
      combined: 'Level 1 • 0 $FIN'
    };
  }
};

/**
 * Format large numbers in compact form
 */
export const formatCompactNumber = (num: number): string => {
  try {
    const absNum = Math.abs(num);
    
    if (absNum >= 1e9) {
      return `${(num / 1e9).toFixed(1)}B`;
    } else if (absNum >= 1e6) {
      return `${(num / 1e6).toFixed(1)}M`;
    } else if (absNum >= 1e3) {
      return `${(num / 1e3).toFixed(1)}K`;
    } else {
      return num.toString();
    }
  } catch (error) {
    console.error('Error formatting compact number:', error);
    return '0';
  }
};

// Helper functions
function calculateXPLevel(xp: number): number {
  // Exponential progression: level = floor(sqrt(xp / 100))
  return Math.floor(Math.sqrt(xp / 100)) + 1;
}

function getXPTier(xp: number) {
  for (const [, tier] of Object.entries(FINOVA_CONSTANTS.XP_LEVELS)) {
    if (xp >= tier.min && xp <= tier.max) {
      const subLevel = Math.floor((xp - tier.min) / ((tier.max - tier.min + 1) / 10)) + 1;
      return { ...tier, subLevel: Math.min(subLevel, 10) };
    }
  }
  return FINOVA_CONSTANTS.XP_LEVELS.MYTHIC;
}

function getRPTier(rp: number) {
  for (const [, tier] of Object.entries(FINOVA_CONSTANTS.RP_TIERS)) {
    if (rp >= tier.min && rp <= tier.max) {
      return tier;
    }
  }
  return FINOVA_CONSTANTS.RP_TIERS.AMBASSADOR;
}

function getXPBadgeEmoji(tier: string): string {
  const badges = {
    Bronze: '🥉',
    Silver: '🥈',
    Gold: '🥇',
    Platinum: '💎',
    Diamond: '💠',
    Mythic: '👑'
  };
  return badges[tier as keyof typeof badges] || '🏆';
}

function getRPTierEmoji(tier: string): string {
  const emojis = {
    Explorer: '🔍',
    Connector: '🔗',
    Influencer: '📢',
    Leader: '👑',
    Ambassador: '🌟'
  };
  return emojis[tier as keyof typeof emojis] || '🏆';
}

/**
 * Sanitize and format user input for display
 */
export const sanitizeUserInput = (input: string, maxLength = 280): string => {
  try {
    if (!input || typeof input !== 'string') {
      return '';
    }

    // Remove potentially harmful characters and scripts
    const sanitized = input
      .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, '')
      .replace(/javascript:/gi, '')
      .replace(/on\w+\s*=/gi, '')
      .replace(/[<>'"&]/g, (match) => {
        const entities: Record<string, string> = {
          '<': '&lt;',
          '>': '&gt;',
          '"': '&quot;',
          "'": '&#39;',
          '&': '&amp;'
        };
        return entities[match] || match;
      });

    // Truncate if too long
    return sanitized.length > maxLength 
      ? `${sanitized.slice(0, maxLength - 3)}...` 
      : sanitized;
  } catch (error) {
    console.error('Error sanitizing user input:', error);
    return '';
  }
};

/**
 * Format API response data with consistent structure
 */
export const formatApiResponse = <T>(
  data: T,
  message = 'Success',
  meta: Record<string, any> = {}
) => {
  return {
    success: true,
    message,
    data,
    meta: {
      timestamp: new Date().toISOString(),
      ...meta
    }
  };
};

/**
 * Format API error response
 */
export const formatApiError = (
  error: string | Error,
  code = 'INTERNAL_ERROR',
  statusCode = 500
) => {
  const message = error instanceof Error ? error.message : error;
  
  return {
    success: false,
    error: {
      code,
      message,
      statusCode,
      timestamp: new Date().toISOString()
    }
  };
};

export default {
  formatFIN,
  formatCurrency,
  formatXP,
  formatRP,
  formatMiningRate,
  formatPercentage,
  formatDate,
  formatTimeDistance,
  formatDuration,
  formatWalletAddress,
  formatTxHash,
  formatNetworkPhase,
  formatUserStats,
  formatCompactNumber,
  sanitizeUserInput,
  formatApiResponse,
  formatApiError
};
