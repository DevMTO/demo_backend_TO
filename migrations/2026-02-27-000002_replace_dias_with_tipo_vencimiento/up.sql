-- Reemplazar dias_pago_anticipado (INTEGER) por tipo_vencimiento (VARCHAR)
-- Valores posibles: 'semanal', 'quincenal', 'mensual'

-- 1. Quitar constraint existente
ALTER TABLE agencias DROP CONSTRAINT IF EXISTS chk_pago_anticipado_coherencia;

-- 2. Quitar columna vieja
ALTER TABLE agencias DROP COLUMN IF EXISTS dias_pago_anticipado;

-- 3. Agregar nueva columna
ALTER TABLE agencias ADD COLUMN tipo_vencimiento VARCHAR(20);

-- 4. Migrar datos: agencias sin pago anticipado obtienen 'mensual' por defecto
UPDATE agencias SET tipo_vencimiento = 'mensual' WHERE pago_anticipado = FALSE;

-- 5. Nueva constraint de coherencia
ALTER TABLE agencias ADD CONSTRAINT chk_pago_anticipado_coherencia CHECK (
    (pago_anticipado = TRUE AND tipo_vencimiento IS NULL)
    OR
    (pago_anticipado = FALSE AND tipo_vencimiento IN ('semanal', 'quincenal', 'mensual'))
);
