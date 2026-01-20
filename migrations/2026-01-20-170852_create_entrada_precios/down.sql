-- ========================================================================
-- ROLLBACK: Eliminar tabla entrada_precios y restaurar columnas en entradas
-- ========================================================================

-- Eliminar trigger
DROP TRIGGER IF EXISTS update_entrada_precios_updated_at ON entrada_precios;

-- Eliminar índices
DROP INDEX IF EXISTS idx_entrada_precios_entrada;
DROP INDEX IF EXISTS idx_entrada_precios_tipo;
DROP INDEX IF EXISTS idx_entrada_precios_edad;

-- Eliminar tabla
DROP TABLE IF EXISTS entrada_precios;

-- Restaurar columnas en entradas
ALTER TABLE entradas ADD COLUMN precio DECIMAL(10,2) NOT NULL DEFAULT 0;
ALTER TABLE entradas ADD COLUMN tipo VARCHAR(50) NOT NULL DEFAULT 'general';

-- Recrear índices originales
CREATE INDEX IF NOT EXISTS idx_entradas_precio ON entradas(precio);
CREATE INDEX IF NOT EXISTS idx_entradas_tipo ON entradas(tipo);
