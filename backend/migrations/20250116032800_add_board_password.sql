-- Add password and is_locked fields to boards table
ALTER TABLE boards
ADD COLUMN password VARCHAR(255) NOT NULL DEFAULT '',
ADD COLUMN is_locked BOOLEAN NOT NULL DEFAULT FALSE;

-- Add index for better query performance (only if it doesn't exist)
CREATE INDEX IF NOT EXISTS idx_boards_share_token ON boards(share_token);