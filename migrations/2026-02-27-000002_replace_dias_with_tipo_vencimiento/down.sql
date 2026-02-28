-- Revertir: quitar tipo_vencimiento, restaurar dias_pago_anticipado
ALTER TABLE agencias DROP CONSTRAINT IF EXISTS chk_pago_anticipado_coherencia;

ALTER TABLE agencias DROP COLUMN IF EXISTS tipo_vencimiento;

ALTER TABLE agencias ADD COLUMN dias_pago_anticipado INTEGER;

UPDATE agencias SET dias_pago_anticipado = 30 WHERE pago_anticipado = FALSE;

ALTER TABLE agencias ADD CONSTRAINT chk_pago_anticipado_coherencia CHECK (
    (pago_anticipado = TRUE AND dias_pago_anticipado IS NULL)
    OR
    (pago_anticipado = FALSE AND dias_pago_anticipado IS NOT NULL)
);
