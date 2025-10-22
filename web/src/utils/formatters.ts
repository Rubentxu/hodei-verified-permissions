/**
 * Formatters - Utility functions for formatting data
 */

import { format, formatDistanceToNow } from 'date-fns';
import { es } from 'date-fns/locale';

/**
 * Format date to readable string
 */
export const formatDate = (date: string | Date): string => {
  try {
    const dateObj = typeof date === 'string' ? new Date(date) : date;
    return format(dateObj, 'dd/MM/yyyy HH:mm', { locale: es });
  } catch {
    return 'Invalid date';
  }
};

/**
 * Format date to relative time (e.g., "hace 2 horas")
 */
export const formatRelativeTime = (date: string | Date): string => {
  try {
    const dateObj = typeof date === 'string' ? new Date(date) : date;
    return formatDistanceToNow(dateObj, { addSuffix: true, locale: es });
  } catch {
    return 'Invalid date';
  }
};

/**
 * Format JSON with indentation
 */
export const formatJSON = (obj: unknown, indent: number = 2): string => {
  try {
    return JSON.stringify(obj, null, indent);
  } catch {
    return 'Invalid JSON';
  }
};

/**
 * Truncate string to max length
 */
export const truncate = (str: string, maxLength: number): string => {
  if (str.length <= maxLength) return str;
  return `${str.slice(0, maxLength)}...`;
};

/**
 * Format bytes to human readable size
 */
export const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
};

/**
 * Format policy statement for display
 */
export const formatPolicyStatement = (statement: string): string => {
  return statement.replace(/\n\s+/g, ' ').trim();
};

/**
 * Highlight Cedar keywords in code
 */
export const highlightCedarKeywords = (code: string): string => {
  const keywords = ['permit', 'forbid', 'when', 'unless', 'in', 'has', 'like'];
  let highlighted = code;

  keywords.forEach((keyword) => {
    const regex = new RegExp(`\\b${keyword}\\b`, 'g');
    highlighted = highlighted.replace(
      regex,
      `<span class="keyword">${keyword}</span>`
    );
  });

  return highlighted;
};
