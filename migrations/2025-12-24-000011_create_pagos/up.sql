-- Migration: Create pagos table
-- Pagos y movimientos financieros

CREATE TABLE pagos (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    id_file UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tipo_movimiento VARCHAR(30) NOT NULL DEFAULT 'ingreso',
    concepto VARCHAR(200) NOT NULL,
    monto DECIMAL(12,2) NOT NULL,
    metodo_pago VARCHAR(50),  -- "efectivo", "transferencia", "tarjeta", "yape", "plin"
    referencia VARCHAR(100),   -- Número de operación/referencia
    evidencia JSONB DEFAULT '{}',  -- {"comprobante_url": "...", "tipo": "boleta/factura", "numero": "..."}
    fecha_pago TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    notas TEXT,
    
    -- Auditoría
    registrado_por UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT chk_pagos_tipo CHECK (
        tipo_movimiento IN ('ingreso', 'egreso', 'adelanto', 'saldo', 'reembolso')
    ),
    CONSTRAINT chk_pagos_monto CHECK (monto != 0)
);

-- Indexes
CREATE INDEX idx_pagos_id_file ON pagos(id_file);
CREATE INDEX idx_pagos_tipo_movimiento ON pagos(tipo_movimiento);
CREATE INDEX idx_pagos_fecha_pago ON pagos(fecha_pago DESC);
CREATE INDEX idx_pagos_metodo_pago ON pagos(metodo_pago);

-- Trigger
CREATE TRIGGER update_pagos_updated_at
    BEFORE UPDATE ON pagos
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Trigger para actualizar monto_pagado en files
CREATE OR REPLACE FUNCTION update_file_monto_pagado()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        UPDATE files 
        SET monto_pagado = COALESCE((
            SELECT SUM(CASE WHEN tipo_movimiento IN ('ingreso', 'adelanto') THEN monto 
                           WHEN tipo_movimiento IN ('egreso', 'reembolso') THEN -monto 
                           ELSE 0 END)
            FROM pagos WHERE id_file = OLD.id_file
        ), 0)
        WHERE id = OLD.id_file;
        RETURN OLD;
    ELSE
        UPDATE files 
        SET monto_pagado = COALESCE((
            SELECT SUM(CASE WHEN tipo_movimiento IN ('ingreso', 'adelanto') THEN monto 
                           WHEN tipo_movimiento IN ('egreso', 'reembolso') THEN -monto 
                           ELSE 0 END)
            FROM pagos WHERE id_file = NEW.id_file
        ), 0)
        WHERE id = NEW.id_file;
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_file_monto
    AFTER INSERT OR UPDATE OR DELETE ON pagos
    FOR EACH ROW
    EXECUTE FUNCTION update_file_monto_pagado();
