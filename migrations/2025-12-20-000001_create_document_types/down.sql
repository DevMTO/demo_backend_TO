-- Revert: Drop document_types table
DROP INDEX IF EXISTS idx_document_types_code;
DROP TABLE IF EXISTS document_types;
