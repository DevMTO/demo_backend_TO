-- Agregar atributos de política de pago a agencias
ALTER TABLE agencias
    ADD COLUMN pago_anticipado BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN dias_pago_anticipado INTEGER;

-- Para agencias existentes con pago_anticipado=FALSE, establecer días por defecto (30 días)
UPDATE agencias SET dias_pago_anticipado = 30 WHERE pago_anticipado = FALSE;

-- Si pago_anticipado es TRUE, dias_pago_anticipado debe ser NULL
-- Si pago_anticipado es FALSE, dias_pago_anticipado debe tener un valor
ALTER TABLE agencias
    ADD CONSTRAINT chk_pago_anticipado_coherencia CHECK (
        (pago_anticipado = TRUE AND dias_pago_anticipado IS NULL)
        OR
        (pago_anticipado = FALSE AND dias_pago_anticipado IS NOT NULL)
    );
