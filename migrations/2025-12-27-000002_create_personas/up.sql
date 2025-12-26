-- ========================================================================
-- TABLA PERSONAS
-- Datos personales base para usuarios, conductores, guías, pasajeros
-- ========================================================================

CREATE TABLE personas (
    id SERIAL PRIMARY KEY,
    tipo_documento VARCHAR(30) NOT NULL DEFAULT 'DNI',
    nro_documento VARCHAR(20) NOT NULL UNIQUE,
    nombre VARCHAR(100) NOT NULL,
    apellidos VARCHAR(100) NOT NULL,
    telefono VARCHAR(20),
    correo VARCHAR(255),
    fecha_nacimiento DATE,
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER,
    updated_by INTEGER
);

-- Índices para búsquedas frecuentes
CREATE INDEX idx_personas_nro_documento ON personas(nro_documento);
CREATE INDEX idx_personas_nombre_apellidos ON personas(nombre, apellidos);
CREATE INDEX idx_personas_tipo_documento ON personas(tipo_documento);
CREATE INDEX idx_personas_correo ON personas(correo) WHERE correo IS NOT NULL;

-- Trigger para updated_at
CREATE TRIGGER update_personas_updated_at
    BEFORE UPDATE ON personas
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
