-- ========================================================================
-- TABLAS DE RELACIÓN PARA FILES (junction tables)
-- ========================================================================

-- Guías asignados a un file
CREATE TABLE file_guias (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    id_guia INTEGER NOT NULL REFERENCES guias(id) ON DELETE CASCADE,
    rol VARCHAR(30) DEFAULT 'asistente',
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT uq_file_guias UNIQUE (id_file, id_guia)
);

CREATE INDEX idx_file_guias_id_file ON file_guias(id_file);
CREATE INDEX idx_file_guias_id_guia ON file_guias(id_guia);

-- Pasajeros asignados a un file
CREATE TABLE file_pasajeros (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    id_persona INTEGER NOT NULL REFERENCES personas(id) ON DELETE CASCADE,
    asiento VARCHAR(10),
    tipo_pasajero VARCHAR(30) DEFAULT 'adulto',
    notas TEXT,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT uq_file_pasajeros UNIQUE (id_file, id_persona)
);

CREATE INDEX idx_file_pasajeros_id_file ON file_pasajeros(id_file);
CREATE INDEX idx_file_pasajeros_id_persona ON file_pasajeros(id_persona);

-- Vehículos asignados a un file (con su conductor)
CREATE TABLE file_vehiculos (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    id_vehiculo INTEGER NOT NULL REFERENCES vehiculos(id) ON DELETE CASCADE,
    id_conductor INTEGER REFERENCES conductores(id) ON DELETE SET NULL,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT uq_file_vehiculos UNIQUE (id_file, id_vehiculo)
);

CREATE INDEX idx_file_vehiculos_id_file ON file_vehiculos(id_file);
CREATE INDEX idx_file_vehiculos_id_vehiculo ON file_vehiculos(id_vehiculo);
CREATE INDEX idx_file_vehiculos_id_conductor ON file_vehiculos(id_conductor) WHERE id_conductor IS NOT NULL;

-- Restaurantes asignados a un file
CREATE TABLE file_restaurantes (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    id_restaurante INTEGER NOT NULL REFERENCES restaurantes(id) ON DELETE CASCADE,
    tipo_servicio VARCHAR(30) DEFAULT 'almuerzo',
    dia INTEGER DEFAULT 1,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_file_restaurantes_id_file ON file_restaurantes(id_file);
CREATE INDEX idx_file_restaurantes_id_restaurante ON file_restaurantes(id_restaurante);

-- Entradas asignadas a un file
CREATE TABLE file_entradas (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    id_entrada INTEGER NOT NULL REFERENCES entradas(id) ON DELETE CASCADE,
    cantidad INTEGER NOT NULL DEFAULT 1,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT uq_file_entradas UNIQUE (id_file, id_entrada)
);

CREATE INDEX idx_file_entradas_id_file ON file_entradas(id_file);
CREATE INDEX idx_file_entradas_id_entrada ON file_entradas(id_entrada);
