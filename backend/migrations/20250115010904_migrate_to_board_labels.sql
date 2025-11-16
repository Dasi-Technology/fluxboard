-- Migration: Migrate to board-level labels system
-- Description: Convert card-level labels to board-level labels with many-to-many relationship
-- This migration will:
-- 1. Create new board_labels table for board-level label definitions
-- 2. Create card_labels junction table for card-label assignments
-- 3. Migrate existing label data, merging duplicates per board
-- 4. Drop old labels table

BEGIN;

-- Step 1: Create board_labels table
CREATE TABLE board_labels (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    board_id UUID NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    color VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Prevent duplicate labels on same board
    UNIQUE(board_id, name, color)
);

-- Create index on board_id for fast lookups
CREATE INDEX idx_board_labels_board_id ON board_labels(board_id);

-- Create trigger for board_labels table
CREATE TRIGGER update_board_labels_updated_at
    BEFORE UPDATE ON board_labels
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Step 2: Create card_labels junction table
CREATE TABLE card_labels (
    card_id UUID NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    label_id UUID NOT NULL REFERENCES board_labels(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (card_id, label_id)
);

-- Create indexes for junction table
CREATE INDEX idx_card_labels_card_id ON card_labels(card_id);
CREATE INDEX idx_card_labels_label_id ON card_labels(label_id);

-- Step 3: Migrate existing label data
-- This will:
-- - Extract all labels with their board context
-- - Create unique board_labels (merging duplicates)
-- - Create card_labels assignments

-- First, create a temporary table with all labels and their board context
CREATE TEMP TABLE temp_label_migration AS
SELECT 
    l.id as old_label_id,
    l.card_id,
    l.name,
    l.color,
    col.board_id,
    l.created_at
FROM labels l
INNER JOIN cards c ON l.card_id = c.id
INNER JOIN columns col ON c.column_id = col.id;

-- Create unique board labels (this automatically merges duplicates)
INSERT INTO board_labels (board_id, name, color, created_at)
SELECT DISTINCT ON (board_id, name, color)
    board_id,
    name,
    color,
    MIN(created_at) OVER (PARTITION BY board_id, name, color) as created_at
FROM temp_label_migration
ORDER BY board_id, name, color, created_at;

-- Create card-label assignments
-- This links each card to the appropriate board label
INSERT INTO card_labels (card_id, label_id, created_at)
SELECT DISTINCT
    tlm.card_id,
    bl.id as label_id,
    tlm.created_at
FROM temp_label_migration tlm
INNER JOIN board_labels bl ON 
    tlm.board_id = bl.board_id AND
    tlm.name = bl.name AND
    tlm.color = bl.color
ORDER BY tlm.card_id, bl.id;

-- Clean up temp table
DROP TABLE temp_label_migration;

-- Step 4: Drop old labels table
DROP TABLE labels;

COMMIT;