// Time Entry Types for Chronos Time Tracking

export interface TimeEntry {
  id: string;
  user_id: string;
  description?: string;
  project_id?: string;
  task_id?: string;
  start_time: string; // ISO datetime string
  end_time?: string; // ISO datetime string, null if running
  duration?: number; // Duration in seconds
  is_running: boolean; // Calculated field
  created_at: string;
  updated_at: string;
}

export interface TimeEntriesListResponse {
  entries: TimeEntry[];
  total_count: number;
  total_duration: number; // Total duration in seconds for filtered results
  page: number;
  per_page: number;
}

export interface CreateTimeEntryRequest {
  description?: string;
  project_id?: string;
  task_id?: string;
  start_time: string; // ISO datetime string
  end_time?: string; // ISO datetime string, optional for running timer
}

export interface UpdateTimeEntryRequest {
  description?: string;
  project_id?: string;
  task_id?: string;
  start_time?: string;
  end_time?: string;
}

export interface StartTimerRequest {
  description?: string;
  project_id?: string;
  task_id?: string;
}

export interface TimeEntryFilters {
  start_date?: string; // ISO date string
  end_date?: string; // ISO date string
  project_id?: string;
  task_id?: string;
  is_running?: boolean;
  page?: number;
  limit?: number;
  sort_by?: "start_time" | "duration";
}

export interface Project {
  id: string;
  name: string;
  description?: string;
  color?: string;
  user_id: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface Task {
  id: string;
  name: string;
  description?: string;
  project_id?: string;
  user_id: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface TimeStats {
  today: number; // seconds
  this_week: number; // seconds
  this_month: number; // seconds
  total_entries_today: number;
  total_entries_week: number;
  total_entries_month: number;
}

// API Response wrapper
export interface ApiResponse<T> {
  data?: T;
  error?: string;
  message?: string;
}

// Error types
export interface TimeEntryError {
  message: string;
  field?: string;
  code?: string;
}

// Timer state
export interface TimerState {
  isRunning: boolean;
  currentEntry: TimeEntry | null;
  elapsedTime: number; // milliseconds since start
  startTime: Date | null;
}

// Form data types
export interface TimeEntryFormData {
  description: string;
  project_id: string;
  task_id: string;
  start_time: Date;
  end_time: Date | null;
}

export interface TimerFormData {
  description: string;
  project_id: string;
  task_id: string;
}
