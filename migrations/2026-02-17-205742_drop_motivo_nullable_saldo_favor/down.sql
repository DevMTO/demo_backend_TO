-- Revert: restaurar motivo y hacer monto_saldo_favor NOT NULL

-- 1. Restaurar columna motivo
ALTER TABLE pagos_files ADD COLUMN motivo TEXT;

-- 2. Hacer monto_saldo_favor NOT NULL con default 0
UPDATE pagos_files SET monto_saldo_favor = 0 WHERE monto_saldo_favor IS NULL;
ALTER TABLE pagos_files ALTER COLUMN monto_saldo_favor SET NOT NULL;
ALTER TABLE pagos_files ALTER COLUMN monto_saldo_favor SET DEFAULT 0;
