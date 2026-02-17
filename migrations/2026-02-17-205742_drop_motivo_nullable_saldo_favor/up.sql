-- Drop motivo column (redundante con notas) y hacer monto_saldo_favor nullable

-- 1. Migrar datos de motivo a notas donde notas esté vacío
UPDATE pagos_files SET notas = motivo WHERE notas IS NULL AND motivo IS NOT NULL;

-- 2. Eliminar columna motivo
ALTER TABLE pagos_files DROP COLUMN motivo;

-- 3. Hacer monto_saldo_favor nullable y convertir 0 a NULL
ALTER TABLE pagos_files ALTER COLUMN monto_saldo_favor DROP NOT NULL;
ALTER TABLE pagos_files ALTER COLUMN monto_saldo_favor DROP DEFAULT;
UPDATE pagos_files SET monto_saldo_favor = NULL WHERE monto_saldo_favor = 0;
