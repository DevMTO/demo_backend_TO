-- Revertir fecha_tour de file_tours
ALTER TABLE file_tours DROP COLUMN fecha_tour;

-- Revertir precio de file_restaurantes
ALTER TABLE file_restaurantes DROP COLUMN precio;
