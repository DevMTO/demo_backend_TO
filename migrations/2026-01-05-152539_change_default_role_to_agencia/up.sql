-- ========================================================================
-- CAMBIAR DEFAULT DE ROLE A AGENCIA
-- El default 'operador' no existe en el sistema, cambiamos a 'agencia'
-- ========================================================================

-- Cambiar el default de la columna role a 'agencia'
ALTER TABLE users ALTER COLUMN role SET DEFAULT 'agencia';

-- Actualizar cualquier registro existente que tenga 'operador' a 'agencia'
UPDATE users SET role = 'agencia' WHERE role = 'operador';
