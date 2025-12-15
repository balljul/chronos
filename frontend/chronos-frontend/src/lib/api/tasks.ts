// Tasks API client functions
export interface Task {
  id: string;
  name: string;
  description?: string;
  project_id?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateTaskRequest {
  name: string;
  description?: string;
  project_id?: string;
}

export interface UpdateTaskRequest {
  name?: string;
  description?: string;
  project_id?: string;
  is_active?: boolean;
}

export interface TaskFilters {
  include_inactive?: boolean;
  project_id?: string;
  without_project?: boolean;
}

class TasksAPI {
  private getAuthHeaders(): HeadersInit {
    const token =
      typeof window !== "undefined"
        ? localStorage.getItem("access_token")
        : null;

    return {
      "Content-Type": "application/json",
      ...(token && { Authorization: `Bearer ${token}` }),
    };
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const errorData = await response
        .json()
        .catch(() => ({ error: "Request failed" }));
      throw new Error(
        errorData.error || `HTTP ${response.status}: ${response.statusText}`,
      );
    }

    // Handle 204 No Content responses
    if (response.status === 204) {
      return {} as T;
    }

    return response.json();
  }

  /**
   * Get list of tasks with optional filters
   */
  async getTasks(filters?: TaskFilters): Promise<Task[]> {
    const params = new URLSearchParams();

    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== "") {
          params.append(key, String(value));
        }
      });
    }

    const url = `/api/tasks${params.toString() ? `?${params.toString()}` : ""}`;

    const response = await fetch(url, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<Task[]>(response);
  }

  /**
   * Get a single task by ID
   */
  async getTask(id: string): Promise<Task> {
    const response = await fetch(`/api/tasks/${id}`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<Task>(response);
  }

  /**
   * Create a new task
   */
  async createTask(data: CreateTaskRequest): Promise<Task> {
    const response = await fetch("/api/tasks", {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(data),
    });

    return this.handleResponse<Task>(response);
  }

  /**
   * Update an existing task
   */
  async updateTask(
    id: string,
    data: UpdateTaskRequest,
  ): Promise<Task> {
    const response = await fetch(`/api/tasks/${id}`, {
      method: "PUT",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(data),
    });

    return this.handleResponse<Task>(response);
  }

  /**
   * Delete a task (soft delete by default)
   */
  async deleteTask(id: string, hardDelete = false): Promise<void> {
    const params = hardDelete ? "?soft=false" : "";
    const response = await fetch(`/api/tasks/${id}${params}`, {
      method: "DELETE",
      headers: this.getAuthHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  /**
   * Archive a task (set is_active to false)
   */
  async archiveTask(id: string): Promise<Task> {
    return this.updateTask(id, { is_active: false });
  }

  /**
   * Restore a task (set is_active to true)
   */
  async restoreTask(id: string): Promise<Task> {
    return this.updateTask(id, { is_active: true });
  }

  /**
   * Get tasks by project ID
   */
  async getTasksByProject(projectId: string, includeInactive = false): Promise<Task[]> {
    return this.getTasks({
      project_id: projectId,
      include_inactive: includeInactive
    });
  }

  /**
   * Get tasks that are not assigned to any project
   */
  async getTasksWithoutProject(includeInactive = false): Promise<Task[]> {
    return this.getTasks({
      without_project: true,
      include_inactive: includeInactive
    });
  }
}

// Export singleton instance
export const tasksAPI = new TasksAPI();

// Export individual functions for convenience
export const {
  getTasks,
  getTask,
  createTask,
  updateTask,
  deleteTask,
  archiveTask,
  restoreTask,
  getTasksByProject,
  getTasksWithoutProject,
} = tasksAPI;