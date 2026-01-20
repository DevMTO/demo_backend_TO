-- Agregar fecha_tour a file_tours para poder especificar la fecha de cada tour individualmente
-- cuando los tours de un mismo file se realizan en fechas diferentes
ALTER TABLE file_tours ADD COLUMN fecha_tour DATE;

-- Agregar precio a file_restaurantes para poder especificar el costo del servicio de restaurante
-- y poder incluirlo en el cálculo total del file
ALTER TABLE file_restaurantes ADD COLUMN precio DECIMAL(12, 2);
