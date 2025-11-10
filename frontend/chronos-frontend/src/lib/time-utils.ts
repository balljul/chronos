// Time utilities for Chronos Time Tracking

/**
 * Format duration in seconds to human readable format
 * Examples: "2h 30m", "45m", "3h 15m", "0m"
 */
export function formatDuration(seconds: number): string {
  if (seconds <= 0) return "0m";

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);

  if (hours > 0 && minutes > 0) {
    return `${hours}h ${minutes}m`;
  } else if (hours > 0) {
    return `${hours}h`;
  } else {
    return `${minutes}m`;
  }
}

/**
 * Format duration for timer display (HH:MM:SS)
 */
export function formatTimerDuration(seconds: number): string {
  // Handle invalid or negative values
  if (isNaN(seconds) || seconds < 0) {
    return "00:00:00";
  }

  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);

  return `${hours.toString().padStart(2, "0")}:${minutes
    .toString()
    .padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

/**
 * Calculate duration between two dates in seconds
 */
export function calculateDuration(startTime: Date, endTime: Date): number {
  return Math.floor((endTime.getTime() - startTime.getTime()) / 1000);
}

/**
 * Calculate elapsed time since start in seconds
 */
export function calculateElapsedTime(startTime: Date): number {
  return Math.floor((Date.now() - startTime.getTime()) / 1000);
}

/**
 * Format date for display (e.g., "Dec 15, 2023")
 */
export function formatDate(dateString: string): string {
  if (!dateString) return "Unknown";

  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return "Invalid date";

  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

/**
 * Format time for display (e.g., "2:30 PM")
 */
export function formatTime(dateString: string): string {
  if (!dateString) return "Unknown";

  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return "Invalid time";

  return date.toLocaleTimeString("en-US", {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });
}

/**
 * Format full date and time for display (e.g., "Dec 15, 2023 at 2:30 PM")
 */
export function formatDateTime(dateString: string): string {
  if (!dateString) return "Unknown";

  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return "Invalid date";

  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });
}

/**
 * Get start of day as ISO string
 */
export function getStartOfDay(date: Date = new Date()): string {
  const start = new Date(date);
  start.setHours(0, 0, 0, 0);
  return start.toISOString();
}

/**
 * Get end of day as ISO string
 */
export function getEndOfDay(date: Date = new Date()): string {
  const end = new Date(date);
  end.setHours(23, 59, 59, 999);
  return end.toISOString();
}

/**
 * Get start of week as ISO string (Monday)
 */
export function getStartOfWeek(date: Date = new Date()): string {
  const start = new Date(date);
  const day = start.getDay();
  const diff = start.getDate() - day + (day === 0 ? -6 : 1); // Adjust for Monday start
  start.setDate(diff);
  start.setHours(0, 0, 0, 0);
  return start.toISOString();
}

/**
 * Get end of week as ISO string (Sunday)
 */
export function getEndOfWeek(date: Date = new Date()): string {
  const end = new Date(date);
  const day = end.getDay();
  const diff = end.getDate() - day + (day === 0 ? 0 : 7); // Adjust for Sunday end
  end.setDate(diff);
  end.setHours(23, 59, 59, 999);
  return end.toISOString();
}

/**
 * Get start of month as ISO string
 */
export function getStartOfMonth(date: Date = new Date()): string {
  const start = new Date(date.getFullYear(), date.getMonth(), 1);
  start.setHours(0, 0, 0, 0);
  return start.toISOString();
}

/**
 * Get end of month as ISO string
 */
export function getEndOfMonth(date: Date = new Date()): string {
  const end = new Date(date.getFullYear(), date.getMonth() + 1, 0);
  end.setHours(23, 59, 59, 999);
  return end.toISOString();
}

/**
 * Check if a date string represents today
 */
export function isToday(dateString: string): boolean {
  const date = new Date(dateString);
  const today = new Date();

  return (
    date.getDate() === today.getDate() &&
    date.getMonth() === today.getMonth() &&
    date.getFullYear() === today.getFullYear()
  );
}

/**
 * Parse ISO date string to Date object with error handling
 */
export function parseDate(dateString: string): Date | null {
  if (!dateString) return null;

  const date = new Date(dateString);
  if (Number.isNaN(date.getTime())) return null;

  return date;
}

/**
 * Convert Date to ISO string for API
 */
export function toISOString(date: Date | null): string | undefined {
  if (!date) return undefined;
  return date.toISOString();
}
