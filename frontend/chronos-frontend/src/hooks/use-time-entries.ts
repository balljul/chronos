import { useState, useEffect, useCallback } from "react";
import { timeEntriesAPI } from "@/lib/api/time-entries";
import type {
  TimeEntry,
  TimeEntriesListResponse,
  TimeEntryFilters,
  CreateTimeEntryRequest,
  UpdateTimeEntryRequest,
  TimeStats,
} from "@/types/time-entries";

interface TimeEntriesState {
  entries: TimeEntry[];
  totalCount: number;
  totalDuration: number;
  page: number;
  perPage: number;
  loading: boolean;
  error: string | null;
  stats: TimeStats;
  statsLoading: boolean;
}

const DEFAULT_FILTERS: TimeEntryFilters = {
  page: 1,
  limit: 20,
  sort_by: "start_time",
};

export function useTimeEntries(
  initialFilters: TimeEntryFilters = DEFAULT_FILTERS,
) {
  const [state, setState] = useState<TimeEntriesState>({
    entries: [],
    totalCount: 0,
    totalDuration: 0,
    page: 1,
    perPage: 20,
    loading: true,
    error: null,
    stats: {
      today: 0,
      this_week: 0,
      this_month: 0,
      total_entries_today: 0,
      total_entries_week: 0,
      total_entries_month: 0,
    },
    statsLoading: true,
  });

  const [filters, setFilters] = useState<TimeEntryFilters>(initialFilters);

  // Load time entries based on current filters
  const loadTimeEntries = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, loading: true, error: null }));

      const response = await timeEntriesAPI.getTimeEntries(filters);

      setState((prev) => ({
        ...prev,
        entries: response.entries,
        totalCount: response.total_count,
        totalDuration: response.total_duration,
        page: response.page,
        perPage: response.per_page,
        loading: false,
      }));
    } catch (error) {
      console.error("Failed to load time entries:", error);
      setState((prev) => ({
        ...prev,
        loading: false,
        error:
          error instanceof Error
            ? error.message
            : "Failed to load time entries",
      }));
    }
  }, [filters]);

  // Load time tracking statistics
  const loadStats = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, statsLoading: true }));

      const stats = await timeEntriesAPI.getTimeStats();

      setState((prev) => ({
        ...prev,
        stats,
        statsLoading: false,
      }));
    } catch (error) {
      console.error("Failed to load time stats:", error);
      setState((prev) => ({
        ...prev,
        statsLoading: false,
        // Keep existing stats on error, don't clear them
      }));
    }
  }, []);

  // Create a new time entry
  const createEntry = useCallback(
    async (data: CreateTimeEntryRequest) => {
      try {
        const newEntry = await timeEntriesAPI.createTimeEntry(data);

        // Refresh the entries and stats
        await Promise.all([loadTimeEntries(), loadStats()]);

        return newEntry;
      } catch (error) {
        throw error;
      }
    },
    [loadTimeEntries, loadStats],
  );

  // Update an existing time entry
  const updateEntry = useCallback(
    async (id: string, data: UpdateTimeEntryRequest) => {
      try {
        const updatedEntry = await timeEntriesAPI.updateTimeEntry(id, data);

        // Update the entry in the current state optimistically
        setState((prev) => ({
          ...prev,
          entries: prev.entries.map((entry) =>
            entry.id === id ? updatedEntry : entry,
          ),
        }));

        // Refresh stats as duration might have changed
        await loadStats();

        return updatedEntry;
      } catch (error) {
        // Refresh entries on error to revert optimistic update
        await loadTimeEntries();
        throw error;
      }
    },
    [loadTimeEntries, loadStats],
  );

  // Delete a time entry
  const deleteEntry = useCallback(
    async (id: string) => {
      try {
        // Optimistically remove from state
        setState((prev) => ({
          ...prev,
          entries: prev.entries.filter((entry) => entry.id !== id),
          totalCount: Math.max(0, prev.totalCount - 1),
        }));

        await timeEntriesAPI.deleteTimeEntry(id);

        // Refresh to get accurate totals and stats
        await Promise.all([loadTimeEntries(), loadStats()]);
      } catch (error) {
        // Revert optimistic update on error
        await loadTimeEntries();
        throw error;
      }
    },
    [loadTimeEntries, loadStats],
  );

  // Update filters (triggers reload)
  const updateFilters = useCallback((newFilters: Partial<TimeEntryFilters>) => {
    setFilters((prev) => ({
      ...prev,
      ...newFilters,
      // Reset to first page when changing filters (except when explicitly setting page)
      ...(newFilters.page === undefined && { page: 1 }),
    }));
  }, []);

  // Clear all filters
  const clearFilters = useCallback(() => {
    setFilters(DEFAULT_FILTERS);
  }, []);

  // Refresh both entries and stats
  const refresh = useCallback(async () => {
    await Promise.all([loadTimeEntries(), loadStats()]);
  }, [loadTimeEntries, loadStats]);

  // Load data when filters change
  useEffect(() => {
    loadTimeEntries();
  }, [loadTimeEntries]);

  // Load stats on mount
  useEffect(() => {
    loadStats();
  }, [loadStats]);

  // Computed values
  const hasEntries = state.entries.length > 0;
  const hasNextPage = state.page * state.perPage < state.totalCount;
  const hasPreviousPage = state.page > 1;
  const totalPages = Math.ceil(state.totalCount / state.perPage);

  // Active filters count (for display)
  const activeFiltersCount = Object.entries(filters).filter(([key, value]) => {
    if (key === "page" || key === "limit" || key === "sort_by") return false;
    return value !== undefined && value !== null && value !== "";
  }).length;

  return {
    // State
    entries: state.entries,
    stats: state.stats,
    loading: state.loading || state.statsLoading,
    error: state.error,

    // Pagination object to match dashboard expectations
    pagination: {
      totalCount: state.totalCount,
      totalDuration: state.totalDuration,
      page: state.page,
      totalPages,
      hasNextPage,
      hasPreviousPage,
    },

    // Filters
    filters,
    activeFiltersCount,

    // Actions
    createEntry,
    updateEntry,
    deleteEntry,
    setPage: (page: number) => updateFilters({ page }),
    setFilters: updateFilters,
    refreshEntries: refresh,
    clearFilters,

    // Computed
    hasEntries,
    isEmpty: !hasEntries && !state.loading,
  };
}
