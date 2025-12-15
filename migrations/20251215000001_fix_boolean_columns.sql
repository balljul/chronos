-- Fix boolean columns to be NOT NULL
ALTER TABLE projects ALTER COLUMN is_active SET NOT NULL;
ALTER TABLE tasks ALTER COLUMN is_active SET NOT NULL;