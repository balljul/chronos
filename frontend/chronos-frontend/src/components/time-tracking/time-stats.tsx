"use client";

import { Clock, Calendar, TrendingUp } from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { formatDuration } from "@/lib/time-utils";
import type { TimeStats } from "@/types/time-entries";

interface TimeStatsProps {
  stats: TimeStats;
  loading: boolean;
}

export function TimeStatsCards({ stats, loading }: TimeStatsProps) {
  if (loading) {
    return (
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {[1, 2, 3].map((i) => (
          <Card key={i}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <Skeleton className="h-4 w-20" />
              <Skeleton className="h-4 w-4" />
            </CardHeader>
            <CardContent>
              <Skeleton className="h-8 w-16 mb-2" />
              <Skeleton className="h-3 w-24" />
            </CardContent>
          </Card>
        ))}
      </div>
    );
  }

  const statCards = [
    {
      title: "Today",
      value: stats.today,
      entries: stats.total_entries_today,
      icon: Clock,
      color: "text-blue-600 dark:text-blue-400",
      bgColor: "bg-blue-100 dark:bg-blue-900/20",
    },
    {
      title: "This Week",
      value: stats.this_week,
      entries: stats.total_entries_week,
      icon: Calendar,
      color: "text-green-600 dark:text-green-400",
      bgColor: "bg-green-100 dark:bg-green-900/20",
    },
    {
      title: "This Month",
      value: stats.this_month,
      entries: stats.total_entries_month,
      icon: TrendingUp,
      color: "text-purple-600 dark:text-purple-400",
      bgColor: "bg-purple-100 dark:bg-purple-900/20",
    },
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {statCards.map((stat) => {
        const Icon = stat.icon;

        return (
          <Card key={stat.title}>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium text-muted-foreground">
                {stat.title}
              </CardTitle>
              <div className={`p-2 rounded-md ${stat.bgColor}`}>
                <Icon className={`h-4 w-4 ${stat.color}`} />
              </div>
            </CardHeader>
            <CardContent>
              <div className="space-y-1">
                <div className="text-2xl font-bold">
                  {stat.value > 0 ? formatDuration(stat.value) : "0m"}
                </div>
                <p className="text-xs text-muted-foreground">
                  {stat.entries === 0
                    ? "No entries"
                    : `${stat.entries} ${stat.entries === 1 ? "entry" : "entries"}`}
                </p>
              </div>
            </CardContent>
          </Card>
        );
      })}
    </div>
  );
}

export function DetailedTimeStats({ stats, loading }: TimeStatsProps) {
  if (loading) {
    return (
      <Card>
        <CardHeader>
          <Skeleton className="h-5 w-32" />
        </CardHeader>
        <CardContent className="space-y-4">
          {[1, 2, 3].map((i) => (
            <div key={i} className="flex justify-between items-center">
              <Skeleton className="h-4 w-24" />
              <Skeleton className="h-4 w-16" />
            </div>
          ))}
        </CardContent>
      </Card>
    );
  }

  const totalHoursToday = stats.today / 3600;
  const averageEntryDuration =
    stats.total_entries_today > 0 ? stats.today / stats.total_entries_today : 0;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Detailed Statistics</CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-3">
          <div className="flex justify-between items-center">
            <span className="text-sm text-muted-foreground">Hours today</span>
            <span className="font-medium">{totalHoursToday.toFixed(1)}h</span>
          </div>

          <div className="flex justify-between items-center">
            <span className="text-sm text-muted-foreground">Average entry</span>
            <span className="font-medium">
              {averageEntryDuration > 0
                ? formatDuration(Math.round(averageEntryDuration))
                : "0m"}
            </span>
          </div>

          <div className="flex justify-between items-center">
            <span className="text-sm text-muted-foreground">
              Productivity score
            </span>
            <span className="font-medium text-green-600 dark:text-green-400">
              {stats.total_entries_today > 5
                ? "High"
                : stats.total_entries_today > 2
                  ? "Medium"
                  : "Low"}
            </span>
          </div>
        </div>

        {/* Progress bars for daily goals */}
        <div className="space-y-3 pt-2 border-t">
          <div className="space-y-1">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Daily Goal (8h)</span>
              <span className="font-medium">
                {Math.min(100, Math.round((totalHoursToday / 8) * 100))}%
              </span>
            </div>
            <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2">
              <div
                className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                style={{
                  width: `${Math.min(100, (totalHoursToday / 8) * 100)}%`,
                }}
              />
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
