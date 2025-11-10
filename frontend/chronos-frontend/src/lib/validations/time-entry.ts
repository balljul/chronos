import * as z from "zod";

// Time Entry Creation Schema
export const createTimeEntrySchema = z
  .object({
    description: z
      .string()
      .max(1000, "Description cannot exceed 1000 characters")
      .optional()
      .or(z.literal("")),
    project_id: z.string().uuid().optional().or(z.literal("")),
    task_id: z.string().uuid().optional().or(z.literal("")),
    start_time: z.date({
      required_error: "Start time is required",
      invalid_type_error: "Please enter a valid start time",
    }),
    end_time: z
      .date({
        invalid_type_error: "Please enter a valid end time",
      })
      .optional(),
  })
  .refine(
    (data) => {
      if (data.end_time && data.start_time) {
        return data.end_time > data.start_time;
      }
      return true;
    },
    {
      message: "End time must be after start time",
      path: ["end_time"],
    },
  )
  .refine(
    (data) => {
      // Ensure start time is not in the future
      return data.start_time <= new Date();
    },
    {
      message: "Start time cannot be in the future",
      path: ["start_time"],
    },
  );

// Time Entry Update Schema (all fields optional)
export const updateTimeEntrySchema = z
  .object({
    description: z
      .string()
      .max(1000, "Description cannot exceed 1000 characters")
      .optional()
      .or(z.literal("")),
    project_id: z.string().uuid().optional().or(z.literal("")),
    task_id: z.string().uuid().optional().or(z.literal("")),
    start_time: z
      .date({
        invalid_type_error: "Please enter a valid start time",
      })
      .optional(),
    end_time: z
      .date({
        invalid_type_error: "Please enter a valid end time",
      })
      .optional(),
  })
  .refine(
    (data) => {
      if (data.end_time && data.start_time) {
        return data.end_time > data.start_time;
      }
      return true;
    },
    {
      message: "End time must be after start time",
      path: ["end_time"],
    },
  );

// Timer Start Schema
export const startTimerSchema = z.object({
  description: z
    .string()
    .max(1000, "Description cannot exceed 1000 characters")
    .optional()
    .or(z.literal("")),
  project_id: z.string().uuid().optional().or(z.literal("")),
  task_id: z.string().uuid().optional().or(z.literal("")),
});

// Time Entry Filters Schema
export const timeEntryFiltersSchema = z.object({
  start_date: z.string().datetime().optional().or(z.literal("")),
  end_date: z.string().datetime().optional().or(z.literal("")),
  project_id: z.string().uuid().optional().or(z.literal("")),
  task_id: z.string().uuid().optional().or(z.literal("")),
  is_running: z.boolean().optional(),
  page: z
    .number()
    .int()
    .min(1, "Page must be at least 1")
    .max(1000, "Page cannot exceed 1000")
    .optional(),
  limit: z
    .number()
    .int()
    .min(1, "Limit must be at least 1")
    .max(100, "Limit cannot exceed 100")
    .optional(),
  sort_by: z.enum(["start_time", "duration"]).optional(),
});

// Form data schemas for React Hook Form
export const timeEntryFormSchema = z
  .object({
    description: z.string().max(1000).optional(),
    project_id: z.string().optional(),
    task_id: z.string().optional(),
    start_time: z.date(),
    end_time: z.date().optional(),
  })
  .refine(
    (data) => {
      if (data.end_time && data.start_time) {
        return data.end_time > data.start_time;
      }
      return true;
    },
    {
      message: "End time must be after start time",
      path: ["end_time"],
    },
  );

export const timerFormSchema = z.object({
  description: z.string().max(1000).optional(),
  project_id: z.string().optional(),
  task_id: z.string().optional(),
});

// Type inference
export type CreateTimeEntryFormData = z.infer<typeof createTimeEntrySchema>;
export type UpdateTimeEntryFormData = z.infer<typeof updateTimeEntrySchema>;
export type StartTimerFormData = z.infer<typeof startTimerSchema>;
export type TimeEntryFiltersData = z.infer<typeof timeEntryFiltersSchema>;
export type TimeEntryFormData = z.infer<typeof timeEntryFormSchema>;
export type TimerFormData = z.infer<typeof timerFormSchema>;
