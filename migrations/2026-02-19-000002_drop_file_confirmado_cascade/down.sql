-- ========================================================================
-- ROLLBACK: Restore file confirmado cascade trigger and function
-- ========================================================================

-- ========================================================================
-- 1) RESTORE TRIGGER FUNCTION: trg_after_file_status_change
-- ========================================================================
CREATE OR REPLACE FUNCTION trg_after_file_status_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Cuando el file pasa a "confirmado", sus file_tours también
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
