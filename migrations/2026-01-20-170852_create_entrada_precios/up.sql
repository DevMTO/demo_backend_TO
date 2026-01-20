-- ========================================================================
-- MIGRACIÓN: Crear tabla entrada_precios
-- Distribución de precios por entrada según tipo y rango de edad
-- ========================================================================
--
-- ESTRUCTURA DE PRECIOS:
-- - tipo_precio: 'general', 'nacional', 'extranjero'
--   * 'general' = mismo precio para nacional y extranjero
--   * 'nacional' = precio específico para turistas nacionales
--   * 'extranjero' = precio específico para turistas extranjeros
--
-- - Rangos de edad:
--   * 0-8 años: generalmente gratis (precio = 0)
--   * 9-16 años: precio de niño/adolescente
--   * 17+ años: precio de adulto
--   * (Futuro: tercera edad)
-- ========================================================================

-- Primero eliminamos el precio de la tabla entradas
-- ya que ahora los precios estarán en entrada_precios
ALTER TABLE entradas DROP COLUMN IF EXISTS precio;
ALTER TABLE entradas DROP COLUMN IF EXISTS tipo;

-- Eliminar índices que dependen de las columnas eliminadas
DROP INDEX IF EXISTS idx_entradas_precio;
DROP INDEX IF EXISTS idx_entradas_tipo;

-- Crear tabla de precios por entrada
CREATE TABLE entrada_precios (
    id SERIAL PRIMARY KEY,
    id_entrada INTEGER NOT NULL REFERENCES entradas(id) ON DELETE CASCADE,
    
    -- Tipo de precio: 'general', 'nacional', 'extranjero'
    -- No usamos ENUM para flexibilidad en el backend
    tipo_precio VARCHAR(30) NOT NULL DEFAULT 'general',
    
    -- Rango de edad
    edad_minima INTEGER NOT NULL DEFAULT 0,
    edad_maxima INTEGER, -- NULL significa "sin límite" (ej: 17+)
    
    -- Precio para este rango
    precio DECIMAL(10,2) NOT NULL DEFAULT 0,
    
    -- Descripción del rango (ej: "Niños", "Adultos", "Tercera edad")
    descripcion VARCHAR(100),
    
    -- Campos de auditoría
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    updated_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    -- Constraint: No puede haber rangos duplicados para misma entrada y tipo
    CONSTRAINT uq_entrada_precio_rango UNIQUE (id_entrada, tipo_precio, edad_minima)
);

-- Índices para búsquedas eficientes
CREATE INDEX idx_entrada_precios_entrada ON entrada_precios(id_entrada);
CREATE INDEX idx_entrada_precios_tipo ON entrada_precios(tipo_precio);
CREATE INDEX idx_entrada_precios_edad ON entrada_precios(edad_minima, edad_maxima);

-- Trigger para updated_at
CREATE TRIGGER update_entrada_precios_updated_at
    BEFORE UPDATE ON entrada_precios
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Comentarios
COMMENT ON TABLE entrada_precios IS 'Distribución de precios por entrada según tipo de turista y rango de edad';
COMMENT ON COLUMN entrada_precios.tipo_precio IS 'Tipo: general (mismo para todos), nacional, extranjero';
COMMENT ON COLUMN entrada_precios.edad_minima IS 'Edad mínima del rango (inclusive)';
COMMENT ON COLUMN entrada_precios.edad_maxima IS 'Edad máxima del rango (inclusive), NULL = sin límite superior';
COMMENT ON COLUMN entrada_precios.precio IS 'Precio en moneda local (0 = gratis)';
COMMENT ON COLUMN entrada_precios.descripcion IS 'Descripción del rango, ej: Niños, Adultos, Tercera edad';

-- ========================================================================
-- Migrar datos existentes: crear precios "general" para entradas existentes
-- ========================================================================
-- Nota: Las entradas existentes tendrán precios por defecto en 0
-- El administrador deberá configurar los precios manualmente
-- ========================================================================

-- Insertar un precio general adulto por defecto para cada entrada existente
INSERT INTO entrada_precios (id_entrada, tipo_precio, edad_minima, edad_maxima, precio, descripcion)
SELECT 
    id,
    'general',
    17,
    NULL,
    0,
    'Adulto'
FROM entradas;

-- Insertar precio general niños (0-8 gratis)
INSERT INTO entrada_precios (id_entrada, tipo_precio, edad_minima, edad_maxima, precio, descripcion)
SELECT 
    id,
    'general',
    0,
    8,
    0,
    'Niño (gratis)'
FROM entradas;

-- Insertar precio general adolescentes (9-16)
INSERT INTO entrada_precios (id_entrada, tipo_precio, edad_minima, edad_maxima, precio, descripcion)
SELECT 
    id,
    'general',
    9,
    16,
    0,
    'Adolescente'
FROM entradas;
