-- Revert: Drop user_documents table
DROP TRIGGER IF EXISTS update_user_documents_updated_at ON user_documents;
DROP INDEX IF EXISTS idx_user_documents_is_primary;
DROP INDEX IF EXISTS idx_user_documents_number;
DROP INDEX IF EXISTS idx_user_documents_document_type;
DROP INDEX IF EXISTS idx_user_documents_user_id;
DROP TABLE IF EXISTS user_documents;
