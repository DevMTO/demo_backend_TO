DROP TRIGGER IF EXISTS trigger_update_file_monto ON pagos;
DROP FUNCTION IF EXISTS update_file_monto_pagado();
DROP TRIGGER IF EXISTS update_pagos_updated_at ON pagos;
DROP TABLE IF EXISTS pagos CASCADE;
