"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState } from "react";
import { Plus, Folder, MoreHorizontal, Edit, Archive, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from "@/components/ui/dialog";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Badge } from "@/components/ui/badge";
import { toast } from "sonner";
import { projectsAPI, type Project } from "@/lib/api/projects";
import ProjectForm from "@/components/projects/project-form";

export default function ProjectsPage() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [loading, setLoading] = useState(true);
  const [authLoading, setAuthLoading] = useState(true);
  const [error, setError] = useState("");
  const [showCreateDialog, setShowCreateDialog] = useState(false);
  const [editingProject, setEditingProject] = useState<Project | null>(null);
  const [includeInactive, setIncludeInactive] = useState(false);
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
      setLoading(true);
      const data = await projectsAPI.getProjects({ include_inactive: includeInactive });
      setProjects(data);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to load projects";
      if (errorMessage.includes("401")) {
        router.push("/login");
        return;
      }
      setError(errorMessage);
      toast.error(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [includeInactive, router]);

  const handleCreateProject = async (data: { name: string; description?: string; color?: string }) => {
    try {
      await projectsAPI.createProject(data);
      toast.success("Project created successfully");
      setShowCreateDialog(false);
      await loadProjects();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to create project";
      toast.error(errorMessage);
    }
  };

  const handleUpdateProject = async (data: { name: string; description?: string; color?: string }) => {
    if (!editingProject) return;

    try {
      await projectsAPI.updateProject(editingProject.id, data);
      toast.success("Project updated successfully");
      setEditingProject(null);
      await loadProjects();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to update project";
      toast.error(errorMessage);
    }
  };

  const handleArchiveProject = async (project: Project) => {
    try {
      if (project.is_active) {
        await projectsAPI.archiveProject(project.id);
        toast.success("Project archived successfully");
      } else {
        await projectsAPI.restoreProject(project.id);
        toast.success("Project restored successfully");
      }
      await loadProjects();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to update project";
      toast.error(errorMessage);
    }
  };

  const handleDeleteProject = async (project: Project) => {
    if (!confirm("Are you sure you want to permanently delete this project? This action cannot be undone.")) {
      return;
    }

    try {
      await projectsAPI.deleteProject(project.id, true);
      toast.success("Project deleted successfully");
      await loadProjects();
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to delete project";
      toast.error(errorMessage);
    }
  };

  useEffect(() => {
    const initPage = async () => {
      setAuthLoading(true);
      const isAuthenticated = await checkAuth();
      setAuthLoading(false);

      if (isAuthenticated) {
        await loadProjects();
      }
    };

    initPage();
  }, [checkAuth, loadProjects]);

  if (authLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-600 mx-auto"></div>
          <p className="mt-4 text-gray-600 dark:text-gray-400">
            Loading projects...
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
                Projects
              </h1>
              <p className="text-sm text-gray-600 dark:text-gray-400">
                Manage your time tracking projects
              </p>
            </div>

            <div className="flex items-center space-x-3">
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
                    New Project
                  </Button>
                </DialogTrigger>
                <DialogContent>
                  <DialogHeader>
                    <DialogTitle>Create New Project</DialogTitle>
                  </DialogHeader>
                  <ProjectForm onSubmit={handleCreateProject} />
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
        ) : projects.length === 0 ? (
          <div className="text-center py-12">
            <Folder className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
              No projects yet
            </h3>
            <p className="text-gray-600 dark:text-gray-400 mb-4">
              Create your first project to start organizing your time tracking.
            </p>
            <Dialog open={showCreateDialog} onOpenChange={setShowCreateDialog}>
              <DialogTrigger asChild>
                <Button>
                  <Plus className="h-4 w-4 mr-2" />
                  Create Your First Project
                </Button>
              </DialogTrigger>
              <DialogContent>
                <DialogHeader>
                  <DialogTitle>Create New Project</DialogTitle>
                </DialogHeader>
                <ProjectForm onSubmit={handleCreateProject} />
              </DialogContent>
            </Dialog>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            {projects.map((project) => (
              <Card key={project.id} className={`relative ${!project.is_active ? 'opacity-60' : ''}`}>
                <CardHeader className="pb-2">
                  <div className="flex items-center justify-between">
                    <div className="flex items-center space-x-2">
                      {project.color && (
                        <div
                          className="w-3 h-3 rounded-full"
                          style={{ backgroundColor: project.color }}
                        />
                      )}
                      <CardTitle className="text-lg">{project.name}</CardTitle>
                    </div>

                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>
                        <Button variant="ghost" className="h-8 w-8 p-0">
                          <MoreHorizontal className="h-4 w-4" />
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end">
                        <DropdownMenuItem onClick={() => setEditingProject(project)}>
                          <Edit className="h-4 w-4 mr-2" />
                          Edit
                        </DropdownMenuItem>
                        <DropdownMenuItem onClick={() => handleArchiveProject(project)}>
                          <Archive className="h-4 w-4 mr-2" />
                          {project.is_active ? "Archive" : "Restore"}
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          onClick={() => handleDeleteProject(project)}
                          className="text-red-600"
                        >
                          <Trash2 className="h-4 w-4 mr-2" />
                          Delete
                        </DropdownMenuItem>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>

                  <div className="flex items-center space-x-2">
                    <Badge variant={project.is_active ? "default" : "secondary"}>
                      {project.is_active ? "Active" : "Archived"}
                    </Badge>
                  </div>
                </CardHeader>

                <CardContent>
                  {project.description && (
                    <CardDescription className="mb-4">
                      {project.description}
                    </CardDescription>
                  )}

                  <div className="text-sm text-gray-500 dark:text-gray-400">
                    Created: {new Date(project.created_at).toLocaleDateString()}
                  </div>
                </CardContent>
              </Card>
            ))}
          </div>
        )}

        {/* Edit Project Dialog */}
        <Dialog open={!!editingProject} onOpenChange={() => setEditingProject(null)}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Edit Project</DialogTitle>
            </DialogHeader>
            {editingProject && (
              <ProjectForm
                initialData={editingProject}
                onSubmit={handleUpdateProject}
              />
            )}
          </DialogContent>
        </Dialog>
      </main>
    </div>
  );
}