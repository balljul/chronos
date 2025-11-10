// Time entries API client functions
import type {
  TimeEntry,
  TimeEntriesListResponse,
  CreateTimeEntryRequest,
  UpdateTimeEntryRequest,
  StartTimerRequest,
  TimeEntryFilters,
  ApiResponse,
} from "@/types/time-entries";

class TimeEntriesAPI {
  private getAuthHeaders(): HeadersInit {
    const token =
      typeof window !== "undefined"
        ? localStorage.getItem("access_token")
        : null;

    return {
      "Content-Type": "application/json",
      ...(token && { Authorization: `Bearer ${token}` }),
    };
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const errorData = await response
        .json()
        .catch(() => ({ error: "Request failed" }));
      throw new Error(
        errorData.error || `HTTP ${response.status}: ${response.statusText}`,
      );
    }

    // Handle 204 No Content responses
    if (response.status === 204) {
      return {} as T;
    }

    return response.json();
  }

  /**
   * Get list of time entries with optional filters
   */
  async getTimeEntries(
    filters?: TimeEntryFilters,
  ): Promise<TimeEntriesListResponse> {
    const params = new URLSearchParams();

    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== "") {
          params.append(key, String(value));
        }
      });
    }

    const url = `/api/time-entries${params.toString() ? `?${params.toString()}` : ""}`;

    const response = await fetch(url, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<TimeEntriesListResponse>(response);
  }

  /**
   * Get a single time entry by ID
   */
  async getTimeEntry(id: string): Promise<TimeEntry> {
    const response = await fetch(`/api/time-entries/${id}`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<TimeEntry>(response);
  }

  /**
   * Create a new time entry
   */
  async createTimeEntry(data: CreateTimeEntryRequest): Promise<TimeEntry> {
    const response = await fetch("/api/time-entries", {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(data),
    });

    return this.handleResponse<TimeEntry>(response);
  }

  /**
   * Update an existing time entry
   */
  async updateTimeEntry(
    id: string,
    data: UpdateTimeEntryRequest,
  ): Promise<TimeEntry> {
    const response = await fetch(`/api/time-entries/${id}`, {
      method: "PATCH",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(data),
    });

    return this.handleResponse<TimeEntry>(response);
  }

  /**
   * Delete a time entry
   */
  async deleteTimeEntry(id: string): Promise<void> {
    const response = await fetch(`/api/time-entries/${id}`, {
      method: "DELETE",
      headers: this.getAuthHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  /**
   * Start a new timer
   */
  async startTimer(data: StartTimerRequest): Promise<TimeEntry> {
    const response = await fetch("/api/time-entries/start", {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(data),
    });

    return this.handleResponse<TimeEntry>(response);
  }

  /**
   * Stop a running timer
   */
  async stopTimer(id: string): Promise<TimeEntry> {
    const response = await fetch(`/api/time-entries/${id}/stop`, {
      method: "PATCH",
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<TimeEntry>(response);
  }

  /**
   * Get currently running timer (returns null if no timer is running)
   */
  async getCurrentTimer(): Promise<TimeEntry | null> {
    try {
      const response = await fetch("/api/time-entries/current", {
        method: "GET",
        headers: this.getAuthHeaders(),
      });

      const result = await this.handleResponse<{ data: TimeEntry | null }>(
        response,
      );
      return result.data;
    } catch (error) {
      // If 404, no timer is running - return null
      if (error instanceof Error && error.message.includes("404")) {
        return null;
      }
      throw error;
    }
  }

  /**
   * Get time tracking statistics
   */
  async getTimeStats(): Promise<{
    today: number;
    this_week: number;
    this_month: number;
    total_entries_today: number;
    total_entries_week: number;
    total_entries_month: number;
  }> {
    const now = new Date();

    // Get today's data
    const todayStart = new Date(now);
    todayStart.setHours(0, 0, 0, 0);
    const todayEnd = new Date(now);
    todayEnd.setHours(23, 59, 59, 999);

    // Get this week's data (Monday to Sunday)
    const weekStart = new Date(now);
    const day = weekStart.getDay();
    const diff = weekStart.getDate() - day + (day === 0 ? -6 : 1);
    weekStart.setDate(diff);
    weekStart.setHours(0, 0, 0, 0);

    const weekEnd = new Date(weekStart);
    weekEnd.setDate(weekEnd.getDate() + 6);
    weekEnd.setHours(23, 59, 59, 999);

    // Get this month's data
    const monthStart = new Date(now.getFullYear(), now.getMonth(), 1);
    const monthEnd = new Date(now.getFullYear(), now.getMonth() + 1, 0);
    monthEnd.setHours(23, 59, 59, 999);

    try {
      // Fetch data for each period
      const [todayData, weekData, monthData] = await Promise.all([
        this.getTimeEntries({
          start_date: todayStart.toISOString(),
          end_date: todayEnd.toISOString(),
          limit: 100,
        }),
        this.getTimeEntries({
          start_date: weekStart.toISOString(),
          end_date: weekEnd.toISOString(),
          limit: 100,
        }),
        this.getTimeEntries({
          start_date: monthStart.toISOString(),
          end_date: monthEnd.toISOString(),
          limit: 100,
        }),
      ]);

      return {
        today: todayData.total_duration,
        this_week: weekData.total_duration,
        this_month: monthData.total_duration,
        total_entries_today: todayData.entries.length,
        total_entries_week: weekData.entries.length,
        total_entries_month: monthData.entries.length,
      };
    } catch (error) {
      console.error("Error fetching time stats:", error);
      // Return zeros if there's an error
      return {
        today: 0,
        this_week: 0,
        this_month: 0,
        total_entries_today: 0,
        total_entries_week: 0,
        total_entries_month: 0,
      };
    }
  }
}

// Export singleton instance
export const timeEntriesAPI = new TimeEntriesAPI();

// Export individual functions for convenience
export const {
  getTimeEntries,
  getTimeEntry,
  createTimeEntry,
  updateTimeEntry,
  deleteTimeEntry,
  startTimer,
  stopTimer,
  getCurrentTimer,
  getTimeStats,
} = timeEntriesAPI;
