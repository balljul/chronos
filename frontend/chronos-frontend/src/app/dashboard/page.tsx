"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState, useRef } from "react";
import { Play, Square, FolderOpen, CheckSquare } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { formatTimerDuration } from "@/lib/time-utils";
import { toast } from "sonner";
import { timeEntriesAPI } from "@/lib/api/time-entries";
import { projectsAPI, type Project } from "@/lib/api/projects";
import { tasksAPI, type Task } from "@/lib/api/tasks";
import TimeEntriesTable from "@/components/time-entries-table";
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
  const [selectedProjectId, setSelectedProjectId] = useState<string | undefined>(undefined);
  const [selectedTaskId, setSelectedTaskId] = useState<string | undefined>(undefined);
  const [currentEntry, setCurrentEntry] = useState<TimeEntry | null>(null);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);
  const descriptionTimeoutRef = useRef<NodeJS.Timeout | null>(null);

  // Projects and tasks state
  const [projects, setProjects] = useState<Project[]>([]);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [filteredTasks, setFilteredTasks] = useState<Task[]>([]);

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

  const loadProjects = useCallback(async () => {
    try {
      const data = await projectsAPI.getProjects({ include_inactive: false });
      setProjects(data);
    } catch (err) {
      console.error("Failed to load projects:", err);
    }
  }, []);

  const loadTasks = useCallback(async () => {
    try {
      const data = await tasksAPI.getTasks({ include_inactive: false });
      setTasks(data);
      setFilteredTasks(data);
    } catch (err) {
      console.error("Failed to load tasks:", err);
    }
  }, []);

  const filterTasksByProject = useCallback((projectId: string | undefined) => {
    if (!projectId) {
      setFilteredTasks(tasks);
    } else {
      const filtered = tasks.filter(task => task.project_id === projectId);
      setFilteredTasks(filtered);
    }
    setSelectedTaskId(undefined);
  }, [tasks]);

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
        setSelectedProjectId(entry.project_id || undefined);
        setSelectedTaskId(entry.task_id || undefined);
        const elapsed = calculateElapsedTime(entry.start_time);
        setElapsedTime(elapsed);
        startTimer(entry);
      } else {
        setCurrentEntry(null);
        setIsRunning(false);
        setElapsedTime(0);
        setSelectedProjectId(undefined);
        setSelectedTaskId(undefined);
      }
    } catch (error) {
      console.error("Failed to load current timer:", error);
    }
  };

  const handleStart = async () => {
    try {
      const newEntry = await timeEntriesAPI.startTimer({
        description: description.trim() || undefined,
        project_id: selectedProjectId,
        task_id: selectedTaskId,
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
      setSelectedProjectId(undefined);
      setSelectedTaskId(undefined);
      toast.success("Timer stopped");
    } catch (error) {
      toast.error("Failed to stop timer");
      console.error("Stop timer error:", error);
      loadCurrentTimer();
    }
  };

  const updateDescription = async (newDescription: string) => {
    if (!currentEntry || !isRunning) return;
    
    try {
      await timeEntriesAPI.updateTimeEntry(currentEntry.id, {
        description: newDescription.trim() || undefined,
      });
      
      setCurrentEntry(prev => prev ? { ...prev, description: newDescription } : null);
    } catch (error) {
      console.error("Failed to update description:", error);
      toast.error("Failed to update description");
    }
  };

  const handleDescriptionChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const newValue = e.target.value;
    setDescription(newValue);

    if (isRunning && currentEntry) {
      // Debounce the API call
      if (descriptionTimeoutRef.current) {
        clearTimeout(descriptionTimeoutRef.current);
      }
      descriptionTimeoutRef.current = setTimeout(() => {
        updateDescription(newValue);
      }, 1000);
    }
  };

  const handleProjectChange = (projectId: string | undefined) => {
    setSelectedProjectId(projectId);
    filterTasksByProject(projectId);

    if (isRunning && currentEntry) {
      updateEntryDetails({ project_id: projectId });
    }
  };

  const handleTaskChange = (taskId: string | undefined) => {
    setSelectedTaskId(taskId);

    if (isRunning && currentEntry) {
      updateEntryDetails({ task_id: taskId });
    }
  };

  const updateEntryDetails = async (updates: { project_id?: string; task_id?: string }) => {
    if (!currentEntry || !isRunning) return;

    try {
      await timeEntriesAPI.updateTimeEntry(currentEntry.id, updates);
      setCurrentEntry(prev => prev ? { ...prev, ...updates } : null);
    } catch (error) {
      console.error("Failed to update entry details:", error);
      toast.error("Failed to update timer details");
    }
  };

  useEffect(() => {
    const loadDashboardData = async () => {
      setAuthLoading(true);
      await fetchProfile();
      await Promise.all([loadProjects(), loadTasks()]);
      setAuthLoading(false);
    };

    loadDashboardData();
  }, [fetchProfile, loadProjects, loadTasks]);

  useEffect(() => {
    filterTasksByProject(selectedProjectId);
  }, [selectedProjectId, filterTasksByProject]);

  useEffect(() => {
    loadCurrentTimer();
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
      if (descriptionTimeoutRef.current) {
        clearTimeout(descriptionTimeoutRef.current);
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
      {/* Header with Timer */}
      <header className="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
              Chronos
            </h1>
            
            {/* Timer in Navbar */}
            <div className="flex items-center space-x-3">
              <Select
                value={selectedProjectId || "no-project"}
                onValueChange={(value) => handleProjectChange(value === "no-project" ? undefined : value)}
                disabled={isRunning}
              >
                <SelectTrigger className="w-40 text-sm">
                  <SelectValue placeholder="Project">
                    {selectedProjectId ?
                      <div className="flex items-center space-x-2">
                        {projects.find(p => p.id === selectedProjectId)?.color && (
                          <div
                            className="w-2 h-2 rounded-full"
                            style={{ backgroundColor: projects.find(p => p.id === selectedProjectId)?.color }}
                          />
                        )}
                        <span>{projects.find(p => p.id === selectedProjectId)?.name}</span>
                      </div>
                      : "Project"
                    }
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="no-project">No Project</SelectItem>
                  {projects.map((project) => (
                    <SelectItem key={project.id} value={project.id}>
                      <div className="flex items-center space-x-2">
                        {project.color && (
                          <div
                            className="w-3 h-3 rounded-full"
                            style={{ backgroundColor: project.color }}
                          />
                        )}
                        <span>{project.name}</span>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <Select
                value={selectedTaskId || "no-task"}
                onValueChange={(value) => handleTaskChange(value === "no-task" ? undefined : value)}
                disabled={isRunning}
              >
                <SelectTrigger className="w-40 text-sm">
                  <SelectValue placeholder="Task">
                    {selectedTaskId ?
                      filteredTasks.find(t => t.id === selectedTaskId)?.name || "Unknown Task"
                      : "Task"
                    }
                  </SelectValue>
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="no-task">No Task</SelectItem>
                  {filteredTasks.map((task) => (
                    <SelectItem key={task.id} value={task.id}>
                      {task.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>

              <Input
                placeholder="What are you working on?"
                value={description}
                onChange={handleDescriptionChange}
                className="w-48 text-sm"
                maxLength={1000}
              />

              <div className="font-mono text-lg font-semibold text-gray-900 dark:text-white">
                {formatTimerDuration(elapsedTime)}
              </div>

              {isRunning ? (
                <Button
                  onClick={handleStop}
                  variant="destructive"
                  size="sm"
                >
                  <Square className="h-4 w-4" />
                </Button>
              ) : (
                <Button
                  onClick={handleStart}
                  size="sm"
                  className="bg-green-600 hover:bg-green-700"
                >
                  <Play className="h-4 w-4" />
                </Button>
              )}
            </div>
            
            <div className="flex items-center space-x-3">
              <button
                type="button"
                onClick={() => router.push("/projects")}
                className="text-sm px-3 py-1.5 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-md transition-colors flex items-center space-x-1"
              >
                <FolderOpen className="h-4 w-4" />
                <span>Projects</span>
              </button>
              <button
                type="button"
                onClick={() => router.push("/tasks")}
                className="text-sm px-3 py-1.5 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-md transition-colors flex items-center space-x-1"
              >
                <CheckSquare className="h-4 w-4" />
                <span>Tasks</span>
              </button>
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

      {/* Main Content */}
      <main className="flex-1 px-4 sm:px-6 lg:px-8 py-8">
        <div className="max-w-7xl mx-auto">
          <TimeEntriesTable onRefresh={loadCurrentTimer} />
        </div>
      </main>
    </div>
  );
}
