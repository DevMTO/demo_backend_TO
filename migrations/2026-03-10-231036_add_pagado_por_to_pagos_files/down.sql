ALTER TABLE pagos_files
    DROP COLUMN IF EXISTS pagado_por,
    DROP COLUMN IF EXISTS pagado_at,
    DROP COLUMN IF EXISTS updated_by;

