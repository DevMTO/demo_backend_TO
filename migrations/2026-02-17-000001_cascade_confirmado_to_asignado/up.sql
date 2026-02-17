-- ========================================================================
-- MIGRACIÓN: Cascada confirmado → asignado
--
-- Cuando un file pasa a "confirmado":
--   → sus file_tours pasan a "confirmado"
--     → file_restaurantes y file_entradas pasan a "asignado"
--     → file_guias y file_vehiculos NO (se asignan en otro flujo)
--
-- También actualiza las funciones anteriores para incluir file_entradas
-- ========================================================================

-- ========================================================================
-- 1) TRIGGER FUNCTION: Cuando un file cambia a "confirmado"
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

-- Trigger en files
CREATE TRIGGER trg_files_status_change
    AFTER UPDATE OF status ON files
    FOR EACH ROW
    EXECUTE FUNCTION trg_after_file_status_change();

-- ========================================================================
-- 2) ACTUALIZAR trigger de file_tours para manejar "confirmado" → sub-files "asignado"
--    (redefine la función existente para agregar este caso + incluir file_entradas)
-- ========================================================================
CREATE OR REPLACE FUNCTION trg_after_file_tour_status_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Cuando file_tour pasa a "confirmado", restaurantes y entradas pasan a "asignado"
    -- (guias y vehiculos NO, se asignan en otro flujo separado)
    IF NEW.status = 'confirmado' AND (OLD.status IS NULL OR OLD.status != 'confirmado') THEN
        UPDATE file_restaurantes SET status = 'asignado'
            WHERE id_file_tour = NEW.id AND status IN ('pendiente', 'reservado');
        UPDATE file_entradas SET status = 'asignado'
            WHERE id_file_tour = NEW.id AND status IN ('pendiente', 'reservado');
    END IF;

    -- Cuando file_tour pasa a "en_curso" o "completado", cascada al file padre
    IF NEW.status IN ('en_curso', 'completado') AND (OLD.status IS NULL OR OLD.status != NEW.status) THEN
        PERFORM check_and_update_file_status(NEW.id_file);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ========================================================================
-- 3) ACTUALIZAR trigger de sub-files para incluir file_entradas
-- ========================================================================

-- Trigger en file_entradas (no existía antes)
CREATE TRIGGER trg_file_entradas_status_change
    AFTER INSERT OR UPDATE OF status ON file_entradas
    FOR EACH ROW
    EXECUTE FUNCTION trg_after_subfile_status_change();

-- ========================================================================
-- 4) ACTUALIZAR check_and_update_file_tour_status para incluir file_entradas
-- ========================================================================
CREATE OR REPLACE FUNCTION check_and_update_file_tour_status(p_file_tour_id INT)
RETURNS VOID AS $$
DECLARE
    v_file_tour RECORD;
    v_all_asignado BOOLEAN;
    v_has_subfiles BOOLEAN;
BEGIN
    SELECT id, id_file, status, fecha_tour
    INTO v_file_tour
    FROM file_tours
    WHERE id = p_file_tour_id;

    IF NOT FOUND THEN
        RETURN;
    END IF;

    IF v_file_tour.status != 'asignado' THEN
        RETURN;
    END IF;

    IF v_file_tour.fecha_tour IS NULL OR v_file_tour.fecha_tour != CURRENT_DATE THEN
        RETURN;
    END IF;

    v_has_subfiles := FALSE;
    IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = p_file_tour_id) THEN
        v_has_subfiles := TRUE;
    END IF;
    IF EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = p_file_tour_id) THEN
        v_has_subfiles := TRUE;
    END IF;
    IF EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = p_file_tour_id) THEN
        v_has_subfiles := TRUE;
    END IF;
    IF EXISTS (SELECT 1 FROM file_entradas WHERE id_file_tour = p_file_tour_id) THEN
        v_has_subfiles := TRUE;
    END IF;

    IF NOT v_has_subfiles THEN
        RETURN;
    END IF;

    v_all_asignado := TRUE;
    IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = p_file_tour_id AND status != 'asignado') THEN
        v_all_asignado := FALSE;
    END IF;
    IF v_all_asignado AND EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = p_file_tour_id AND status != 'asignado') THEN
        v_all_asignado := FALSE;
    END IF;
    IF v_all_asignado AND EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = p_file_tour_id AND status != 'asignado') THEN
        v_all_asignado := FALSE;
    END IF;
    IF v_all_asignado AND EXISTS (SELECT 1 FROM file_entradas WHERE id_file_tour = p_file_tour_id AND status != 'asignado') THEN
        v_all_asignado := FALSE;
    END IF;

    IF NOT v_all_asignado THEN
        RETURN;
    END IF;

    UPDATE file_guias SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_vehiculos SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_restaurantes SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_entradas SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';

    UPDATE file_tours SET status = 'en_curso' WHERE id = p_file_tour_id;

    PERFORM check_and_update_file_status(v_file_tour.id_file);
END;
$$ LANGUAGE plpgsql;

-- ========================================================================
-- 5) ACTUALIZAR automatizar_estados_por_fecha para incluir file_entradas
-- ========================================================================
CREATE OR REPLACE FUNCTION automatizar_estados_por_fecha()
RETURNS VOID AS $$
DECLARE
    v_ft RECORD;
    v_has_subfiles BOOLEAN;
    v_all_asignado BOOLEAN;
BEGIN
    -- PASO 1: asignado -> en_curso (file_tours con fecha_tour = hoy)
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status = 'asignado'
          AND fecha_tour = CURRENT_DATE
    LOOP
        v_has_subfiles := FALSE;
        IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = v_ft.id) THEN
            v_has_subfiles := TRUE;
        END IF;
        IF EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = v_ft.id) THEN
            v_has_subfiles := TRUE;
        END IF;
        IF EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = v_ft.id) THEN
            v_has_subfiles := TRUE;
        END IF;
        IF EXISTS (SELECT 1 FROM file_entradas WHERE id_file_tour = v_ft.id) THEN
            v_has_subfiles := TRUE;
        END IF;

        IF NOT v_has_subfiles THEN
            CONTINUE;
        END IF;

        v_all_asignado := TRUE;
        IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = v_ft.id AND status != 'asignado') THEN
            v_all_asignado := FALSE;
        END IF;
        IF v_all_asignado AND EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = v_ft.id AND status != 'asignado') THEN
            v_all_asignado := FALSE;
        END IF;
        IF v_all_asignado AND EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = v_ft.id AND status != 'asignado') THEN
            v_all_asignado := FALSE;
        END IF;
        IF v_all_asignado AND EXISTS (SELECT 1 FROM file_entradas WHERE id_file_tour = v_ft.id AND status != 'asignado') THEN
            v_all_asignado := FALSE;
        END IF;

        IF NOT v_all_asignado THEN
            CONTINUE;
        END IF;

        UPDATE file_guias SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_vehiculos SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_restaurantes SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_entradas SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';

        UPDATE file_tours SET status = 'en_curso' WHERE id = v_ft.id;

        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;

    -- PASO 2: en_curso -> completado (file_tours con fecha_tour < hoy)
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status = 'en_curso'
          AND fecha_tour < CURRENT_DATE
    LOOP
        UPDATE file_guias SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_vehiculos SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_restaurantes SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_entradas SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';

        UPDATE file_tours SET status = 'completado' WHERE id = v_ft.id;

        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;
END;
$$ LANGUAGE plpgsql;
