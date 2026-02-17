-- Revert: quitar columnas agregadas a pagos_files
ALTER TABLE pagos_files
    DROP COLUMN IF EXISTS id_file_tour,
    DROP COLUMN IF EXISTS tipo_registro,
    DROP COLUMN IF EXISTS monto_saldo_favor,
    DROP COLUMN IF EXISTS motivo,
    DROP COLUMN IF EXISTS saldo_autorizado,
    DROP COLUMN IF EXISTS saldo_autorizado_por,
    DROP COLUMN IF EXISTS saldo_autorizado_at;

DROP INDEX IF EXISTS idx_pagos_files_tipo_registro;
DROP INDEX IF EXISTS idx_pagos_files_file_tour;
DROP INDEX IF EXISTS idx_pagos_files_saldo;

-- NOTA: Las tablas cancelaciones, no_shows, saldos_favor, movimientos_saldo_favor
-- necesitarían ser recreadas manualmente si se revierte esta migración.