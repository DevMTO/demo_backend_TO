-- ========================================================================
-- ROLLBACK: Automatización de estados de file_tour y file
-- ========================================================================

-- 1) Drop triggers
DROP TRIGGER IF EXISTS trg_file_guias_status_change ON file_guias;
DROP TRIGGER IF EXISTS trg_file_vehiculos_status_change ON file_vehiculos;
DROP TRIGGER IF EXISTS trg_file_restaurantes_status_change ON file_restaurantes;
DROP TRIGGER IF EXISTS trg_file_tours_status_change ON file_tours;

-- 2) Drop trigger functions
DROP FUNCTION IF EXISTS trg_after_subfile_status_change();
DROP FUNCTION IF EXISTS trg_after_file_tour_status_change();

-- 3) Drop helper functions
DROP FUNCTION IF EXISTS check_and_update_file_tour_status(INT);
DROP FUNCTION IF EXISTS check_and_update_file_status(INT);

-- 4) Drop daily function
DROP FUNCTION IF EXISTS automatizar_estados_por_fecha();
