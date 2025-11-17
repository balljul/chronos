"use client";

import { useEffect, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Calendar, ChevronLeft, ChevronRight, Edit, Filter, Plus, Search, Trash2, X } from "lucide-react";
import { formatTimerDuration, formatDuration } from "@/lib/time-utils";
import { timeEntriesAPI } from "@/lib/api/time-entries";
import { toast } from "sonner";
import { confirmAlert } from "react-confirm-alert";
import "react-confirm-alert/src/react-confirm-alert.css";
import type { TimeEntry, TimeEntriesListResponse, UpdateTimeEntryRequest } from "@/types/time-entries";

interface TimeEntriesTableProps {
  onRefresh?: () => void;
  onAddEntry?: () => void;
}

export default function TimeEntriesTable({ onRefresh, onAddEntry }: TimeEntriesTableProps) {
  const [data, setData] = useState<TimeEntriesListResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [page, setPage] = useState(1);
  const [isDeleting, setIsDeleting] = useState<string | null>(null);
  const [searchTerm, setSearchTerm] = useState("");
  const [dateFilter, setDateFilter] = useState("");
  const [projectFilter, setProjectFilter] = useState("");
  const [showFilters, setShowFilters] = useState(false);
  const [editingEntry, setEditingEntry] = useState<TimeEntry | null>(null);
  const [isUpdating, setIsUpdating] = useState(false);
  const perPage = 10;

  const buildFilters = () => {
    const filters: any = {
      page,
      limit: perPage,
      sort_by: "start_time",
    };

    if (searchTerm.trim()) {
      filters.search = searchTerm.trim();
    }

    if (projectFilter && projectFilter !== "all") {
      filters.project_id = projectFilter;
    }

    if (dateFilter) {
      const today = new Date();
      let start_date: string | undefined;
      let end_date: string | undefined;

      switch (dateFilter) {
        case "today":
          start_date = today.toISOString().split("T")[0];
          end_date = start_date;
          break;
        case "yesterday":
          const yesterday = new Date(today);
          yesterday.setDate(today.getDate() - 1);
          start_date = yesterday.toISOString().split("T")[0];
          end_date = start_date;
          break;
        case "this_week":
          const startOfWeek = new Date(today);
          startOfWeek.setDate(today.getDate() - today.getDay());
          start_date = startOfWeek.toISOString().split("T")[0];
          end_date = today.toISOString().split("T")[0];
          break;
        case "this_month":
          start_date = new Date(today.getFullYear(), today.getMonth(), 1).toISOString().split("T")[0];
          end_date = today.toISOString().split("T")[0];
          break;
      }

      if (start_date) filters.start_date = start_date;
      if (end_date) filters.end_date = end_date;
    }

    return filters;
  };

  const loadTimeEntries = async (pageNum: number = page) => {
    try {
      setLoading(true);
      const filters = buildFilters();
      filters.page = pageNum;
      const response = await timeEntriesAPI.getTimeEntries(filters);
      setData(response);
    } catch (error) {
      console.error("Failed to load time entries:", error);
      toast.error("Failed to load time entries");
    } finally {
      setLoading(false);
    }
  };

  const clearFilters = () => {
    setSearchTerm("");
    setDateFilter("");
    setProjectFilter("");
    setPage(1);
  };

  const hasActiveFilters = searchTerm || dateFilter || projectFilter;

  const handleEdit = (entry: TimeEntry) => {
    setEditingEntry(entry);
  };

  const handleUpdateEntry = async (updatedData: UpdateTimeEntryRequest) => {
    if (!editingEntry) return;

    try {
      setIsUpdating(true);
      await timeEntriesAPI.updateTimeEntry(editingEntry.id, updatedData);
      toast.success("Time entry updated");
      await loadTimeEntries();
      onRefresh?.();
      setEditingEntry(null);
    } catch (error) {
      console.error("Failed to update time entry:", error);
      toast.error("Failed to update time entry");
    } finally {
      setIsUpdating(false);
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
    <div className="w-full">
      {/* Header */}
      <div className="px-4 py-3 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
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
              <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">
                Description
              </th>
              <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">
                Date
              </th>
              <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">
                Start
              </th>
              <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">
                End
              </th>
              <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">
                Duration
              </th>
              <th className="px-3 py-2 text-left text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">
                Status
              </th>
              <th className="px-3 py-2 text-right text-xs font-medium text-gray-500 dark:text-gray-300 uppercase">
                Actions
              </th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-200 dark:divide-gray-600 bg-white dark:bg-gray-800">
            {data.entries.map((entry) => (
              <tr key={entry.id} className="hover:bg-gray-50 dark:hover:bg-gray-700">
                <td className="px-3 py-2">
                  <div className="text-sm font-medium text-gray-900 dark:text-white">
                    {entry.description || "No description"}
                  </div>
                </td>
                <td className="px-3 py-2">
                  <div className="text-sm text-gray-900 dark:text-white">
                    {formatDate(entry.start_time)}
                  </div>
                </td>
                <td className="px-3 py-2">
                  <div className="text-sm text-gray-900 dark:text-white">
                    {formatTime(entry.start_time)}
                  </div>
                </td>
                <td className="px-3 py-2">
                  <div className="text-sm text-gray-900 dark:text-white">
                    {entry.end_time ? formatTime(entry.end_time) : (
                      <span className="text-gray-400 dark:text-gray-500">—</span>
                    )}
                  </div>
                </td>
                <td className="px-3 py-2">
                  <div className="text-sm font-medium text-gray-900 dark:text-white">
                    {entry.duration ? formatDuration(entry.duration) : (
                      entry.is_running ? (
                        <span className="text-green-600 dark:text-green-400">Running</span>
                      ) : (
                        <span className="text-gray-400 dark:text-gray-500">—</span>
                      )
                    )}
                  </div>
                </td>
                <td className="px-3 py-2">
                  <Badge
                    variant={entry.is_running ? "default" : "secondary"}
                    className={entry.is_running ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-300" : ""}
                  >
                    {entry.is_running ? "Running" : "Completed"}
                  </Badge>
                </td>
                <td className="px-3 py-2 text-right">
                  <div className="flex justify-end space-x-1">
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleEdit(entry)}
                      disabled={entry.is_running}
                      title={entry.is_running ? "Cannot edit running timer" : "Edit time entry"}
                      className="h-8 w-8 p-0"
                    >
                      <Edit className="h-4 w-4" />
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleDelete(entry.id, entry.description)}
                      disabled={isDeleting === entry.id}
                      title="Delete time entry"
                      className="h-8 w-8 p-0"
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="px-4 py-3 border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800">
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

      {/* Edit Modal */}
      {editingEntry && (
        <EditTimeEntryModal
          entry={editingEntry}
          onClose={() => setEditingEntry(null)}
          onUpdate={handleUpdateEntry}
          isUpdating={isUpdating}
        />
      )}
    </div>
  );
}

interface EditTimeEntryModalProps {
  entry: TimeEntry;
  onClose: () => void;
  onUpdate: (data: UpdateTimeEntryRequest) => void;
  isUpdating: boolean;
}

function EditTimeEntryModal({ entry, onClose, onUpdate, isUpdating }: EditTimeEntryModalProps) {
  const [description, setDescription] = useState(entry.description || "");
  const [startTime, setStartTime] = useState(() => {
    const date = new Date(entry.start_time);
    return date.toISOString().slice(0, 16);
  });
  const [endTime, setEndTime] = useState(() => {
    if (entry.end_time) {
      const date = new Date(entry.end_time);
      return date.toISOString().slice(0, 16);
    }
    return "";
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    
    const updateData: UpdateTimeEntryRequest = {
      description: description.trim() || undefined,
      start_time: new Date(startTime).toISOString(),
    };

    if (endTime) {
      updateData.end_time = new Date(endTime).toISOString();
    }

    onUpdate(updateData);
  };

  const formatDateTimeForInput = (isoString: string) => {
    return new Date(isoString).toISOString().slice(0, 16);
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 p-6 rounded-lg shadow-xl max-w-md w-full mx-4 border border-gray-200 dark:border-gray-700">
        <h2 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
          Edit Time Entry
        </h2>
        
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Description
            </label>
            <Input
              type="text"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder="Enter description"
              className="w-full"
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              Start Time
            </label>
            <Input
              type="datetime-local"
              value={startTime}
              onChange={(e) => setStartTime(e.target.value)}
              className="w-full"
              required
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
              End Time
            </label>
            <Input
              type="datetime-local"
              value={endTime}
              onChange={(e) => setEndTime(e.target.value)}
              className="w-full"
              placeholder="Leave empty if still running"
            />
          </div>

          <div className="flex justify-end space-x-2 pt-4">
            <Button
              type="button"
              variant="outline"
              onClick={onClose}
              disabled={isUpdating}
            >
              Cancel
            </Button>
            <Button
              type="submit"
              disabled={isUpdating || !startTime}
            >
              {isUpdating ? "Updating..." : "Update Entry"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}