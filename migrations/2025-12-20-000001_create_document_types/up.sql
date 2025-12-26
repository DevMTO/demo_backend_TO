-- Migration: Create document_types table
-- Initial setup for document type catalog

CREATE TABLE document_types (
    id SERIAL PRIMARY KEY,
    code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    format_regex VARCHAR(100),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default document types
INSERT INTO document_types (code, name, format_regex) VALUES
    ('DNI', 'Documento Nacional de Identidad', '^\d{8}$'),
    ('PASSPORT', 'Pasaporte', '^[A-Z0-9]{6,12}$'),
    ('FOREIGNER_CARD', 'Carné de Extranjería', '^[A-Z0-9]{9,12}$'),
    ('RUC', 'Registro Único de Contribuyentes', '^\d{11}$');

-- Create index for quick lookups
CREATE INDEX idx_document_types_code ON document_types(code);
