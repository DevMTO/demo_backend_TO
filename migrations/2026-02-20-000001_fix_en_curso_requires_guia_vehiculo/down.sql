-- Revert: restaurar versiones anteriores de las 3 funciones

-- 1) Revert check_and_update_file_tour_status (sin requisito de guías/vehículos)
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

    IF v_file_tour.status NOT IN ('confirmado', 'asignado') THEN
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

-- 2) Revert automatizar_estados_por_fecha (con check estricto en PASO 2)
CREATE OR REPLACE FUNCTION automatizar_estados_por_fecha()
RETURNS VOID AS $$
DECLARE
    v_ft RECORD;
    v_has_subfiles BOOLEAN;
    v_all_asignado BOOLEAN;
    v_all_en_curso BOOLEAN;
BEGIN
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

    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status = 'en_curso'
          AND fecha_tour < CURRENT_DATE
    LOOP
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

        UPDATE file_guias SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_vehiculos SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_restaurantes SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_entradas SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';

        UPDATE file_tours SET status = 'completado' WHERE id = v_ft.id;

        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- 3) Revert check_and_update_file_status (sin excluir cancelados/no_show)
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
    ELSIF v_en_curso_count > 0 THEN
        UPDATE files SET status = 'en_curso' WHERE id = p_file_id AND status NOT IN ('en_curso', 'completado');
    END IF;
END;
$$ LANGUAGE plpgsql;
