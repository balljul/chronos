"use client";

import { useState } from "react";
import { CalendarIcon, Filter, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
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
import { Separator } from "@/components/ui/separator";
import { formatDate } from "@/lib/time-utils";
import type { TimeEntryFilters } from "@/types/time-entries";

interface TimeEntryFiltersProps {
  filters: TimeEntryFilters;
  onFiltersChange: (filters: TimeEntryFilters) => void;
  loading?: boolean;
}

export function TimeEntryFiltersComponent({
  filters,
  onFiltersChange,
  loading = false,
}: TimeEntryFiltersProps) {
  const [datePickerOpen, setDatePickerOpen] = useState<"start" | "end" | null>(
    null,
  );

  const handleFilterChange = (key: keyof TimeEntryFilters, value: any) => {
    onFiltersChange({
      ...filters,
      [key]: value === "" || value === "all" ? undefined : value,
    });
  };

  const clearFilters = () => {
    onFiltersChange({
      project_id: undefined,
      task_id: undefined,
      start_date: undefined,
      end_date: undefined,
      is_running: undefined,
    });
  };

  const hasActiveFilters = Object.values(filters).some(
    (value) => value !== undefined && value !== "",
  );

  const getDatePresets = () => [
    {
      label: "Today",
      value: () => {
        const today = new Date();
        today.setHours(0, 0, 0, 0);
        const endOfDay = new Date();
        endOfDay.setHours(23, 59, 59, 999);
        return {
          start_date: today.toISOString(),
          end_date: endOfDay.toISOString(),
        };
      },
    },
    {
      label: "Yesterday",
      value: () => {
        const yesterday = new Date();
        yesterday.setDate(yesterday.getDate() - 1);
        yesterday.setHours(0, 0, 0, 0);
        const endOfYesterday = new Date(yesterday);
        endOfYesterday.setHours(23, 59, 59, 999);
        return {
          start_date: yesterday.toISOString(),
          end_date: endOfYesterday.toISOString(),
        };
      },
    },
    {
      label: "This Week",
      value: () => {
        const today = new Date();
        const firstDay = new Date(
          today.setDate(today.getDate() - today.getDay()),
        );
        firstDay.setHours(0, 0, 0, 0);
        const endOfWeek = new Date();
        endOfWeek.setHours(23, 59, 59, 999);
        return {
          start_date: firstDay.toISOString(),
          end_date: endOfWeek.toISOString(),
        };
      },
    },
    {
      label: "Last Week",
      value: () => {
        const today = new Date();
        const lastWeekStart = new Date(
          today.setDate(today.getDate() - today.getDay() - 7),
        );
        lastWeekStart.setHours(0, 0, 0, 0);
        const lastWeekEnd = new Date(lastWeekStart);
        lastWeekEnd.setDate(lastWeekEnd.getDate() + 6);
        lastWeekEnd.setHours(23, 59, 59, 999);
        return {
          start_date: lastWeekStart.toISOString(),
          end_date: lastWeekEnd.toISOString(),
        };
      },
    },
    {
      label: "This Month",
      value: () => {
        const today = new Date();
        const firstDay = new Date(today.getFullYear(), today.getMonth(), 1);
        const endOfMonth = new Date(
          today.getFullYear(),
          today.getMonth() + 1,
          0,
        );
        endOfMonth.setHours(23, 59, 59, 999);
        return {
          start_date: firstDay.toISOString(),
          end_date: endOfMonth.toISOString(),
        };
      },
    },
    {
      label: "Last Month",
      value: () => {
        const today = new Date();
        const firstDay = new Date(today.getFullYear(), today.getMonth() - 1, 1);
        const lastDay = new Date(today.getFullYear(), today.getMonth(), 0);
        lastDay.setHours(23, 59, 59, 999);
        return {
          start_date: firstDay.toISOString(),
          end_date: lastDay.toISOString(),
        };
      },
    },
  ];

  const applyDatePreset = (preset: {
    start_date: string;
    end_date: string;
  }) => {
    onFiltersChange({
      ...filters,
      start_date: preset.start_date,
      end_date: preset.end_date,
    });
  };

  return (
    <Card>
      <CardContent className="p-4">
        <div className="space-y-4">
          {/* Header with clear filters */}
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Filter className="h-4 w-4 text-muted-foreground" />
              <span className="font-medium text-sm">Filters</span>
              {hasActiveFilters && (
                <Badge variant="secondary" className="ml-2">
                  Active
                </Badge>
              )}
            </div>
            {hasActiveFilters && (
              <Button
                variant="ghost"
                size="sm"
                onClick={clearFilters}
                disabled={loading}
                className="h-8 px-2"
              >
                <X className="h-3 w-3 mr-1" />
                Clear
              </Button>
            )}
          </div>

          {/* Date Range */}
          <div className="space-y-3">
            <div className="text-sm font-medium">Date Range</div>

            {/* Date Presets */}
            <div className="flex flex-wrap gap-2">
              {getDatePresets().map((preset) => (
                <Button
                  key={preset.label}
                  variant="outline"
                  size="sm"
                  onClick={() => applyDatePreset(preset.value())}
                  disabled={loading}
                  className="h-8 text-xs"
                >
                  {preset.label}
                </Button>
              ))}
            </div>

            {/* Custom Date Range */}
            <div className="grid grid-cols-2 gap-3">
              <div className="space-y-2">
                <label className="text-xs text-muted-foreground">From</label>
                <Popover
                  open={datePickerOpen === "start"}
                  onOpenChange={(open) =>
                    setDatePickerOpen(open ? "start" : null)
                  }
                >
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      className="w-full justify-start text-left font-normal"
                      disabled={loading}
                    >
                      <CalendarIcon className="mr-2 h-4 w-4" />
                      {filters.start_date ? (
                        formatDate(filters.start_date)
                      ) : (
                        <span className="text-muted-foreground">
                          Pick a date
                        </span>
                      )}
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-0">
                    <Calendar
                      mode="single"
                      selected={
                        filters.start_date
                          ? new Date(filters.start_date)
                          : undefined
                      }
                      onSelect={(date) => {
                        handleFilterChange("start_date", date?.toISOString());
                        setDatePickerOpen(null);
                      }}
                      disabled={(date) =>
                        date > new Date() ||
                        (filters.end_date && date > new Date(filters.end_date))
                      }
                      initialFocus
                    />
                  </PopoverContent>
                </Popover>
              </div>

              <div className="space-y-2">
                <label className="text-xs text-muted-foreground">To</label>
                <Popover
                  open={datePickerOpen === "end"}
                  onOpenChange={(open) =>
                    setDatePickerOpen(open ? "end" : null)
                  }
                >
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      className="w-full justify-start text-left font-normal"
                      disabled={loading}
                    >
                      <CalendarIcon className="mr-2 h-4 w-4" />
                      {filters.end_date ? (
                        formatDate(filters.end_date)
                      ) : (
                        <span className="text-muted-foreground">
                          Pick a date
                        </span>
                      )}
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-0">
                    <Calendar
                      mode="single"
                      selected={
                        filters.end_date
                          ? new Date(filters.end_date)
                          : undefined
                      }
                      onSelect={(date) => {
                        handleFilterChange("end_date", date?.toISOString());
                        setDatePickerOpen(null);
                      }}
                      disabled={(date) =>
                        date > new Date() ||
                        (filters.start_date &&
                          date < new Date(filters.start_date))
                      }
                      initialFocus
                    />
                  </PopoverContent>
                </Popover>
              </div>
            </div>
          </div>

          <Separator />

          {/* Project and Task Filters */}
          <div className="grid grid-cols-2 gap-3">
            <div className="space-y-2">
              <label className="text-xs text-muted-foreground">Project</label>
              <Select
                value={filters.project_id || ""}
                onValueChange={(value) =>
                  handleFilterChange("project_id", value)
                }
                disabled={loading}
              >
                <SelectTrigger>
                  <SelectValue placeholder="All projects" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All projects</SelectItem>
                  {/* TODO: Load actual projects from API */}
                  <SelectItem value="project-1">Sample Project</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="space-y-2">
              <label className="text-xs text-muted-foreground">Task</label>
              <Select
                value={filters.task_id || ""}
                onValueChange={(value) => handleFilterChange("task_id", value)}
                disabled={loading}
              >
                <SelectTrigger>
                  <SelectValue placeholder="All tasks" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">All tasks</SelectItem>
                  {/* TODO: Load actual tasks from API */}
                  <SelectItem value="task-1">Sample Task</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>

          <Separator />

          {/* Status Filter */}
          <div className="space-y-2">
            <label className="text-xs text-muted-foreground">Status</label>
            <Select
              value={filters.is_running?.toString() || ""}
              onValueChange={(value) =>
                handleFilterChange(
                  "is_running",
                  value === "" ? undefined : value === "true",
                )
              }
              disabled={loading}
            >
              <SelectTrigger>
                <SelectValue placeholder="All entries" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All entries</SelectItem>
                <SelectItem value="true">Running only</SelectItem>
                <SelectItem value="false">Completed only</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Active Filters Summary */}
          {hasActiveFilters && (
            <>
              <Separator />
              <div className="space-y-2">
                <div className="text-xs text-muted-foreground">
                  Active Filters:
                </div>
                <div className="flex flex-wrap gap-2">
                  {filters.start_date && (
                    <Badge variant="secondary" className="text-xs">
                      From: {formatDate(filters.start_date)}
                      <button
                        onClick={() =>
                          handleFilterChange("start_date", undefined)
                        }
                        className="ml-1 hover:bg-secondary-foreground/10 rounded-full p-0.5"
                      >
                        <X className="h-2.5 w-2.5" />
                      </button>
                    </Badge>
                  )}
                  {filters.end_date && (
                    <Badge variant="secondary" className="text-xs">
                      To: {formatDate(filters.end_date)}
                      <button
                        onClick={() =>
                          handleFilterChange("end_date", undefined)
                        }
                        className="ml-1 hover:bg-secondary-foreground/10 rounded-full p-0.5"
                      >
                        <X className="h-2.5 w-2.5" />
                      </button>
                    </Badge>
                  )}
                  {filters.project_id && (
                    <Badge variant="secondary" className="text-xs">
                      Project: Sample Project
                      <button
                        onClick={() =>
                          handleFilterChange("project_id", undefined)
                        }
                        className="ml-1 hover:bg-secondary-foreground/10 rounded-full p-0.5"
                      >
                        <X className="h-2.5 w-2.5" />
                      </button>
                    </Badge>
                  )}
                  {filters.task_id && (
                    <Badge variant="secondary" className="text-xs">
                      Task: Sample Task
                      <button
                        onClick={() => handleFilterChange("task_id", undefined)}
                        className="ml-1 hover:bg-secondary-foreground/10 rounded-full p-0.5"
                      >
                        <X className="h-2.5 w-2.5" />
                      </button>
                    </Badge>
                  )}
                  {filters.is_running !== undefined && (
                    <Badge variant="secondary" className="text-xs">
                      {filters.is_running ? "Running" : "Completed"}
                      <button
                        onClick={() =>
                          handleFilterChange("is_running", undefined)
                        }
                        className="ml-1 hover:bg-secondary-foreground/10 rounded-full p-0.5"
                      >
                        <X className="h-2.5 w-2.5" />
                      </button>
                    </Badge>
                  )}
                </div>
              </div>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
