"use client";

import { AlertTriangle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { formatDate, formatTime, formatDuration } from "@/lib/time-utils";
import type { TimeEntry } from "@/types/time-entries";

interface DeleteConfirmationDialogProps {
  isOpen: boolean;
  onClose: () => void;
  entry: TimeEntry | null;
  onConfirm: () => Promise<void>;
  loading?: boolean;
}

export function DeleteConfirmationDialog({
  isOpen,
  onClose,
  entry,
  onConfirm,
  loading = false,
}: DeleteConfirmationDialogProps) {
  const handleConfirm = async () => {
    try {
      await onConfirm();
      onClose();
    } catch (error) {
      // Error handling is done in the parent component
    }
  };

  if (!entry) return null;

  const isRunning = entry.is_running || !entry.end_time;

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2 text-red-600 dark:text-red-400">
            <AlertTriangle className="h-5 w-5" />
            Delete Time Entry
          </DialogTitle>
          <DialogDescription>
            This action cannot be undone. The time entry will be permanently
            removed from your records.
          </DialogDescription>
        </DialogHeader>

        {/* Entry Details */}
        <div className="space-y-4 py-4">
          <div className="bg-muted/50 p-4 rounded-lg space-y-3">
            {/* Entry Status */}
            {isRunning && (
              <div className="flex items-center gap-2 text-amber-600 dark:text-amber-400">
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse" />
                <span className="text-sm font-medium">
                  This entry is currently running
                </span>
              </div>
            )}

            {/* Description */}
            <div>
              <div className="text-sm font-medium text-muted-foreground mb-1">
                Description
              </div>
              <div className="text-sm">
                {entry.description ? (
                  <span>{entry.description}</span>
                ) : (
                  <span className="text-muted-foreground italic">
                    No description
                  </span>
                )}
              </div>
            </div>

            {/* Date and Time */}
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  Date
                </div>
                <div>{formatDate(entry.start_time)}</div>
              </div>
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  Duration
                </div>
                <div className="font-mono">
                  {entry.duration ? (
                    formatDuration(entry.duration)
                  ) : isRunning ? (
                    <span className="text-muted-foreground">Running...</span>
                  ) : (
                    <span className="text-muted-foreground">0m</span>
                  )}
                </div>
              </div>
            </div>

            {/* Time Range */}
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  Start Time
                </div>
                <div>{formatTime(entry.start_time)}</div>
              </div>
              <div>
                <div className="font-medium text-muted-foreground mb-1">
                  End Time
                </div>
                <div>
                  {entry.end_time ? (
                    formatTime(entry.end_time)
                  ) : (
                    <Badge
                      variant="secondary"
                      className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                    >
                      Running
                    </Badge>
                  )}
                </div>
              </div>
            </div>

            {/* Project and Task */}
            {(entry.project_id || entry.task_id) && (
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <div className="font-medium text-muted-foreground mb-1">
                    Project
                  </div>
                  <div>
                    {entry.project_id ? (
                      <Badge variant="outline">Project Name</Badge>
                    ) : (
                      <span className="text-muted-foreground">None</span>
                    )}
                  </div>
                </div>
                <div>
                  <div className="font-medium text-muted-foreground mb-1">
                    Task
                  </div>
                  <div>
                    {entry.task_id ? (
                      <Badge variant="outline">Task Name</Badge>
                    ) : (
                      <span className="text-muted-foreground">None</span>
                    )}
                  </div>
                </div>
              </div>
            )}
          </div>

          {/* Warning for Running Entry */}
          {isRunning && (
            <div className="bg-amber-50 dark:bg-amber-950/20 border border-amber-200 dark:border-amber-900/30 p-3 rounded-md">
              <div className="flex items-start gap-2">
                <AlertTriangle className="h-4 w-4 text-amber-600 dark:text-amber-400 mt-0.5 flex-shrink-0" />
                <div className="text-sm text-amber-700 dark:text-amber-300">
                  <strong>Warning:</strong> Deleting a running time entry will
                  stop the timer and permanently remove all recorded time.
                </div>
              </div>
            </div>
          )}
        </div>

        <DialogFooter className="flex gap-3">
          <Button
            type="button"
            variant="outline"
            onClick={onClose}
            disabled={loading}
          >
            Cancel
          </Button>
          <Button
            type="button"
            variant="destructive"
            onClick={handleConfirm}
            disabled={loading}
            className="min-w-[100px]"
          >
            {loading ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Deleting...
              </>
            ) : (
              <>Delete Entry</>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
