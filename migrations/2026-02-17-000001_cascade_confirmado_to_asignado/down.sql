-- Rollback: quitar trigger de files y file_entradas
DROP TRIGGER IF EXISTS trg_files_status_change ON files;
DROP TRIGGER IF EXISTS trg_file_entradas_status_change ON file_entradas;
DROP FUNCTION IF EXISTS trg_after_file_status_change();

-- Restaurar trg_after_file_tour_status_change sin el caso "confirmado" y sin file_entradas
CREATE OR REPLACE FUNCTION trg_after_file_tour_status_change()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.status IN ('en_curso', 'completado') AND (OLD.status IS NULL OR OLD.status != NEW.status) THEN
        PERFORM check_and_update_file_status(NEW.id_file);
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Restaurar check_and_update_file_tour_status sin file_entradas
CREATE OR REPLACE FUNCTION check_and_update_file_tour_status(p_file_tour_id INT)
RETURNS VOID AS $$
DECLARE
    v_file_tour RECORD;
    v_all_asignado BOOLEAN;
    v_has_subfiles BOOLEAN;
BEGIN
    SELECT id, id_file, status, fecha_tour INTO v_file_tour FROM file_tours WHERE id = p_file_tour_id;
    IF NOT FOUND THEN RETURN; END IF;
    IF v_file_tour.status != 'asignado' THEN RETURN; END IF;
    IF v_file_tour.fecha_tour IS NULL OR v_file_tour.fecha_tour != CURRENT_DATE THEN RETURN; END IF;

    v_has_subfiles := FALSE;
    IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = p_file_tour_id) THEN v_has_subfiles := TRUE; END IF;
    IF EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = p_file_tour_id) THEN v_has_subfiles := TRUE; END IF;
    IF EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = p_file_tour_id) THEN v_has_subfiles := TRUE; END IF;
    IF NOT v_has_subfiles THEN RETURN; END IF;

    v_all_asignado := TRUE;
    IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = p_file_tour_id AND status != 'asignado') THEN v_all_asignado := FALSE; END IF;
    IF v_all_asignado AND EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = p_file_tour_id AND status != 'asignado') THEN v_all_asignado := FALSE; END IF;
    IF v_all_asignado AND EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = p_file_tour_id AND status != 'asignado') THEN v_all_asignado := FALSE; END IF;
    IF NOT v_all_asignado THEN RETURN; END IF;

    UPDATE file_guias SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_vehiculos SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_restaurantes SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_tours SET status = 'en_curso' WHERE id = p_file_tour_id;
    PERFORM check_and_update_file_status(v_file_tour.id_file);
END;
$$ LANGUAGE plpgsql;

-- Restaurar automatizar_estados_por_fecha sin file_entradas
CREATE OR REPLACE FUNCTION automatizar_estados_por_fecha()
RETURNS VOID AS $$
DECLARE
    v_ft RECORD;
    v_has_subfiles BOOLEAN;
    v_all_asignado BOOLEAN;
BEGIN
    FOR v_ft IN SELECT id, id_file FROM file_tours WHERE status = 'asignado' AND fecha_tour = CURRENT_DATE LOOP
        v_has_subfiles := FALSE;
        IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = v_ft.id) THEN v_has_subfiles := TRUE; END IF;
        IF EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = v_ft.id) THEN v_has_subfiles := TRUE; END IF;
        IF EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = v_ft.id) THEN v_has_subfiles := TRUE; END IF;
        IF NOT v_has_subfiles THEN CONTINUE; END IF;

        v_all_asignado := TRUE;
        IF EXISTS (SELECT 1 FROM file_guias WHERE id_file_tour = v_ft.id AND status != 'asignado') THEN v_all_asignado := FALSE; END IF;
        IF v_all_asignado AND EXISTS (SELECT 1 FROM file_vehiculos WHERE id_file_tour = v_ft.id AND status != 'asignado') THEN v_all_asignado := FALSE; END IF;
        IF v_all_asignado AND EXISTS (SELECT 1 FROM file_restaurantes WHERE id_file_tour = v_ft.id AND status != 'asignado') THEN v_all_asignado := FALSE; END IF;
        IF NOT v_all_asignado THEN CONTINUE; END IF;

        UPDATE file_guias SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_vehiculos SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_restaurantes SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_tours SET status = 'en_curso' WHERE id = v_ft.id;
        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;

    FOR v_ft IN SELECT id, id_file FROM file_tours WHERE status = 'en_curso' AND fecha_tour < CURRENT_DATE LOOP
        UPDATE file_guias SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_vehiculos SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_restaurantes SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_tours SET status = 'completado' WHERE id = v_ft.id;
        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;
END;
$$ LANGUAGE plpgsql;
