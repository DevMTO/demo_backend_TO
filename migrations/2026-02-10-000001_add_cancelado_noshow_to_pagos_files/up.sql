-- Remove CHECK constraint on pagos_files.estado (handle validation in backend)
ALTER TABLE pagos_files DROP CONSTRAINT IF EXISTS chk_pago_file_estado;

-- Drop unused/empty contabilidad tables
-- (cuentas, movimientos, tarifas_servicios, pagos)
-- Keep: pagos_files, pagos_proveedores

-- Drop movimientos first (references cuentas)
DROP TABLE IF EXISTS movimientos CASCADE;
-- Drop cuentas 
DROP TABLE IF EXISTS cuentas CASCADE;
-- Drop tarifas_servicios
DROP TABLE IF EXISTS tarifas_servicios CASCADE;
-- Drop old pagos table
DROP TABLE IF EXISTS pagos CASCADE;

-- Remove the trigger that auto-creates cuentas for agencias (if exists)
DROP FUNCTION IF EXISTS crear_cuenta_agencia() CASCADE;
