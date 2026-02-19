-- ========================================================================
-- MIGRACIÓN: Corregir automatización para recuperar días atrasados
--
-- Problema: si el backend estuvo apagado varios días, la función diaria
-- no encontraba file_tours atrasados porque:
-- - PASO 1 solo buscaba fecha_tour = CURRENT_DATE (no días pasados)
-- - PASO 2 requería status 'en_curso', pero los atrasados seguían en 'asignado'
--
-- Solución: PASO 1 usa fecha_tour <= CURRENT_DATE para incluir atrasados
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
    -- PASO 1: asignado → en_curso
    --         file_tour en 'confirmado' o 'asignado', fecha_tour <= hoy
    --         (usa <= para recuperar días atrasados si el backend estuvo apagado)
    -- =================================================================
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status IN ('confirmado', 'asignado')
          AND fecha_tour <= CURRENT_DATE
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
    -- PASO 2: en_curso → completado (fecha_tour < hoy)
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
