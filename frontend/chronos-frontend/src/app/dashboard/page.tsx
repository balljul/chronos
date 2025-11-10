"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState, useRef } from "react";
import { Play, Square } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { formatTimerDuration } from "@/lib/time-utils";
import { toast } from "sonner";
import { timeEntriesAPI } from "@/lib/api/time-entries";
import type { TimeEntry } from "@/types/time-entries";

interface ProfileData {
  id: string;
  name?: string;
  email: string;
  created_at: string;
  updated_at: string;
}

export default function Dashboard() {
  const [profile, setProfile] = useState<ProfileData | null>(null);
  const [authLoading, setAuthLoading] = useState(true);
  const [error, setError] = useState("");
  const router = useRouter();

  // Timer state
  const [isRunning, setIsRunning] = useState(false);
  const [elapsedTime, setElapsedTime] = useState(0);
  const [description, setDescription] = useState("");
  const [currentEntry, setCurrentEntry] = useState<TimeEntry | null>(null);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  const getAuthToken = useCallback(() => {
    if (typeof window === "undefined") {
      return null;
    }
    return localStorage.getItem("access_token");
  }, []);

  const logout = async () => {
    try {
      const token = getAuthToken();
      if (token) {
        await fetch("/api/auth/logout", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${token}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            refresh_token:
              typeof window !== "undefined"
                ? localStorage.getItem("refresh_token")
                : null,
            logout_all_devices: false,
          }),
        });
      }
    } catch (error) {
      console.error("Logout error:", error);
    } finally {
      if (typeof window !== "undefined") {
        localStorage.removeItem("access_token");
        localStorage.removeItem("refresh_token");
      }
      router.push("/login");
    }
  };

  const fetchProfile = useCallback(async () => {
    try {
      const token = getAuthToken();
      if (!token) {
        router.push("/login");
        return;
      }

      const response = await fetch("/api/profile", {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (response.status === 401) {
        if (typeof window !== "undefined") {
          localStorage.removeItem("access_token");
          localStorage.removeItem("refresh_token");
        }
        router.push("/login");
        return;
      }

      if (!response.ok) {
        throw new Error("Failed to fetch profile");
      }

      const data = await response.json();
      setProfile(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load profile");
    }
  }, [router, getAuthToken]);

  // Timer functions
  const calculateElapsedTime = (startTime: string): number => {
    const start = new Date(startTime);
    const now = new Date();
    return Math.floor((now.getTime() - start.getTime()) / 1000);
  };

  const startTimer = (entry: TimeEntry) => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }
    
    intervalRef.current = setInterval(() => {
      setElapsedTime(calculateElapsedTime(entry.start_time));
    }, 1000);
  };

  const stopTimer = () => {
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  };

  const loadCurrentTimer = async () => {
    try {
      const entry = await timeEntriesAPI.getCurrentTimer();
      if (entry && !entry.end_time) {
        setCurrentEntry(entry);
        setIsRunning(true);
        setDescription(entry.description || "");
        const elapsed = calculateElapsedTime(entry.start_time);
        setElapsedTime(elapsed);
        startTimer(entry);
      } else {
        setCurrentEntry(null);
        setIsRunning(false);
        setElapsedTime(0);
      }
    } catch (error) {
      console.error("Failed to load current timer:", error);
    }
  };

  const handleStart = async () => {
    try {
      const newEntry = await timeEntriesAPI.startTimer({
        description: description.trim() || undefined,
      });
      
      setCurrentEntry(newEntry);
      setIsRunning(true);
      setElapsedTime(0);
      startTimer(newEntry);
      toast.success("Timer started");
    } catch (error) {
      toast.error("Failed to start timer");
      console.error("Start timer error:", error);
    }
  };

  const handleStop = async () => {
    if (!currentEntry) return;
    
    try {
      stopTimer();
      await timeEntriesAPI.stopTimer(currentEntry.id);
      
      setIsRunning(false);
      setCurrentEntry(null);
      setElapsedTime(0);
      setDescription("");
      toast.success("Timer stopped");
    } catch (error) {
      toast.error("Failed to stop timer");
      console.error("Stop timer error:", error);
      loadCurrentTimer();
    }
  };

  useEffect(() => {
    const loadDashboardData = async () => {
      setAuthLoading(true);
      await fetchProfile();
      setAuthLoading(false);
    };

    loadDashboardData();
  }, [fetchProfile]);

  useEffect(() => {
    loadCurrentTimer();
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-600 mx-auto"></div>
          <p className="mt-4 text-gray-600 dark:text-gray-400">
            Loading dashboard...
          </p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <p className="text-red-600 dark:text-red-400">{error}</p>
          <button
            type="button"
            onClick={() => router.push("/login")}
            className="mt-4 px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700"
          >
            Back to Login
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900 flex flex-col">
      {/* Header */}
      <header className="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
              Chronos
            </h1>
            <div className="flex items-center space-x-3">
              <button
                type="button"
                onClick={() => router.push("/profile")}
                className="text-sm px-3 py-1.5 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-md transition-colors"
              >
                Profile
              </button>
              <button
                type="button"
                onClick={logout}
                className="text-sm px-3 py-1.5 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-md transition-colors"
              >
                Logout
              </button>
            </div>
          </div>
        </div>
      </header>

      {/* Timer */}
      <main className="flex-1 flex items-center justify-center px-4 sm:px-6 lg:px-8">
        <div className="w-full max-w-md mx-auto border border-gray-200 dark:border-gray-800 rounded-lg p-8 bg-gray-50/50 dark:bg-gray-900/50">
          {/* Status Badge */}
          {isRunning && (
            <div className="flex justify-center mb-6">
              <Badge
                variant="secondary"
                className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 px-3 py-1"
              >
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse mr-2" />
                Running
              </Badge>
            </div>
          )}

          {/* Timer Display */}
          <div className="text-center space-y-2 mb-8">
            <div className="text-6xl font-mono font-bold text-gray-900 dark:text-gray-100">
              {formatTimerDuration(elapsedTime)}
            </div>
            {isRunning && currentEntry && (
              <div className="text-sm text-muted-foreground">
                Started at {new Date(currentEntry.start_time).toLocaleTimeString()}
              </div>
            )}
          </div>

          {/* Description Input */}
          <div className="space-y-2 mb-6">
            <Input
              placeholder={
                isRunning ? "What are you working on?" : "What will you work on?"
              }
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="text-center border-gray-300 dark:border-gray-700"
              maxLength={1000}
            />
          </div>

          {/* Control Button */}
          <div className="flex justify-center mb-6">
            {isRunning ? (
              <Button
                onClick={handleStop}
                variant="destructive"
                className="px-8"
              >
                <Square className="mr-2 h-4 w-4" />
                Stop
              </Button>
            ) : (
              <Button
                onClick={handleStart}
                className="px-8 bg-green-600 hover:bg-green-700"
              >
                <Play className="mr-2 h-4 w-4" />
                Start
              </Button>
            )}
          </div>

          {/* Quick Actions */}
          {!isRunning && (
            <div className="flex gap-2 justify-center">
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDescription("Meeting")}
                className="border-gray-300 dark:border-gray-700"
              >
                Meeting
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDescription("Development")}
                className="border-gray-300 dark:border-gray-700"
              >
                Development
              </Button>
              <Button
                variant="outline"
                size="sm"
                onClick={() => setDescription("Break")}
                className="border-gray-300 dark:border-gray-700"
              >
                Break
              </Button>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}
