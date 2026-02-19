ALTER TABLE pagos_files
    DROP COLUMN IF EXISTS entradas,
    DROP COLUMN IF EXISTS entrada_precio;

ALTER TABLE pagos_files ALTER COLUMN estado TYPE VARCHAR(20);
