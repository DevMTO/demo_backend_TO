-- ========================================================================
-- MIGRACIÓN: Corregir transiciones del día del tour
--
-- Lógica:
-- 1. Día del tour + todos los sub-files en "asignado" → sub-files a "en_curso", file_tour a "en_curso"
-- 2. Pasó el día del tour + todos los sub-files en "en_curso" → sub-files a "completado", file_tour a "completado"
--
-- Corrección: el file_tour puede estar en 'confirmado' o 'asignado' para la transición a en_curso
-- (antes solo aceptaba 'asignado', pero el file_tour puede no haber llegado a ese estado)
-- ========================================================================

-- ========================================================================
-- 1) ACTUALIZAR check_and_update_file_tour_status
--    Acepta file_tour en 'confirmado' o 'asignado' para transición a en_curso
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

    -- Transición a en_curso: file_tour en 'confirmado' o 'asignado'
    IF v_file_tour.status NOT IN ('confirmado', 'asignado') THEN
        RETURN;
    END IF;

    -- Solo si fecha_tour = hoy
    IF v_file_tour.fecha_tour IS NULL OR v_file_tour.fecha_tour != CURRENT_DATE THEN
        RETURN;
    END IF;

    -- Verificar si tiene sub-files
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

    -- Verificar que TODOS los sub-files existentes estén en 'asignado'
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

    -- Transicionar sub-files a 'en_curso'
    UPDATE file_guias SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_vehiculos SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_restaurantes SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_entradas SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';

    -- File_tour a 'en_curso'
    UPDATE file_tours SET status = 'en_curso' WHERE id = p_file_tour_id;

    PERFORM check_and_update_file_status(v_file_tour.id_file);
END;
$$ LANGUAGE plpgsql;

-- ========================================================================
-- 2) ACTUALIZAR automatizar_estados_por_fecha
--    Acepta file_tour en 'confirmado' o 'asignado' para PASO 1
-- ========================================================================
CREATE OR REPLACE FUNCTION automatizar_estados_por_fecha()
RETURNS VOID AS $$
DECLARE
    v_ft RECORD;
    v_has_subfiles BOOLEAN;
    v_all_asignado BOOLEAN;
    v_all_en_curso BOOLEAN;
BEGIN
    -- =================================================================
    -- PASO 1: Día del tour → sub-files asignado → en_curso
    --         file_tour en 'confirmado' o 'asignado', fecha_tour = hoy
    -- =================================================================
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status IN ('confirmado', 'asignado')
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

        -- Verificar que TODOS los sub-files estén en 'asignado'
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

        -- Sub-files a 'en_curso'
        UPDATE file_guias SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_vehiculos SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_restaurantes SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_entradas SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';

        -- File_tour a 'en_curso'
        UPDATE file_tours SET status = 'en_curso' WHERE id = v_ft.id;

        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;

    -- =================================================================
    -- PASO 2: Pasó el día → sub-files en_curso → completado
    --         file_tour en 'en_curso', fecha_tour < hoy
    -- =================================================================
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status = 'en_curso'
          AND fecha_tour < CURRENT_DATE
    LOOP
        -- Verificar que TODOS los sub-files estén en 'en_curso'
        v_all_en_curso := TRUE;
        IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = v_ft.id AND status != 'en_curso') THEN
            v_all_en_curso := FALSE;
        END IF;
        IF v_all_en_curso AND EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = v_ft.id AND status != 'en_curso') THEN
            v_all_en_curso := FALSE;
        END IF;
        IF v_all_en_curso AND EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = v_ft.id AND status != 'en_curso') THEN
            v_all_en_curso := FALSE;
        END IF;
        IF v_all_en_curso AND EXISTS (SELECT 1 FROM file_entradas WHERE id_file_tour = v_ft.id AND status != 'en_curso') THEN
            v_all_en_curso := FALSE;
        END IF;

        IF NOT v_all_en_curso THEN
            CONTINUE;
        END IF;

        -- Sub-files a 'completado'
        UPDATE file_guias SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_vehiculos SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_restaurantes SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_entradas SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';

        -- File_tour a 'completado'
        UPDATE file_tours SET status = 'completado' WHERE id = v_ft.id;

        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;
END;
$$ LANGUAGE plpgsql;
