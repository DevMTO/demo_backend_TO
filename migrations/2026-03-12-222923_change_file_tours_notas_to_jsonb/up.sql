-- Convertir file_tours.notas de TEXT a JSONB
-- Los datos existentes (JSON como texto) se convierten automáticamente
ALTER TABLE file_tours
  ALTER COLUMN notas TYPE JSONB
  USING CASE
    WHEN notas IS NULL THEN NULL
    WHEN notas ~ '^\s*[\[\{]' THEN notas::jsonb
    ELSE jsonb_build_array(jsonb_build_object('nota', notas, 'timestamp', NOW()))
  END;

