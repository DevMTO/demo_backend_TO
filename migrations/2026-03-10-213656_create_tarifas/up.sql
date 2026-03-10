-- Tabla de tarifas: precios por tour según tipo de entidad
CREATE TABLE tarifas (
    id SERIAL PRIMARY KEY,
    id_tour INTEGER NOT NULL REFERENCES tours(id) ON DELETE CASCADE,
    tipo_entidad VARCHAR(50) NOT NULL,  -- 'agencias', 'hoteles', etc.
    precio DECIMAL(10,2) NOT NULL DEFAULT 0,
    descripcion TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),
    updated_by INTEGER REFERENCES users(id),
    UNIQUE(id_tour, tipo_entidad)
);

-- Trigger de updated_at
SELECT diesel_manage_updated_at('tarifas');

-- Migrar precio_base existente como tarifa de agencias
INSERT INTO tarifas (id_tour, tipo_entidad, precio, descripcion)
SELECT id, 'agencias', precio_base, 'Tarifa base migrada desde precio_base del tour'
FROM tours
WHERE precio_base > 0;

-- Eliminar precio_base de tours (ahora vive en tarifas)
ALTER TABLE tours DROP COLUMN precio_base;
