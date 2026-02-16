-- ========================================================================
-- MIGRACIÓN: Automatización de estados de file_tour y file
--
-- Componentes:
-- 1. Función auxiliar: check_and_update_file_tour_status(p_file_tour_id)
-- 2. Función auxiliar: check_and_update_file_status(p_file_id)
-- 3. Trigger function: trg_after_subfile_status_change()
-- 4. Trigger function: trg_after_file_tour_status_change()
-- 5. Triggers en file_guias, file_vehiculos, file_restaurantes, file_tours
-- 6. Función diaria: automatizar_estados_por_fecha()
-- ========================================================================

-- ========================================================================
-- 1) FUNCIÓN AUXILIAR: Verificar y actualizar status de un file_tour
-- ========================================================================
CREATE OR REPLACE FUNCTION check_and_update_file_tour_status(p_file_tour_id INT)
RETURNS VOID AS $$
DECLARE
    v_file_tour RECORD;
    v_all_asignado BOOLEAN;
    v_has_subfiles BOOLEAN;
BEGIN
    -- Obtener el file_tour
    SELECT id, id_file, status, fecha_tour
    INTO v_file_tour
    FROM file_tours
    WHERE id = p_file_tour_id;

    IF NOT FOUND THEN
        RETURN;
    END IF;

    -- Solo actuar si el file_tour está en 'asignado'
    IF v_file_tour.status != 'asignado' THEN
        RETURN;
    END IF;

    -- Solo actuar si fecha_tour = hoy
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

    -- Si no tiene sub-files, no hacer nada
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

    IF NOT v_all_asignado THEN
        RETURN;
    END IF;

    -- Todas las condiciones cumplidas: cambiar sub-files a 'en_curso'
    UPDATE file_guias SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_vehiculos SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';
    UPDATE file_restaurantes SET status = 'en_curso' WHERE id_file_tour = p_file_tour_id AND status = 'asignado';

    -- Cambiar file_tour a 'en_curso'
    UPDATE file_tours SET status = 'en_curso' WHERE id = p_file_tour_id;

    -- Cascada al file padre
    PERFORM check_and_update_file_status(v_file_tour.id_file);
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION check_and_update_file_tour_status(INT) IS 'Verifica si un file_tour con fecha=hoy y todos sus sub-files asignados debe pasar a en_curso';

-- ========================================================================
-- 2) FUNCIÓN AUXILIAR: Verificar y actualizar status de un file
-- ========================================================================
CREATE OR REPLACE FUNCTION check_and_update_file_status(p_file_id INT)
RETURNS VOID AS $$
DECLARE
    v_file_status VARCHAR(30);
    v_total_tours INT;
    v_en_curso_count INT;
    v_completado_count INT;
BEGIN
    -- Obtener status actual del file
    SELECT status INTO v_file_status FROM files WHERE id = p_file_id;

    IF NOT FOUND THEN
        RETURN;
    END IF;

    -- Contar file_tours y sus estados
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

    -- Si todos los file_tours están completados -> file a 'completado'
    IF v_completado_count = v_total_tours THEN
        UPDATE files SET status = 'completado' WHERE id = p_file_id AND status != 'completado';
    -- Si todos están en 'en_curso' o 'completado' -> file a 'en_curso'
    ELSIF (v_en_curso_count + v_completado_count) = v_total_tours THEN
        UPDATE files SET status = 'en_curso' WHERE id = p_file_id AND status NOT IN ('en_curso', 'completado');
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION check_and_update_file_status(INT) IS 'Verifica si un file debe cambiar a en_curso o completado basado en el status de sus file_tours';

-- ========================================================================
-- 3) TRIGGER FUNCTION: Después de cambio de status en sub-files
-- ========================================================================
CREATE OR REPLACE FUNCTION trg_after_subfile_status_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Solo actuar cuando el status cambia a 'asignado' (INSERT o UPDATE)
    IF NEW.status = 'asignado' THEN
        PERFORM check_and_update_file_tour_status(NEW.id_file_tour);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION trg_after_subfile_status_change() IS 'Trigger: cuando un sub-file cambia a asignado, verifica si el file_tour debe pasar a en_curso';

