-- ========================================================================
-- REVERTIR DEFAULT DE ROLE A OPERADOR
-- ========================================================================

-- Revertir el default de la columna role a 'operador'
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'operador';
