-- Fix the conflicting NOT NULL and ON DELETE SET NULL constraint on uploaded_by
ALTER TABLE card_attachments
DROP CONSTRAINT card_attachments_uploaded_by_fkey;

-- Add the foreign key constraint with CASCADE instead of SET NULL
-- This means if a user is deleted, their attachments are also deleted
ALTER TABLE card_attachments
ADD CONSTRAINT card_attachments_uploaded_by_fkey 
FOREIGN KEY (uploaded_by) 
REFERENCES users(id) 
ON DELETE CASCADE;