-- ========================================================================
-- 4) TRIGGER FUNCTION: Después de cambio de status en file_tours
-- ========================================================================
CREATE OR REPLACE FUNCTION trg_after_file_tour_status_change()
RETURNS TRIGGER AS $$
BEGIN
    -- Solo actuar cuando el status cambia a 'en_curso' o 'completado'
    IF NEW.status IN ('en_curso', 'completado') AND (OLD.status IS NULL OR OLD.status != NEW.status) THEN
        PERFORM check_and_update_file_status(NEW.id_file);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION trg_after_file_tour_status_change() IS 'Trigger: cuando un file_tour cambia a en_curso/completado, verifica si el file debe cambiar de status';

-- ========================================================================
-- 5) CREAR TRIGGERS
-- ========================================================================

-- Trigger en file_guias
CREATE TRIGGER trg_file_guias_status_change
    AFTER INSERT OR UPDATE OF status ON file_guias
    FOR EACH ROW
    EXECUTE FUNCTION trg_after_subfile_status_change();

-- Trigger en file_vehiculos
CREATE TRIGGER trg_file_vehiculos_status_change
    AFTER INSERT OR UPDATE OF status ON file_vehiculos
    FOR EACH ROW
    EXECUTE FUNCTION trg_after_subfile_status_change();

-- Trigger en file_restaurantes
CREATE TRIGGER trg_file_restaurantes_status_change
    AFTER INSERT OR UPDATE OF status ON file_restaurantes
    FOR EACH ROW
    EXECUTE FUNCTION trg_after_subfile_status_change();

-- Trigger en file_tours
CREATE TRIGGER trg_file_tours_status_change
    AFTER UPDATE OF status ON file_tours
    FOR EACH ROW
    EXECUTE FUNCTION trg_after_file_tour_status_change();

-- ========================================================================
-- 6) FUNCIÓN DIARIA: automatizar_estados_por_fecha()
-- ========================================================================
CREATE OR REPLACE FUNCTION automatizar_estados_por_fecha()
RETURNS VOID AS $$
DECLARE
    v_ft RECORD;
    v_has_subfiles BOOLEAN;
    v_all_asignado BOOLEAN;
BEGIN
    -- =====================================================================
    -- PASO 1: asignado -> en_curso (file_tours con fecha_tour = hoy)
    -- =====================================================================
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status = 'asignado'
          AND fecha_tour = CURRENT_DATE
    LOOP
        -- Verificar si tiene sub-files
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

        IF NOT v_has_subfiles THEN
            CONTINUE;
        END IF;

        -- Verificar que todos los sub-files existentes estén en 'asignado'
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

        IF NOT v_all_asignado THEN
            CONTINUE;
        END IF;

        -- Actualizar sub-files a 'en_curso'
        UPDATE file_guias SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_vehiculos SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';
        UPDATE file_restaurantes SET status = 'en_curso' WHERE id_file_tour = v_ft.id AND status = 'asignado';

        -- Actualizar file_tour a 'en_curso'
        UPDATE file_tours SET status = 'en_curso' WHERE id = v_ft.id;

        -- Cascada al file
        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;

    -- =====================================================================
    -- PASO 2: en_curso -> completado (file_tours con fecha_tour < hoy)
    -- =====================================================================
    FOR v_ft IN
        SELECT id, id_file
        FROM file_tours
        WHERE status = 'en_curso'
          AND fecha_tour < CURRENT_DATE
    LOOP
        -- Actualizar sub-files a 'completado'
        UPDATE file_guias SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_vehiculos SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';
        UPDATE file_restaurantes SET status = 'completado' WHERE id_file_tour = v_ft.id AND status = 'en_curso';

        -- Actualizar file_tour a 'completado'
        UPDATE file_tours SET status = 'completado' WHERE id = v_ft.id;

        -- Cascada al file
        PERFORM check_and_update_file_status(v_ft.id_file);
    END LOOP;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION automatizar_estados_por_fecha() IS 'Función diaria: transiciona file_tours asignados->en_curso (fecha=hoy) y en_curso->completado (fecha<hoy)';
