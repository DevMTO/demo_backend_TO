-- Agregar status a file_tours (default 'reservado')
ALTER TABLE file_tours 
ADD COLUMN status VARCHAR(30) NOT NULL DEFAULT 'reservado';

-- Comentario para documentar valores posibles
COMMENT ON COLUMN file_tours.status IS 'Estado del file_tour: reservado, confirmado, en_progreso, completado, cancelado';
