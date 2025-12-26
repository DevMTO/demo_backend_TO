-- Rollback: Drop pagos table and related functions
DROP TRIGGER IF EXISTS trigger_update_file_monto ON pagos;
DROP FUNCTION IF EXISTS update_file_monto_pagado();
DROP TABLE IF EXISTS pagos;
