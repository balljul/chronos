"use client";

import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ChevronLeft, ChevronRight, Edit, Play, Trash2 } from "lucide-react";
import { formatTimerDuration } from "@/lib/time-utils";
import { timeEntriesAPI } from "@/lib/api/time-entries";
import { toast } from "sonner";
import { confirmAlert } from "react-confirm-alert";
import "react-confirm-alert/src/react-confirm-alert.css";
import type { TimeEntry, TimeEntriesListResponse } from "@/types/time-entries";

interface TimeEntriesTableProps {
  onRefresh?: () => void;
}

export default function TimeEntriesTable({ onRefresh }: TimeEntriesTableProps) {
  const [data, setData] = useState<TimeEntriesListResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [isDeleting, setIsDeleting] = useState<string | null>(null);
  const perPage = 10;

  const loadTimeEntries = async (pageNum: number = page) => {
    try {
      setLoading(true);
      const response = await timeEntriesAPI.getTimeEntries({
        page: pageNum,
        limit: perPage,
        sort_by: "start_time",
      });
      setData(response);
    } catch (error) {
      console.error("Failed to load time entries:", error);
      toast.error("Failed to load time entries");
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = (id: string, description?: string) => {
    confirmAlert({
      title: "Delete Time Entry",
      message: `Are you sure you want to delete ${
        description ? `"${description}"` : "this time entry"
      }? This action cannot be undone.`,
      closeOnEscape: true,
      closeOnClickOutside: true,
      customUI: ({ onClose, title, message }) => (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-xl max-w-md w-full mx-4">
            <h2 className="text-lg font-semibold mb-2 text-gray-900 dark:text-white">
              {title}
            </h2>
            <p className="text-gray-600 dark:text-gray-300 mb-6">{message}</p>
            <div className="flex justify-end space-x-2">
              <button
                onClick={onClose}
                className="px-4 py-2 text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={async () => {
                  try {
                    setIsDeleting(id);
                    await timeEntriesAPI.deleteTimeEntry(id);
                    toast.success("Time entry deleted");
                    await loadTimeEntries();
                    onRefresh?.();
                  } catch (error) {
                    console.error("Failed to delete time entry:", error);
                    toast.error("Failed to delete time entry");
                  } finally {
                    setIsDeleting(null);
                  }
                  onClose();
                }}
                className="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
              >
                Delete
              </button>
            </div>
          </div>
        </div>
      ),
    });
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString("en-US", {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  };

  const formatTime = (dateString: string) => {
    return new Date(dateString).toLocaleTimeString("en-US", {
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const goToPage = (pageNum: number) => {
    setPage(pageNum);
    loadTimeEntries(pageNum);
  };

  useEffect(() => {
    loadTimeEntries();
  }, []);

  if (loading && !data) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"></div>
      </div>
    );
  }

  if (!data || data.entries.length === 0) {
    return (
      <div className="text-center p-8 text-gray-500 dark:text-gray-400">
        <p>No time entries found.</p>
        <p className="text-sm mt-2">Start tracking your time to see entries here.</p>
      </div>
    );
  }

  const totalPages = Math.ceil(data.total_count / perPage);

  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow-sm border border-gray-200 dark:border-gray-700">
      {/* Header */}
      <div className="px-6 py-4 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-lg font-medium text-gray-900 dark:text-white">
              Time Entries
            </h2>
            <p className="text-sm text-gray-500 dark:text-gray-400">
              {data.total_count} entries • Total: {formatTimerDuration(data.total_duration)}
            </p>
          </div>
          <Button
            onClick={() => loadTimeEntries()}
            variant="outline"
            size="sm"
            disabled={loading}
          >
            {loading ? "Loading..." : "Refresh"}
          </Button>
        </div>
      </div>

      {/* Table */}
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="bg-gray-50 dark:bg-gray-700">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                Description
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                Date
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                Time
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                Duration
              </th>
              <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                Status
              </th>
              <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase tracking-wider">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 dark:divide-gray-600">
            {data.entries.map((entry) => (
              <tr key={entry.id} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                <td className="px-6 py-4">
                  <div className="text-sm font-medium text-gray-900 dark:text-white">
                    {entry.description || "No description"}
                  </div>
                </td>
                <td className="px-6 py-4">
                  <div className="text-sm text-gray-900 dark:text-white">
                    {formatDate(entry.start_time)}
                  </div>
                </td>
                <td className="px-6 py-4">
                  <div className="text-sm text-gray-900 dark:text-white">
                    {formatTime(entry.start_time)}
                    {entry.end_time && (
                      <>
                        {" - "}
                        {formatTime(entry.end_time)}
                      </>
                    )}
                  </div>
                </td>
                <td className="px-6 py-4">
                  <div className="text-sm font-mono text-gray-900 dark:text-white">
                    {entry.duration ? formatTimerDuration(entry.duration) : "N/A"}
                  </div>
                </td>
                <td className="px-6 py-4">
                  <Badge
                    variant={entry.is_running ? "default" : "secondary"}
                    className={entry.is_running ? "bg-green-100 text-green-800" : ""}
                  >
                    {entry.is_running ? "Running" : "Completed"}
                  </Badge>
                </td>
                <td className="px-6 py-4 text-right space-x-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      // TODO: Implement edit functionality
                      toast.info("Edit functionality coming soon");
                    }}
                  >
                    <Edit className="h-4 w-4" />
                  </Button>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleDelete(entry.id, entry.description)}
                    disabled={isDeleting === entry.id}
                  >
                    <Trash2 className="h-4 w-4" />
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="px-6 py-4 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between">
            <div className="text-sm text-gray-700 dark:text-gray-300">
              Showing {(page - 1) * perPage + 1} to{" "}
              {Math.min(page * perPage, data.total_count)} of {data.total_count} entries
            </div>
            <div className="flex items-center space-x-2">
              <Button
                variant="outline"
                size="sm"
                onClick={() => goToPage(page - 1)}
                disabled={page <= 1 || loading}
              >
                <ChevronLeft className="h-4 w-4" />
                Previous
              </Button>
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
                      variant={page === pageNum ? "default" : "outline"}
                      size="sm"
                      onClick={() => goToPage(pageNum)}
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
                onClick={() => goToPage(page + 1)}
                disabled={page >= totalPages || loading}
              >
                Next
                <ChevronRight className="h-4 w-4" />
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}