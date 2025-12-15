"use client";

import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { toast } from "sonner";
import type { Task } from "@/lib/api/tasks";
import type { Project } from "@/lib/api/projects";

interface TaskFormProps {
  projects: Project[];
  initialData?: Task;
  onSubmit: (data: { name: string; description?: string; project_id?: string }) => Promise<void>;
}

export default function TaskForm({ projects, initialData, onSubmit }: TaskFormProps) {
  const [name, setName] = useState(initialData?.name || "");
  const [description, setDescription] = useState(initialData?.description || "");
  const [projectId, setProjectId] = useState<string | undefined>(initialData?.project_id || undefined);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!name.trim()) {
      toast.error("Task name is required");
      return;
    }

    setLoading(true);
    try {
      await onSubmit({
        name: name.trim(),
        description: description.trim() || undefined,
        project_id: projectId || undefined,
      });
    } catch (error) {
      // Error handling is done in the parent component
    } finally {
      setLoading(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="name">Task Name *</Label>
        <Input
          id="name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          placeholder="Enter task name"
          maxLength={255}
          required
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="description">Description</Label>
        <Textarea
          id="description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          placeholder="Enter task description (optional)"
          maxLength={1000}
          rows={3}
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="project">Project</Label>
        <Select value={projectId || "no-project"} onValueChange={(value) => setProjectId(value === "no-project" ? undefined : value)}>
          <SelectTrigger>
            <SelectValue placeholder="Select a project (optional)" />
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
      </div>

      <div className="flex justify-end space-x-2 pt-4">
        <Button type="submit" disabled={loading}>
          {loading ? "Saving..." : initialData ? "Update Task" : "Create Task"}
        </Button>
      </div>
    </form>
  );
}