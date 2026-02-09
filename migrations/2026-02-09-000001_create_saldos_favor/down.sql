-- Revert saldos a favor tables

DROP TRIGGER IF EXISTS trg_crear_saldo_favor_agencia ON agencias;
DROP FUNCTION IF EXISTS crear_saldo_favor_agencia();

DROP TABLE IF EXISTS movimientos_saldo_favor;
DROP TABLE IF EXISTS no_shows;
DROP TABLE IF EXISTS saldos_favor;
DROP TABLE IF EXISTS cancelaciones;
