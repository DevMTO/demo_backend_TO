-- ========================================================================
-- ROLLBACK: Restore file confirmado cascade + revert check_and_update_file_status
-- ========================================================================

-- ========================================================================
-- 1) RESTORE TRIGGER FUNCTION: trg_after_file_status_change
-- ========================================================================
CREATE OR REPLACE FUNCTION trg_after_file_status_change()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status = 'confirmado' AND (OLD.status IS NULL OR OLD.status != 'confirmado') THEN
        UPDATE file_tours
        SET status = 'confirmado'
        WHERE id_file = NEW.id
          AND status IN ('pendiente', 'reservado');
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ========================================================================
-- 2) RECREATE TRIGGER en files
-- ========================================================================
CREATE TRIGGER trg_files_status_change
    AFTER UPDATE OF status ON files
    FOR EACH ROW
    EXECUTE FUNCTION trg_after_file_status_change();

-- ========================================================================
-- 3) REVERT check_and_update_file_status a la versión anterior
--    (requiere TODOS en_curso para file a en_curso)
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

    IF v_completado_count = v_total_tours THEN
        UPDATE files SET status = 'completado' WHERE id = p_file_id AND status != 'completado';
    ELSIF (v_en_curso_count + v_completado_count) = v_total_tours THEN
        UPDATE files SET status = 'en_curso' WHERE id = p_file_id AND status NOT IN ('en_curso', 'completado');
    END IF;
END;
$$ LANGUAGE plpgsql;
