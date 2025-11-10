"use client";

import { useState, useEffect } from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { CalendarIcon, Clock, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Calendar } from "@/components/ui/calendar";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { formatDate, formatTime } from "@/lib/time-utils";
import {
  createTimeEntrySchema,
  updateTimeEntrySchema,
} from "@/lib/validations/time-entry";
import type {
  TimeEntry,
  CreateTimeEntryFormData,
  UpdateTimeEntryFormData,
} from "@/types/time-entries";

interface TimeEntryFormProps {
  isOpen: boolean;
  onClose: () => void;
  entry?: TimeEntry | null;
  onSubmit: (
    data: CreateTimeEntryFormData | UpdateTimeEntryFormData,
  ) => Promise<void>;
  loading?: boolean;
}

export function TimeEntryForm({
  isOpen,
  onClose,
  entry = null,
  onSubmit,
  loading = false,
}: TimeEntryFormProps) {
  const [datePickerOpen, setDatePickerOpen] = useState<"start" | "end" | null>(
    null,
  );
  const [submitting, setSubmitting] = useState(false);

  const isEditing = Boolean(entry?.id);
  const schema = isEditing ? updateTimeEntrySchema : createTimeEntrySchema;

  const form = useForm<CreateTimeEntryFormData | UpdateTimeEntryFormData>({
    resolver: zodResolver(schema),
    defaultValues: {
      description: "",
      start_time: new Date(),
      end_time: undefined,
      project_id: "",
      task_id: "",
    },
  });

  const {
    register,
    handleSubmit,
    setValue,
    watch,
    reset,
    formState: { errors },
  } = form;

  // Watch form values for real-time validation
  const startTime = watch("start_time");
  const endTime = watch("end_time");

  // Reset form when dialog opens/closes or entry changes
  useEffect(() => {
    if (isOpen) {
      if (entry) {
        reset({
          description: entry.description || "",
          start_time: new Date(entry.start_time),
          end_time: entry.end_time ? new Date(entry.end_time) : undefined,
          project_id: entry.project_id || "",
          task_id: entry.task_id || "",
        });
      } else {
        reset({
          description: "",
          start_time: new Date(),
          end_time: undefined,
          project_id: "",
          task_id: "",
        });
      }
    }
  }, [isOpen, entry, reset]);

  const handleFormSubmit = async (
    data: CreateTimeEntryFormData | UpdateTimeEntryFormData,
  ) => {
    setSubmitting(true);
    try {
      // Clean up empty string values
      const cleanedData = {
        ...data,
        description: data.description?.trim() || undefined,
        project_id: data.project_id || undefined,
        task_id: data.task_id || undefined,
      };

      await onSubmit(cleanedData);
      onClose();
    } catch (error) {
      // Error handling is done in the parent component
    } finally {
      setSubmitting(false);
    }
  };

  const handleTimeChange = (field: "start_time" | "end_time", time: string) => {
    const currentValue = watch(field);
    if (currentValue) {
      const [hours, minutes] = time.split(":").map(Number);
      const newDate = new Date(currentValue);
      newDate.setHours(hours, minutes, 0, 0);
      setValue(field, newDate);
    }
  };

  const formatTimeForInput = (date: Date | undefined): string => {
    if (!date) return "";
    return date.toLocaleTimeString("en-GB", {
      hour12: false,
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <Dialog open={isOpen} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[500px] max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            {isEditing ? "Edit Time Entry" : "Create Time Entry"}
          </DialogTitle>
        </DialogHeader>

        <form onSubmit={handleSubmit(handleFormSubmit)} className="space-y-6">
          {/* Description */}
          <div className="space-y-2">
            <Label htmlFor="description">Description</Label>
            <Textarea
              id="description"
              placeholder="What did you work on?"
              className="min-h-[80px] resize-none"
              maxLength={1000}
              {...register("description")}
            />
            {errors.description && (
              <p className="text-sm text-red-600 dark:text-red-400">
                {errors.description.message}
              </p>
            )}
          </div>

          {/* Date Selection */}
          <div className="space-y-2">
            <Label>Date</Label>
            <Popover
              open={datePickerOpen === "start"}
              onOpenChange={(open) => setDatePickerOpen(open ? "start" : null)}
            >
              <PopoverTrigger asChild>
                <Button
                  variant="outline"
                  className="w-full justify-start text-left font-normal"
                  disabled={loading || submitting}
                >
                  <CalendarIcon className="mr-2 h-4 w-4" />
                  {startTime ? (
                    formatDate(startTime.toISOString())
                  ) : (
                    <span className="text-muted-foreground">Pick a date</span>
                  )}
                </Button>
              </PopoverTrigger>
              <PopoverContent className="w-auto p-0">
                <Calendar
                  mode="single"
                  selected={startTime}
                  onSelect={(date) => {
                    if (date) {
                      setValue("start_time", date);
                    }
                    setDatePickerOpen(null);
                  }}
                  disabled={(date) => date > new Date()}
                  initialFocus
                />
              </PopoverContent>
            </Popover>
          </div>

          {/* Time Range */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="start-time">Start Time</Label>
              <Input
                id="start-time"
                type="time"
                step="60"
                value={formatTimeForInput(startTime)}
                onChange={(e) => handleTimeChange("start_time", e.target.value)}
                disabled={loading || submitting}
              />
              {errors.start_time && (
                <p className="text-sm text-red-600 dark:text-red-400">
                  {errors.start_time.message}
                </p>
              )}
            </div>

            <div className="space-y-2">
              <Label htmlFor="end-time">End Time</Label>
              <Input
                id="end-time"
                type="time"
                step="60"
                value={formatTimeForInput(endTime)}
                onChange={(e) => handleTimeChange("end_time", e.target.value)}
                disabled={loading || submitting}
                placeholder="Optional"
              />
              {errors.end_time && (
                <p className="text-sm text-red-600 dark:text-red-400">
                  {errors.end_time.message}
                </p>
              )}
            </div>
          </div>

          {/* Duration Preview */}
          {startTime && endTime && endTime > startTime && (
            <div className="text-sm text-muted-foreground bg-muted/50 p-3 rounded-md">
              Duration:{" "}
              {Math.round(
                (endTime.getTime() - startTime.getTime()) / 1000 / 60,
              )}{" "}
              minutes
            </div>
          )}

          {/* Project Selection */}
          <div className="space-y-2">
            <Label htmlFor="project">Project</Label>
            <Select
              value={watch("project_id") || ""}
              onValueChange={(value) => setValue("project_id", value)}
              disabled={loading || submitting}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select a project (optional)" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">No project</SelectItem>
                {/* TODO: Load actual projects from API */}
                <SelectItem value="project-1">Sample Project</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Task Selection */}
          <div className="space-y-2">
            <Label htmlFor="task">Task</Label>
            <Select
              value={watch("task_id") || ""}
              onValueChange={(value) => setValue("task_id", value)}
              disabled={loading || submitting}
            >
              <SelectTrigger>
                <SelectValue placeholder="Select a task (optional)" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="none">No task</SelectItem>
                {/* TODO: Load actual tasks from API */}
                <SelectItem value="task-1">Sample Task</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <DialogFooter className="flex gap-3">
            <Button
              type="button"
              variant="outline"
              onClick={onClose}
              disabled={submitting}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={loading || submitting}
              className="min-w-[100px]"
            >
              {submitting ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  {isEditing ? "Updating..." : "Creating..."}
                </>
              ) : (
                <>{isEditing ? "Update" : "Create"} Entry</>
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
