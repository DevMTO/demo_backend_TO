-- ========================================================================
-- AGREGAR CAMPO turno_tour A TABLA FILES
-- Turno del tour: mañana, tarde, noche, full day, etc.
-- ========================================================================

ALTER TABLE files ADD COLUMN turno_tour VARCHAR(30) NULL;

-- Comentario para documentación
COMMENT ON COLUMN files.turno_tour IS 'Turno del tour: mañana, tarde, noche, full_day, etc.';