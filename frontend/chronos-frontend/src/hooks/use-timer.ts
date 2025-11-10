import { useState, useEffect, useCallback, useRef } from "react";
import { timeEntriesAPI } from "@/lib/api/time-entries";
import type { TimeEntry, StartTimerRequest } from "@/types/time-entries";

interface TimerState {
  isRunning: boolean;
  currentEntry: TimeEntry | null;
  elapsedTime: number; // seconds
  loading: boolean;
  error: string | null;
}

export function useTimer() {
  const [state, setState] = useState<TimerState>({
    isRunning: false,
    currentEntry: null,
    elapsedTime: 0,
    loading: true,
    error: null,
  });

  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  // Calculate elapsed time from start_time
  const calculateElapsedTime = useCallback((startTime: string): number => {
    if (!startTime) {
      console.error("No start time provided");
      return 0;
    }

    const start = new Date(startTime);
    const now = new Date();
    
    if (isNaN(start.getTime())) {
      console.error("Invalid start time:", startTime);
      return 0;
    }
    
    const elapsed = Math.floor((now.getTime() - start.getTime()) / 1000);
    return Math.max(0, elapsed); // Ensure non-negative
  }, []);

  // Start the real-time timer update
  const startTimerUpdate = useCallback(
    (entry: TimeEntry) => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }

      intervalRef.current = setInterval(() => {
        setState((prev) => ({
          ...prev,
          elapsedTime: calculateElapsedTime(entry.start_time),
        }));
      }, 1000);
    },
    [calculateElapsedTime],
  );

  // Stop the real-time timer update
  const stopTimerUpdate = useCallback(() => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  // Load current timer state on mount
  const loadCurrentTimer = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, loading: true, error: null }));

      const currentEntry = await timeEntriesAPI.getCurrentTimer();

      if (currentEntry && currentEntry.end_time === null) {
        // Timer is running
        const elapsed = calculateElapsedTime(currentEntry.start_time);
        setState((prev) => ({
          ...prev,
          isRunning: true,
          currentEntry,
          elapsedTime: elapsed,
          loading: false,
        }));
        startTimerUpdate(currentEntry);
      } else {
        // No timer running
        setState((prev) => ({
          ...prev,
          isRunning: false,
          currentEntry: null,
          elapsedTime: 0,
          loading: false,
        }));
      }
    } catch (error) {
      console.error("Failed to load current timer:", error);
      setState((prev) => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : "Failed to load timer",
        isRunning: false,
        currentEntry: null,
        elapsedTime: 0,
      }));
    }
  }, [calculateElapsedTime, startTimerUpdate]);

  // Start a new timer
  const startTimer = useCallback(
    async (data: StartTimerRequest) => {
      try {
        setState((prev) => ({ ...prev, loading: true, error: null }));

        const newEntry = await timeEntriesAPI.startTimer(data);

        setState((prev) => ({
          ...prev,
          isRunning: true,
          currentEntry: newEntry,
          elapsedTime: 0,
          loading: false,
        }));

        startTimerUpdate(newEntry);
        return newEntry;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          loading: false,
          error:
            error instanceof Error ? error.message : "Failed to start timer",
        }));
        throw error;
      }
    },
    [startTimerUpdate],
  );

  // Stop the current timer
  const stopTimer = useCallback(async () => {
    if (!state.currentEntry) {
      throw new Error("No timer is currently running");
    }

    try {
      setState((prev) => ({ ...prev, loading: true, error: null }));

      const stoppedEntry = await timeEntriesAPI.stopTimer(
        state.currentEntry.id,
      );

      stopTimerUpdate();

      setState((prev) => ({
        ...prev,
        isRunning: false,
        currentEntry: null,
        elapsedTime: 0,
        loading: false,
      }));

      return stoppedEntry;
    } catch (error) {
      setState((prev) => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : "Failed to stop timer",
      }));
      throw error;
    }
  }, [state.currentEntry, stopTimerUpdate]);

  // Refresh timer state (useful after operations)
  const refreshTimer = useCallback(async () => {
    await loadCurrentTimer();
  }, [loadCurrentTimer]);

  // Load timer state on mount
  useEffect(() => {
    loadCurrentTimer();
  }, [loadCurrentTimer]);

  // Cleanup interval on unmount
  useEffect(() => {
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  // Handle page visibility changes (pause/resume timer when tab is hidden/visible)
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (state.isRunning && state.currentEntry) {
        if (document.hidden) {
          // Page is hidden, stop the timer update to save resources
          stopTimerUpdate();
        } else {
          // Page is visible again, restart timer update and recalculate elapsed time
          const elapsed = calculateElapsedTime(state.currentEntry.start_time);
          setState((prev) => ({ ...prev, elapsedTime: elapsed }));
          startTimerUpdate(state.currentEntry);
        }
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [
    state.isRunning,
    state.currentEntry,
    calculateElapsedTime,
    startTimerUpdate,
    stopTimerUpdate,
  ]);

  return {
    // State
    isRunning: state.isRunning,
    currentEntry: state.currentEntry,
    elapsedTime: state.elapsedTime,
    loading: state.loading,
    error: state.error,

    // Actions
    startTimer,
    stopTimer,
    refreshTimer,

    // Computed values
    canStart: !state.isRunning && !state.loading,
    canStop: state.isRunning && !state.loading,
  };
}
