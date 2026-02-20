-- ========================================================================
-- MIGRACIÓN: Fix transiciones de estado automáticas
--
-- 3 problemas corregidos:
--
-- 1) check_and_update_file_tour_status: Al confirmar un file del mismo día,
--    la cadena de triggers transicionaba a en_curso inmediatamente porque
--    solo verificaba sub-files EXISTENTES sin requerir guías y vehículos.
--    FIX: Requiere file_guias Y file_vehiculos para transicionar a en_curso.
--
-- 2) automatizar_estados_por_fecha PASO 2: Exigía que TODOS los sub-files
--    estuvieran en 'en_curso' para pasar a completado. Si algún sub-file
--    quedaba en 'asignado' o 'confirmado', el file_tour se atascaba en
--    en_curso para siempre.
--    FIX: Si fecha_tour < hoy, transicionar todos los sub-files no finales
--    directamente a completado sin importar su estado intermedio.
--
-- 3) check_and_update_file_status: Requería que TODOS los file_tours fueran
--    'completado' (v_completado_count = v_total_tours). Si había un file_tour
--    cancelado o no_show, el file nunca pasaba a completado.
--    FIX: Excluir file_tours en estados finales (cancelado, no_show) del conteo.
-- ========================================================================

-- ========================================================================
-- 1) FIX: check_and_update_file_tour_status
--    Requiere guías Y vehículos para transicionar a en_curso
-- ========================================================================
CREATE OR REPLACE FUNCTION check_and_update_file_tour_status(p_file_tour_id INT)
RETURNS VOID AS $$
DECLARE
    v_file_tour RECORD;
    v_all_asignado BOOLEAN;
    v_has_guias BOOLEAN;
    v_has_vehiculos BOOLEAN;
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

    -- REQUIERE que existan guías Y vehículos asignados
    -- Sin guía y vehículo, no se puede iniciar el tour
    v_has_guias := EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = p_file_tour_id);
    v_has_vehiculos := EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = p_file_tour_id);

    IF NOT v_has_guias OR NOT v_has_vehiculos THEN
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
-- 2) FIX: automatizar_estados_por_fecha
--    PASO 1: Requiere guías Y vehículos para en_curso
--    PASO 2: No exige que todos sub-files estén en 'en_curso'.
--            Si fecha_tour < hoy, todo lo no-final pasa a completado.
-- ========================================================================
CREATE OR REPLACE FUNCTION automatizar_estados_por_fecha()
RETURNS VOID AS $$
DECLARE
    v_ft RECORD;
    v_has_guias BOOLEAN;
    v_has_vehiculos BOOLEAN;
    v_all_asignado BOOLEAN;
BEGIN
    -- =================================================================
    -- PASO 1: asignado → en_curso
    --         file_tour en 'confirmado' o 'asignado', fecha_tour <= hoy
    --         REQUIERE guías Y vehículos asignados
    -- =================================================================
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status IN ('confirmado', 'asignado')
          AND fecha_tour <= CURRENT_DATE
    LOOP
        -- Requiere guías Y vehículos
        v_has_guias := EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = v_ft.id);
        v_has_vehiculos := EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = v_ft.id);

        IF NOT v_has_guias OR NOT v_has_vehiculos THEN
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
    -- PASO 2: en_curso → completado (fecha_tour < hoy)
    --         El tour ya pasó. Todos los sub-files no finales pasan a
    --         completado directamente, sin importar su estado intermedio.
    -- =================================================================
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status = 'en_curso'
          AND fecha_tour < CURRENT_DATE
    LOOP
        -- Sub-files a 'completado' (todo lo que no sea cancelado/no_show/completado)
        UPDATE file_guias SET status = 'completado'
            WHERE id_file_tour = v_ft.id AND status NOT IN ('completado', 'cancelado', 'no_show');
        UPDATE file_vehiculos SET status = 'completado'
            WHERE id_file_tour = v_ft.id AND status NOT IN ('completado', 'cancelado', 'no_show');
        UPDATE file_restaurantes SET status = 'completado'
            WHERE id_file_tour = v_ft.id AND status NOT IN ('completado', 'cancelado', 'no_show');
        UPDATE file_entradas SET status = 'completado'
            WHERE id_file_tour = v_ft.id AND status NOT IN ('completado', 'cancelado', 'no_show');

        -- File_tour a 'completado'
        UPDATE file_tours SET status = 'completado' WHERE id = v_ft.id;

        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- ========================================================================
-- 3) FIX: check_and_update_file_status
--    Excluir file_tours cancelados/no_show del conteo.
--    Si todos los tours "activos" están completados → file a completado.
--    Si al menos uno está en_curso → file a en_curso.
-- ========================================================================
CREATE OR REPLACE FUNCTION check_and_update_file_status(p_file_id INT)
RETURNS VOID AS $$
DECLARE
    v_file_status VARCHAR(30);
    v_active_tours INT;
    v_en_curso_count INT;
    v_completado_count INT;
BEGIN
    SELECT status INTO v_file_status FROM files WHERE id = p_file_id;

    IF NOT FOUND THEN
        RETURN;
    END IF;

    -- Contar solo tours activos (excluir cancelados y no_show)
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE status = 'en_curso'),
        COUNT(*) FILTER (WHERE status = 'completado')
    INTO v_active_tours, v_en_curso_count, v_completado_count
    FROM file_tours
    WHERE id_file = p_file_id
      AND status NOT IN ('cancelado', 'no_show');

    IF v_active_tours = 0 THEN
        RETURN;
    END IF;

    -- Si TODOS los tours activos están completados → file a 'completado'
    IF v_completado_count = v_active_tours THEN
        UPDATE files SET status = 'completado' WHERE id = p_file_id AND status != 'completado';
    -- Si AL MENOS UNO está en 'en_curso' → file a 'en_curso'
    ELSIF v_en_curso_count > 0 THEN
        UPDATE files SET status = 'en_curso' WHERE id = p_file_id AND status NOT IN ('en_curso', 'completado');
    END IF;
END;
$$ LANGUAGE plpgsql;
