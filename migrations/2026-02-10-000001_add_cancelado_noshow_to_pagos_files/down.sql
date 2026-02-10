-- Revert: restore original pagos_files estado CHECK constraint
ALTER TABLE pagos_files DROP CONSTRAINT IF EXISTS chk_pago_file_estado;
ALTER TABLE pagos_files ADD CONSTRAINT chk_pago_file_estado 
    CHECK (estado IN ('pendiente', 'parcial', 'pagado', 'vencido'));

-- Note: Dropped tables (pagos, cuentas, movimientos, tarifas_servicios)
-- cannot be automatically restored. Manual recreation would be needed.
