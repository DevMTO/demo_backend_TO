-- Migration: Remove unused tables (not in diagram)

-- Eliminar dependencias primero
DROP INDEX IF EXISTS idx_user_documents_user_id;
DROP INDEX IF EXISTS idx_user_documents_document_type_id;
DROP INDEX IF EXISTS idx_document_types_code;

-- Eliminar tablas
DROP TABLE IF EXISTS user_documents CASCADE;
DROP TABLE IF EXISTS document_types CASCADE;

-- Nota: oauth_providers, refresh_tokens, login_attempts se mantienen para seguridad
