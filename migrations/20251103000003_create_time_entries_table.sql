CREATE TABLE time_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    description TEXT,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    start_time TIMESTAMP WITH TIME ZONE NOT NULL,
    end_time TIMESTAMP WITH TIME ZONE, -- NULL indicates timer is currently running
    duration INTEGER, -- Duration in seconds, calculated field
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    
    -- Ensure that end_time is after start_time when both are set
    CONSTRAINT chk_end_after_start CHECK (end_time IS NULL OR end_time > start_time)
);

-- Indexes for performance
CREATE INDEX idx_time_entries_user_id ON time_entries(user_id);
CREATE INDEX idx_time_entries_start_time ON time_entries(start_time);
CREATE INDEX idx_time_entries_user_start_time ON time_entries(user_id, start_time);
CREATE INDEX idx_time_entries_project_id ON time_entries(project_id);
CREATE INDEX idx_time_entries_task_id ON time_entries(task_id);
CREATE INDEX idx_time_entries_running ON time_entries(user_id) WHERE end_time IS NULL;

-- Unique constraint to prevent multiple running timers per user
CREATE UNIQUE INDEX idx_time_entries_user_running ON time_entries(user_id) 
WHERE end_time IS NULL;