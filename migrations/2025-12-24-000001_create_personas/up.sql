-- Migration: Create personas table
-- Tabla base para almacenar información personal (usada por usuarios, conductores, guías, etc.)

CREATE TABLE personas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tipo_documento VARCHAR(30) NOT NULL DEFAULT 'DNI',
    nro_documento VARCHAR(20) NOT NULL UNIQUE,
    nombre VARCHAR(100) NOT NULL,
    apellidos VARCHAR(100) NOT NULL,
    telefono VARCHAR(20),
    correo VARCHAR(255),
    fecha_nacimiento DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_personas_tipo_documento CHECK (
        tipo_documento IN ('DNI', 'PASAPORTE', 'CARNET_EXTRANJERIA', 'RUC', 'OTRO')
    ),
    CONSTRAINT chk_personas_correo_format CHECK (
        correo IS NULL OR correo ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}$'
    )
);

-- Indexes
CREATE INDEX idx_personas_nro_documento ON personas(nro_documento);
CREATE INDEX idx_personas_nombre ON personas(nombre);
CREATE INDEX idx_personas_apellidos ON personas(apellidos);
CREATE INDEX idx_personas_tipo_documento ON personas(tipo_documento);

-- Trigger para updated_at
CREATE TRIGGER update_personas_updated_at
    BEFORE UPDATE ON personas
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Agregar relación opcional a users (un usuario puede tener una persona asociada)
ALTER TABLE users ADD COLUMN id_persona UUID REFERENCES personas(id) ON DELETE SET NULL;
CREATE INDEX idx_users_id_persona ON users(id_persona);
