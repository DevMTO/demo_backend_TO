-- Actualizar constraint único de file_entradas
-- ANTES: UNIQUE(id_file_tour, id_entrada) - Solo una entrada por tour
-- DESPUÉS: UNIQUE(id_file_tour, id_entrada, COALESCE(id_entrada_precio, -1))
--          Permite múltiples entradas con diferentes precios

-- Eliminar el constraint único actual
ALTER TABLE file_entradas DROP CONSTRAINT IF EXISTS uq_file_tour_entradas;

-- Crear nuevo constraint que considera el precio
-- Usa COALESCE para manejar NULL (cuando no hay precio específico)
CREATE UNIQUE INDEX uq_file_entradas_tour_precio ON file_entradas (
    id_file_tour, 
    id_entrada, 
    COALESCE(id_entrada_precio, -1)
);

-- Comentario documentando el cambio
COMMENT ON INDEX uq_file_entradas_tour_precio IS 
'Constraint único: permite misma entrada múltiples veces si tiene diferente precio. NULL se trata como -1 para uniqueness.';
