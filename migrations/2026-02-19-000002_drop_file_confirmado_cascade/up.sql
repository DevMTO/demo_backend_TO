-- ========================================================================
-- MIGRACIÓN: Drop file confirmado cascade trigger + Fix check_and_update_file_status
--
-- 1. Elimina trg_files_status_change y trg_after_file_status_change()
-- 2. Corrige check_and_update_file_status:
--    - ANTES: file a en_curso solo si TODOS los file_tours son en_curso
--    - AHORA: file a en_curso si AL MENOS UNO es en_curso
-- ========================================================================

-- ========================================================================
-- 1) DROP TRIGGER en files
-- ========================================================================
DROP TRIGGER IF EXISTS trg_files_status_change ON files;

-- ========================================================================
-- 2) DROP TRIGGER FUNCTION
-- ========================================================================
DROP FUNCTION IF EXISTS trg_after_file_status_change();

-- ========================================================================
-- 3) FIX: check_and_update_file_status
--    ANY file_tour en_curso → file a en_curso
--    ALL file_tours completado → file a completado
-- ========================================================================
CREATE OR REPLACE FUNCTION check_and_update_file_status(p_file_id INT)
RETURNS VOID AS $$
DECLARE
    v_file_status VARCHAR(30);
    v_total_tours INT;
    v_en_curso_count INT;
    v_completado_count INT;
BEGIN
    SELECT status INTO v_file_status FROM files WHERE id = p_file_id;

    IF NOT FOUND THEN
        RETURN;
    END IF;

    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE status = 'en_curso'),
        COUNT(*) FILTER (WHERE status = 'completado')
    INTO v_total_tours, v_en_curso_count, v_completado_count
    FROM file_tours
    WHERE id_file = p_file_id;

    IF v_total_tours = 0 THEN
        RETURN;
    END IF;

    -- Si TODOS los file_tours están completados → file a 'completado'
    IF v_completado_count = v_total_tours THEN
        UPDATE files SET status = 'completado' WHERE id = p_file_id AND status != 'completado';
    -- Si AL MENOS UNO está en 'en_curso' → file a 'en_curso'
    ELSIF v_en_curso_count > 0 THEN
        UPDATE files SET status = 'en_curso' WHERE id = p_file_id AND status NOT IN ('en_curso', 'completado');
    END IF;
END;
$$ LANGUAGE plpgsql;
