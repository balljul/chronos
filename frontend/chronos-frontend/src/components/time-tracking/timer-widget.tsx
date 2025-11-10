"use client";

import { useState } from "react";
import { Play, Square, Clock, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useTimer } from "@/hooks/use-timer";
import { formatTimerDuration } from "@/lib/time-utils";
import { toast } from "sonner";

interface TimerWidgetProps {
  onTimerStart?: () => void;
  onTimerStop?: () => void;
}

export function TimerWidget({ onTimerStart, onTimerStop }: TimerWidgetProps) {
  const {
    isRunning,
    currentEntry,
    elapsedTime,
    loading,
    error,
    startTimer,
    stopTimer,
    canStart,
    canStop,
  } = useTimer();

  const [description, setDescription] = useState("");
  const [projectId, setProjectId] = useState("");
  const [taskId, setTaskId] = useState("");

  const handleStartTimer = async () => {
    try {
      await startTimer({
        description: description.trim() || undefined,
        project_id: projectId || undefined,
        task_id: taskId || undefined,
      });

      toast.success("Timer started");
      onTimerStart?.();

      // Keep description for editing, but clear project/task for next entry
      setProjectId("");
      setTaskId("");
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to start timer",
      );
    }
  };

  const handleStopTimer = async () => {
    try {
      await stopTimer();
      toast.success("Timer stopped");
      onTimerStop?.();

      // Clear description after stopping
      setDescription("");
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to stop timer",
      );
    }
  };

  // Update description from current entry when timer is running
  const currentDescription =
    isRunning && currentEntry ? currentEntry.description || "" : description;

  if (loading && !isRunning) {
    return (
      <div className="w-full max-w-md mx-auto text-center">
        <Loader2 className="h-8 w-8 animate-spin text-muted-foreground mx-auto" />
      </div>
    );
  }

  return (
    <div className="w-full max-w-md mx-auto border border-gray-200 dark:border-gray-800 rounded-lg p-8 bg-gray-50/50 dark:bg-gray-900/50">
      {error && (
        <div className="text-sm text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-950 p-3 rounded-md border border-red-200 dark:border-red-800 mb-6">
          {error}
        </div>
      )}

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
        {isRunning && currentEntry?.start_time && (
          <div className="text-sm text-muted-foreground">
            Started at {
              (() => {
                const date = new Date(currentEntry.start_time);
                return isNaN(date.getTime()) 
                  ? "Invalid Date" 
                  : date.toLocaleTimeString();
              })()
            }
          </div>
        )}
      </div>

      {/* Description Input */}
      <div className="space-y-2 mb-6">
        <Input
          placeholder={
            isRunning ? "What are you working on?" : "What will you work on?"
          }
          value={currentDescription}
          onChange={(e) => setDescription(e.target.value)}
          disabled={isRunning || loading}
          className="text-center border-gray-300 dark:border-gray-700"
          maxLength={1000}
        />
      </div>

      {/* Project and Task Selectors */}
      {!isRunning && (
        <div className="grid grid-cols-2 gap-3 mb-6">
          <Select
            value={projectId}
            onValueChange={setProjectId}
            disabled={loading}
          >
            <SelectTrigger className="border-gray-300 dark:border-gray-700">
              <SelectValue placeholder="Project" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none">No project</SelectItem>
              <SelectItem value="project-1">Sample Project</SelectItem>
            </SelectContent>
          </Select>

          <Select
            value={taskId}
            onValueChange={setTaskId}
            disabled={loading}
          >
            <SelectTrigger className="border-gray-300 dark:border-gray-700">
              <SelectValue placeholder="Task" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none">No task</SelectItem>
              <SelectItem value="task-1">Sample Task</SelectItem>
            </SelectContent>
          </Select>
        </div>
      )}

      {/* Current Entry Details */}
      {isRunning && currentEntry && (currentEntry.project_id || currentEntry.task_id) && (
        <div className="text-center text-sm text-muted-foreground space-y-1 mb-6 p-3 border border-gray-200 dark:border-gray-700 rounded-md bg-gray-100/50 dark:bg-gray-800/50">
          {currentEntry.project_id && <div>Project: Project Name</div>}
          {currentEntry.task_id && <div>Task: Task Name</div>}
        </div>
      )}

      {/* Control Button */}
      <div className="flex justify-center mb-6">
        {isRunning ? (
          <Button
            onClick={handleStopTimer}
            disabled={!canStop}
            variant="destructive"
            className="px-8"
          >
            {loading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Square className="mr-2 h-4 w-4" />
            )}
            Stop
          </Button>
        ) : (
          <Button
            onClick={handleStartTimer}
            disabled={!canStart}
            className="px-8 bg-green-600 hover:bg-green-700"
          >
            {loading ? (
              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            ) : (
              <Play className="mr-2 h-4 w-4" />
            )}
            Start
          </Button>
        )}
      </div>

      {/* Quick Actions */}
      {!isRunning && !loading && (
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
  );
}
