-- Rollback: restaurar versiones anteriores de las funciones
-- (las versiones de la migración 2026-02-17-000001 se restauran automáticamente
-- al volver a ejecutar esa migración)
SELECT 1; -- No-op, las funciones se sobreescriben con CREATE OR REPLACE
