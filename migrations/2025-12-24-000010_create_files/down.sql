-- Rollback: Drop files table and related functions
DROP TRIGGER IF EXISTS trigger_generate_file_code ON files;
DROP FUNCTION IF EXISTS generate_file_code();
DROP TABLE IF EXISTS files;
