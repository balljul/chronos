"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState } from "react";
import { Plus, CheckSquare, MoreHorizontal, Edit, Archive, Trash2, Filter } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Badge } from "@/components/ui/badge";
import { toast } from "sonner";
import { tasksAPI, type Task } from "@/lib/api/tasks";
import { projectsAPI, type Project } from "@/lib/api/projects";
import TaskForm from "@/components/tasks/task-form";

export default function TasksPage() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [authLoading, setAuthLoading] = useState(true);
  const [error, setError] = useState("");
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [includeInactive, setIncludeInactive] = useState(false);
  const [filterProjectId, setFilterProjectId] = useState<string>("all");
  const router = useRouter();

  const getAuthToken = useCallback(() => {
    if (typeof window === "undefined") {
      return null;
    }
    return localStorage.getItem("access_token");
  }, []);

  const checkAuth = useCallback(async () => {
    const token = getAuthToken();
    if (!token) {
      router.push("/login");
      return false;
    }
    return true;
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
      setLoading(true);
      let data: Task[];

      if (filterProjectId === "none") {
        data = await tasksAPI.getTasksWithoutProject(includeInactive);
      } else if (filterProjectId === "all") {
        data = await tasksAPI.getTasks({ include_inactive: includeInactive });
      } else {
        data = await tasksAPI.getTasksByProject(filterProjectId, includeInactive);
      }

      setTasks(data);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to load tasks";
      if (errorMessage.includes("401")) {
        router.push("/login");
        return;
      }
      setError(errorMessage);
      toast.error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [includeInactive, filterProjectId, router]);

  const handleCreateTask = async (data: { name: string; description?: string; project_id?: string }) => {
    try {
      await tasksAPI.createTask(data);
      toast.success("Task created successfully");
      setShowCreateDialog(false);
      await loadTasks();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to create task";
      toast.error(errorMessage);
    }
  };

  const handleUpdateTask = async (data: { name: string; description?: string; project_id?: string }) => {
    if (!editingTask) return;

    try {
      await tasksAPI.updateTask(editingTask.id, data);
      toast.success("Task updated successfully");
      setEditingTask(null);
      await loadTasks();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to update task";
      toast.error(errorMessage);
    }
  };

  const handleArchiveTask = async (task: Task) => {
    try {
      if (task.is_active) {
        await tasksAPI.archiveTask(task.id);
        toast.success("Task archived successfully");
      } else {
        await tasksAPI.restoreTask(task.id);
        toast.success("Task restored successfully");
      }
      await loadTasks();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to update task";
      toast.error(errorMessage);
    }
  };

  const handleDeleteTask = async (task: Task) => {
    if (!confirm("Are you sure you want to permanently delete this task? This action cannot be undone.")) {
      return;
    }

    try {
      await tasksAPI.deleteTask(task.id, true);
      toast.success("Task deleted successfully");
      await loadTasks();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to delete task";
      toast.error(errorMessage);
    }
  };

  const getProjectName = (projectId?: string) => {
    if (!projectId) return "No Project";
    const project = projects.find(p => p.id === projectId);
    return project?.name || "Unknown Project";
  };

  const getProjectColor = (projectId?: string) => {
    if (!projectId) return undefined;
    const project = projects.find(p => p.id === projectId);
    return project?.color;
  };

  useEffect(() => {
    const initPage = async () => {
      setAuthLoading(true);
      const isAuthenticated = await checkAuth();
      setAuthLoading(false);

      if (isAuthenticated) {
        await loadProjects();
        await loadTasks();
      }
    };

    initPage();
  }, [checkAuth, loadProjects, loadTasks]);

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-600 mx-auto"></div>
          <p className="mt-4 text-gray-600 dark:text-gray-400">
            Loading tasks...
          </p>
        </div>
      </div>
    );
  }

  if (error && !loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <p className="text-red-600 dark:text-red-400">{error}</p>
          <Button
            onClick={() => router.push("/dashboard")}
            className="mt-4"
          >
            Back to Dashboard
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-white dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-4">
            <div>
              <h1 className="text-2xl font-semibold text-gray-900 dark:text-white">
                Tasks
              </h1>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Manage your time tracking tasks
              </p>
            </div>

            <div className="flex items-center space-x-3">
              <div className="flex items-center space-x-2">
                <Filter className="h-4 w-4 text-gray-500" />
                <Select value={filterProjectId} onValueChange={setFilterProjectId}>
                  <SelectTrigger className="w-48">
                    <SelectValue placeholder="Filter by project" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="all">All Projects</SelectItem>
                    <SelectItem value="none">No Project</SelectItem>
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
              </div>

              <Button
                variant="outline"
                onClick={() => setIncludeInactive(!includeInactive)}
              >
                {includeInactive ? "Hide Archived" : "Show Archived"}
              </Button>

              <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
                <DialogTrigger asChild>
                  <Button>
                    <Plus className="h-4 w-4 mr-2" />
                    New Task
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Create New Task</DialogTitle>
                  </DialogHeader>
                  <TaskForm projects={projects} onSubmit={handleCreateTask} />
                </DialogContent>
              </Dialog>

              <Button
                variant="outline"
                onClick={() => router.push("/dashboard")}
              >
                Back to Dashboard
              </Button>
            </div>
          </div>
        </div>
      </header>

      {/* Main Content */}
      <main className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
        {loading ? (
          <div className="flex justify-center items-center py-12">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"></div>
          </div>
        ) : tasks.length === 0 ? (
          <div className="text-center py-12">
            <CheckSquare className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
              No tasks yet
            </h3>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              Create your first task to start organizing your work.
            </p>
            <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
              <DialogTrigger asChild>
                <Button>
                  <Plus className="h-4 w-4 mr-2" />
                  Create Your First Task
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Create New Task</DialogTitle>
                </DialogHeader>
                <TaskForm projects={projects} onSubmit={handleCreateTask} />
              </DialogContent>
            </Dialog>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {tasks.map((task) => (
              <Card key={task.id} className={`relative ${!task.is_active ? 'opacity-60' : ''}`}>
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <CardTitle className="text-lg">{task.name}</CardTitle>

                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" className="h-8 w-8 p-0">
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={() => setEditingTask(task)}>
                          <Edit className="h-4 w-4 mr-2" />
                          Edit
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleArchiveTask(task)}>
                          <Archive className="h-4 w-4 mr-2" />
                          {task.is_active ? "Archive" : "Restore"}
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onClick={() => handleDeleteTask(task)}
                          className="text-red-600"
                        >
                          <Trash2 className="h-4 w-4 mr-2" />
                          Delete
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>

                  <div className="flex items-center space-x-2">
                    <Badge variant={task.is_active ? "default" : "secondary"}>
                      {task.is_active ? "Active" : "Archived"}
                    </Badge>
                    {task.project_id && (
                      <div className="flex items-center space-x-1 text-sm text-gray-600 dark:text-gray-400">
                        {getProjectColor(task.project_id) && (
                          <div
                            className="w-2 h-2 rounded-full"
                            style={{ backgroundColor: getProjectColor(task.project_id) }}
                          />
                        )}
                        <span>{getProjectName(task.project_id)}</span>
                      </div>
                    )}
                  </div>
                </CardHeader>

                <CardContent>
                  {task.description && (
                    <CardDescription className="mb-4">
                      {task.description}
                    </CardDescription>
                  )}

                  <div className="text-sm text-gray-500 dark:text-gray-400">
                    Created: {new Date(task.created_at).toLocaleDateString()}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}

        {/* Edit Task Dialog */}
        <Dialog open={!!editingTask} onOpenChange={() => setEditingTask(null)}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Edit Task</DialogTitle>
            </DialogHeader>
            {editingTask && (
              <TaskForm
                projects={projects}
                initialData={editingTask}
                onSubmit={handleUpdateTask}
              />
            )}
          </DialogContent>
        </Dialog>
      </main>
    </div>
  );
}