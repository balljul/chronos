"use client";

import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState } from "react";

interface ProfileData {
  id: string;
  name?: string;
  email: string;
  created_at: string;
  updated_at: string;
}

interface User {
  id: string;
  name?: string;
  email: string;
  created_at: string;
  updated_at: string;
}

export default function Dashboard() {
  const [profile, setProfile] = useState<ProfileData | null>(null);
  const [users, setUsers] = useState<User[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");
  const router = useRouter();

  const getAuthToken = useCallback(() => {
    if (typeof window === "undefined") {
      return null;
    }
    return localStorage.getItem("access_token");
  }, []);

  const logout = async () => {
    try {
      const token = getAuthToken();
      if (token) {
        await fetch("/api/auth/logout", {
          method: "POST",
          headers: {
            Authorization: `Bearer ${token}`,
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            refresh_token:
              typeof window !== "undefined"
                ? localStorage.getItem("refresh_token")
                : null,
            logout_all_devices: false,
          }),
        });
      }
    } catch (error) {
      console.error("Logout error:", error);
    } finally {
      if (typeof window !== "undefined") {
        localStorage.removeItem("access_token");
        localStorage.removeItem("refresh_token");
      }
      router.push("/login");
    }
  };

  const fetchProfile = useCallback(async () => {
    try {
      const token = getAuthToken();
      if (!token) {
        router.push("/login");
        return;
      }

      const response = await fetch("/api/profile", {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (response.status === 401) {
        if (typeof window !== "undefined") {
          localStorage.removeItem("access_token");
          localStorage.removeItem("refresh_token");
        }
        router.push("/login");
        return;
      }

      if (!response.ok) {
        throw new Error("Failed to fetch profile");
      }

      const data = await response.json();
      setProfile(data);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load profile");
    }
  }, [router, getAuthToken]);

  const fetchUsers = useCallback(async () => {
    try {
      const token = getAuthToken();
      if (!token) return;

      const response = await fetch("/api/users", {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (response.ok) {
        const data = await response.json();
        setUsers(data);
      }
    } catch (err) {
      console.error("Failed to fetch users:", err);
    }
  }, [getAuthToken]);

  useEffect(() => {
    const loadDashboardData = async () => {
      setLoading(true);
      await fetchProfile();
      await fetchUsers();
      setLoading(false);
    };

    loadDashboardData();
  }, [fetchProfile, fetchUsers]);

  const formatDate = (dateString: string) => {
    if (!dateString) return "Unknown";

    const date = new Date(dateString);
    if (Number.isNaN(date.getTime())) return "Invalid date";

    return date.toLocaleString();
  };

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-600 mx-auto"></div>
          <p className="mt-4 text-gray-600 dark:text-gray-400">
            Loading dashboard...
          </p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
        <div className="text-center">
          <p className="text-red-600 dark:text-red-400">{error}</p>
          <button
            type="button"
            onClick={() => router.push("/login")}
            className="mt-4 px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700"
          >
            Back to Login
          </button>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Header */}
      <header className="bg-white dark:bg-gray-800 shadow">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex justify-between items-center py-6">
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
              Dashboard
            </h1>
            <div className="flex items-center space-x-3">
              <button
                type="button"
                onClick={() => router.push("/profile")}
                className="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500"
              >
                Profile
              </button>
              <button
                type="button"
                onClick={logout}
                className="px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-red-500"
              >
                Logout
              </button>
            </div>
          </div>
        </div>
      </header>

      <main className="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
        {/* System Users Section */}
        <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
          <div className="px-4 py-5 sm:p-6">
            <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
              System Users ({users.length})
            </h2>
            <div className="max-h-96 overflow-y-auto">
              {users.length > 0 ? (
                <div className="space-y-3">
                  {users.map((user) => (
                    <div
                      key={user.id}
                      className="border border-gray-200 dark:border-gray-700 rounded-lg p-3"
                    >
                      <div className="flex justify-between items-start">
                        <div className="flex-1">
                          <p className="text-sm font-medium text-gray-900 dark:text-white">
                            {user.name || "Unnamed User"}
                          </p>
                          <p className="text-sm text-gray-500 dark:text-gray-400">
                            {user.email}
                          </p>
                          <p className="text-xs text-gray-400 dark:text-gray-500 mt-1">
                            Joined: {formatDate(user.created_at)}
                          </p>
                        </div>
                        {user.id === profile?.id && (
                          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800 dark:bg-indigo-900 dark:text-indigo-200">
                            You
                          </span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-sm text-gray-500 dark:text-gray-400">
                  No users found
                </p>
              )}
            </div>
          </div>
        </div>

        {/* Statistics Section */}
        <div className="mt-8">
          <div className="bg-white dark:bg-gray-800 overflow-hidden shadow rounded-lg">
            <div className="px-4 py-5 sm:p-6">
              <h2 className="text-lg font-medium text-gray-900 dark:text-white mb-4">
                Statistics
              </h2>
              <div className="grid grid-cols-1 sm:grid-cols-3 gap-6">
                <div className="text-center">
                  <div className="text-2xl font-bold text-indigo-600 dark:text-indigo-400">
                    {users.length}
                  </div>
                  <div className="text-sm text-gray-500 dark:text-gray-400">
                    Total Users
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-green-600 dark:text-green-400">
                    {users.filter((u) => u.name).length}
                  </div>
                  <div className="text-sm text-gray-500 dark:text-gray-400">
                    Users with Names
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-2xl font-bold text-purple-600 dark:text-purple-400">
                    {(() => {
                      if (!profile?.created_at) return 0;

                      const createdDate = new Date(profile.created_at);
                      if (Number.isNaN(createdDate.getTime())) return 0;

                      const daysSince = Math.ceil(
                        (Date.now() - createdDate.getTime()) /
                          (1000 * 60 * 60 * 24),
                      );

                      return daysSince > 0 ? daysSince : 0;
                    })()}
                  </div>
                  <div className="text-sm text-gray-500 dark:text-gray-400">
                    Days Since You Joined
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
