-- Add encargado field to restaurantes table
ALTER TABLE restaurantes ADD COLUMN encargado INTEGER REFERENCES personas(id) ON DELETE SET NULL;

-- Create index for encargado foreign key
CREATE INDEX idx_restaurantes_encargado ON restaurantes(encargado);
