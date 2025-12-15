// Projects API client functions
export interface Project {
  id: string;
  name: string;
  description?: string;
  color?: string;
  is_active: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateProjectRequest {
  name: string;
  description?: string;
  color?: string;
}

export interface UpdateProjectRequest {
  name?: string;
  description?: string;
  color?: string;
  is_active?: boolean;
}

export interface ProjectFilters {
  include_inactive?: boolean;
}

class ProjectsAPI {
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
   * Get list of projects with optional filters
   */
  async getProjects(filters?: ProjectFilters): Promise<Project[]> {
    const params = new URLSearchParams();

    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== "") {
          params.append(key, String(value));
        }
      });
    }

    const url = `/api/projects${params.toString() ? `?${params.toString()}` : ""}`;

    const response = await fetch(url, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<Project[]>(response);
  }

  /**
   * Get a single project by ID
   */
  async getProject(id: string): Promise<Project> {
    const response = await fetch(`/api/projects/${id}`, {
      method: "GET",
      headers: this.getAuthHeaders(),
    });

    return this.handleResponse<Project>(response);
  }

  /**
   * Create a new project
   */
  async createProject(data: CreateProjectRequest): Promise<Project> {
    const response = await fetch("/api/projects", {
      method: "POST",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(data),
    });

    return this.handleResponse<Project>(response);
  }

  /**
   * Update an existing project
   */
  async updateProject(
    id: string,
    data: UpdateProjectRequest,
  ): Promise<Project> {
    const response = await fetch(`/api/projects/${id}`, {
      method: "PUT",
      headers: this.getAuthHeaders(),
      body: JSON.stringify(data),
    });

    return this.handleResponse<Project>(response);
  }

  /**
   * Delete a project (soft delete by default)
   */
  async deleteProject(id: string, hardDelete = false): Promise<void> {
    const params = hardDelete ? "?soft=false" : "";
    const response = await fetch(`/api/projects/${id}${params}`, {
      method: "DELETE",
      headers: this.getAuthHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  /**
   * Archive a project (set is_active to false)
   */
  async archiveProject(id: string): Promise<Project> {
    return this.updateProject(id, { is_active: false });
  }

  /**
   * Restore a project (set is_active to true)
   */
  async restoreProject(id: string): Promise<Project> {
    return this.updateProject(id, { is_active: true });
  }
}

// Export singleton instance
export const projectsAPI = new ProjectsAPI();

// Export individual functions for convenience
export const {
  getProjects,
  getProject,
  createProject,
  updateProject,
  deleteProject,
  archiveProject,
  restoreProject,
} = projectsAPI;