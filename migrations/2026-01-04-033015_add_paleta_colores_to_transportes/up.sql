-- Add paleta_colores column to transportes table
-- This mirrors the structure in agencias for brand customization

ALTER TABLE transportes
ADD COLUMN IF NOT EXISTS paleta_colores JSONB DEFAULT '{}';

-- Add comment for documentation
COMMENT ON COLUMN transportes.paleta_colores IS 'Color palette for transport company branding (primario, secundario, cta, etc.)';
