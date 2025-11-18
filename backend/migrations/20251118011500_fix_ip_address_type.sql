-- Fix ip_address column type from INET to TEXT for compatibility
ALTER TABLE user_sessions ALTER COLUMN ip_address TYPE TEXT;