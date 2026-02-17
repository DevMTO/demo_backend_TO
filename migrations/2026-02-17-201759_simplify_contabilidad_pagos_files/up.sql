-- =============================================================================
-- SIMPLIFICACIÓN: Todo en pagos_files
-- DROP: cancelaciones, no_shows, saldos_favor, movimientos_saldo_favor
-- ADD: columnas a pagos_files para cancel/no_show/saldo
-- =============================================================================

-- 1. Agregar nuevas columnas a pagos_files
ALTER TABLE pagos_files
    ADD COLUMN id_file_tour INTEGER REFERENCES file_tours(id) ON DELETE SET NULL,
    ADD COLUMN tipo_registro VARCHAR(30) NOT NULL DEFAULT 'deuda',
    ADD COLUMN monto_saldo_favor NUMERIC(12,2) NOT NULL DEFAULT 0,
    ADD COLUMN motivo TEXT,
    ADD COLUMN saldo_autorizado BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN saldo_autorizado_por INTEGER REFERENCES users(id),
    ADD COLUMN saldo_autorizado_at TIMESTAMPTZ;

-- 2. Migrar datos existentes de cancelaciones → pagos_files
INSERT INTO pagos_files (
    id_file, id_agencia, monto_total, monto_pagado, estado, notas, created_at, updated_at, created_by,
    id_file_tour, tipo_registro, monto_saldo_favor, motivo
)
SELECT
    c.id_file,
    c.id_agencia,
    c.monto_total_file,
    c.monto_pagado,
    CASE
        WHEN c.tipo_cancelacion IN ('no_show', 'no_show_tour') THEN 'no_show'
        ELSE 'cancelado'
    END as estado,
    c.notas,
    c.created_at,
    c.created_at,
    c.created_by,
    c.id_file_tour,
    c.tipo_cancelacion as tipo_registro,
    c.monto_saldo_favor,
    c.motivo
FROM cancelaciones c;

-- 3. Para cancelaciones con saldo > 0, auto-autorizar
UPDATE pagos_files
SET saldo_autorizado = TRUE
WHERE tipo_registro IN ('cancelacion', 'cancelacion_tour')
  AND monto_saldo_favor > 0;

-- 4. Drop tablas (en orden por dependencias FK)
DROP TABLE IF EXISTS movimientos_saldo_favor CASCADE;
DROP TABLE IF EXISTS no_shows CASCADE;
DROP TABLE IF EXISTS cancelaciones CASCADE;
DROP TABLE IF EXISTS saldos_favor CASCADE;

-- 5. Drop trigger y función (CASCADE para dependencias)
DROP TRIGGER IF EXISTS trg_crear_saldo_favor ON agencias;
DROP FUNCTION IF EXISTS crear_saldo_favor_agencia() CASCADE;

-- 6. Índices
CREATE INDEX IF NOT EXISTS idx_pagos_files_tipo_registro ON pagos_files(tipo_registro);
CREATE INDEX IF NOT EXISTS idx_pagos_files_file_tour ON pagos_files(id_file_tour) WHERE id_file_tour IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_pagos_files_saldo ON pagos_files(id_agencia, monto_saldo_favor) WHERE monto_saldo_favor > 0;