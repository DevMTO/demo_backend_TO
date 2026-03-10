-- Restaurar precio_base en tours
ALTER TABLE tours ADD COLUMN precio_base DECIMAL(10,2) NOT NULL DEFAULT 0;

-- Restaurar precios desde tarifas de agencias
UPDATE tours SET precio_base = t.precio
FROM tarifas t
WHERE t.id_tour = tours.id AND t.tipo_entidad = 'agencias';

DROP TABLE IF EXISTS tarifas;
