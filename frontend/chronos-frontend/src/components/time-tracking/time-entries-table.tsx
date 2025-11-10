"use client";

import { useState } from "react";
import { MoreHorizontal, Edit, Trash2, Play, Clock } from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Skeleton } from "@/components/ui/skeleton";
import { formatDate, formatTime, formatDuration } from "@/lib/time-utils";
import type { TimeEntry } from "@/types/time-entries";

interface TimeEntriesTableProps {
  entries: TimeEntry[];
  loading: boolean;
  totalCount: number;
  totalDuration: number;
  page: number;
  totalPages: number;
  hasNextPage: boolean;
  hasPreviousPage: boolean;
  onEdit: (entry: TimeEntry) => void;
  onDelete: (entry: TimeEntry) => void;
  onPageChange: (page: number) => void;
  onDuplicate?: (entry: TimeEntry) => void;
}

export function TimeEntriesTable({
  entries,
  loading,
  totalCount,
  totalDuration,
  page,
  totalPages,
  hasNextPage,
  hasPreviousPage,
  onEdit,
  onDelete,
  onPageChange,
  onDuplicate,
}: TimeEntriesTableProps) {
  const [processingId, setProcessingId] = useState<string | null>(null);

  const handleAction = async (entryId: string, action: () => Promise<void>) => {
    setProcessingId(entryId);
    try {
      await action();
    } finally {
      setProcessingId(null);
    }
  };

  if (loading && entries.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Time Entries
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            {[1, 2, 3, 4, 5].map((i) => (
              <div key={i} className="flex items-center space-x-4">
                <Skeleton className="h-4 w-20" />
                <Skeleton className="h-4 w-40" />
                <Skeleton className="h-4 w-24" />
                <Skeleton className="h-4 w-16" />
                <Skeleton className="h-4 w-16" />
                <Skeleton className="h-8 w-8" />
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    );
  }

  const isEmpty = entries.length === 0 && !loading;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Time Entries
            {totalCount > 0 && (
              <Badge variant="secondary" className="ml-2">
                {totalCount}
              </Badge>
            )}
          </CardTitle>
          {totalDuration > 0 && (
            <div className="text-sm text-muted-foreground">
              Total:{" "}
              <span className="font-medium">
                {formatDuration(totalDuration)}
              </span>
            </div>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {isEmpty ? (
          <div className="text-center py-12">
            <Clock className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
            <h3 className="text-lg font-semibold mb-2">No time entries yet</h3>
            <p className="text-muted-foreground mb-4">
              Start tracking your time by clicking the timer above or create a
              manual entry.
            </p>
            <Button onClick={() => onEdit({} as TimeEntry)}>
              Create Manual Entry
            </Button>
          </div>
        ) : (
          <>
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Date</TableHead>
                    <TableHead>Description</TableHead>
                    <TableHead>Project</TableHead>
                    <TableHead>Start Time</TableHead>
                    <TableHead>End Time</TableHead>
                    <TableHead>Duration</TableHead>
                    <TableHead className="w-[50px]"></TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {entries.map((entry) => {
                    const isRunning = entry.is_running || !entry.end_time;
                    const isProcessing = processingId === entry.id;

                    return (
                      <TableRow
                        key={entry.id}
                        className={
                          isRunning
                            ? "bg-green-50 dark:bg-green-950/10"
                            : undefined
                        }
                      >
                        <TableCell className="font-medium">
                          {formatDate(entry.start_time)}
                        </TableCell>

                        <TableCell>
                          <div className="max-w-xs">
                            {entry.description ? (
                              <span className="truncate block">
                                {entry.description}
                              </span>
                            ) : (
                              <span className="text-muted-foreground italic">
                                No description
                              </span>
                            )}
                          </div>
                        </TableCell>

                        <TableCell>
                          {entry.project_id ? (
                            <Badge variant="outline">
                              Project Name{" "}
                              {/* TODO: Load actual project name */}
                            </Badge>
                          ) : (
                            <span className="text-muted-foreground text-sm">
                              No project
                            </span>
                          )}
                        </TableCell>

                        <TableCell>{formatTime(entry.start_time)}</TableCell>

                        <TableCell>
                          {entry.end_time ? (
                            formatTime(entry.end_time)
                          ) : (
                            <Badge
                              variant="secondary"
                              className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
                            >
                              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse mr-1" />
                              Running
                            </Badge>
                          )}
                        </TableCell>

                        <TableCell>
                          <span className="font-mono text-sm">
                            {entry.duration ? (
                              formatDuration(entry.duration)
                            ) : isRunning ? (
                              <span className="text-muted-foreground">
                                Live
                              </span>
                            ) : (
                              <span className="text-muted-foreground">0m</span>
                            )}
                          </span>
                        </TableCell>

                        <TableCell>
                          <DropdownMenu>
                            <DropdownMenuTrigger asChild>
                              <Button
                                variant="ghost"
                                size="sm"
                                className="h-8 w-8 p-0"
                                disabled={isProcessing}
                              >
                                <MoreHorizontal className="h-4 w-4" />
                                <span className="sr-only">Open menu</span>
                              </Button>
                            </DropdownMenuTrigger>
                            <DropdownMenuContent align="end">
                              <DropdownMenuItem onClick={() => onEdit(entry)}>
                                <Edit className="mr-2 h-4 w-4" />
                                Edit
                              </DropdownMenuItem>

                              {onDuplicate && (
                                <DropdownMenuItem
                                  onClick={() => onDuplicate(entry)}
                                >
                                  <Play className="mr-2 h-4 w-4" />
                                  Duplicate
                                </DropdownMenuItem>
                              )}

                              <DropdownMenuSeparator />

                              <DropdownMenuItem
                                onClick={() =>
                                  handleAction(entry.id, () =>
                                    Promise.resolve(onDelete(entry)),
                                  )
                                }
                                className="text-red-600 dark:text-red-400"
                                disabled={isProcessing}
                              >
                                <Trash2 className="mr-2 h-4 w-4" />
                                Delete
                              </DropdownMenuItem>
                            </DropdownMenuContent>
                          </DropdownMenu>
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </div>

            {/* Pagination */}
            {totalPages > 1 && (
              <div className="flex items-center justify-between px-2 py-4">
                <div className="text-sm text-muted-foreground">
                  Page {page} of {totalPages}
                </div>

                <div className="flex items-center space-x-2">
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onPageChange(page - 1)}
                    disabled={!hasPreviousPage || loading}
                  >
                    Previous
                  </Button>

                  {/* Page numbers */}
                  <div className="flex items-center space-x-1">
                    {Array.from({ length: Math.min(5, totalPages) }, (_, i) => {
                      let pageNum;
                      if (totalPages <= 5) {
                        pageNum = i + 1;
                      } else if (page <= 3) {
                        pageNum = i + 1;
                      } else if (page >= totalPages - 2) {
                        pageNum = totalPages - 4 + i;
                      } else {
                        pageNum = page - 2 + i;
                      }

                      return (
                        <Button
                          key={pageNum}
                          variant={pageNum === page ? "default" : "outline"}
                          size="sm"
                          onClick={() => onPageChange(pageNum)}
                          disabled={loading}
                          className="w-8 h-8 p-0"
                        >
                          {pageNum}
                        </Button>
                      );
                    })}
                  </div>

                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => onPageChange(page + 1)}
                    disabled={!hasNextPage || loading}
                  >
                    Next
                  </Button>
                </div>
              </div>
            )}
          </>
        )}

        {loading && entries.length > 0 && (
          <div className="flex justify-center py-4">
            <div className="text-sm text-muted-foreground">Loading...</div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
