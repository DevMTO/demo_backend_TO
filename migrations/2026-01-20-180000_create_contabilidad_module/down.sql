-- ========================================================================
-- ROLLBACK: Módulo de Contabilidad
-- ========================================================================

-- Eliminar triggers primero
DROP TRIGGER IF EXISTS trigger_crear_cuenta_agencia ON agencias;
DROP TRIGGER IF EXISTS trigger_tarifas_servicios_updated_at ON tarifas_servicios;
DROP TRIGGER IF EXISTS trigger_pagos_proveedores_updated_at ON pagos_proveedores;
DROP TRIGGER IF EXISTS trigger_pagos_files_updated_at ON pagos_files;
DROP TRIGGER IF EXISTS trigger_cuentas_updated_at ON cuentas;

-- Eliminar funciones
DROP FUNCTION IF EXISTS crear_cuenta_agencia();
DROP FUNCTION IF EXISTS update_cuentas_updated_at();

-- Eliminar tablas en orden inverso (respetando FKs)
DROP TABLE IF EXISTS tarifas_servicios;
DROP TABLE IF EXISTS pagos_proveedores;
DROP TABLE IF EXISTS pagos_files;
DROP TABLE IF EXISTS movimientos;
DROP TABLE IF EXISTS cuentas;
