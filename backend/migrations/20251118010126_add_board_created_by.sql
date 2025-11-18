-- Add optional user relationship to boards
ALTER TABLE boards
ADD COLUMN created_by UUID REFERENCES users(id) ON DELETE SET NULL;

-- Index for filtering boards by creator
CREATE INDEX idx_boards_created_by ON boards(created_by);
