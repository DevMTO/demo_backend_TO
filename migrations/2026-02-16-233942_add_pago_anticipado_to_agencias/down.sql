-- Revertir atributos de política de pago de agencias
ALTER TABLE agencias
    DROP CONSTRAINT IF EXISTS chk_pago_anticipado_coherencia;

ALTER TABLE agencias
    DROP COLUMN IF EXISTS dias_pago_anticipado,
    DROP COLUMN IF EXISTS pago_anticipado;
