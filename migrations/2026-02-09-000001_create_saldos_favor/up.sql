-- Tabla de cancelaciones de files
-- Registra tanto cancelaciones normales (antes de 8PM) como no_shows (después de 8PM)
CREATE TABLE IF NOT EXISTS cancelaciones (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id),
    id_agencia INTEGER NOT NULL REFERENCES agencias(id),
    
    -- Montos
    monto_total_file NUMERIC(12,2) NOT NULL DEFAULT 0,          -- Monto total del file original
    monto_pagado NUMERIC(12,2) NOT NULL DEFAULT 0,              -- Lo que ya había pagado la agencia
    monto_saldo_favor NUMERIC(12,2) NOT NULL DEFAULT 0,         -- Lo que se convierte en saldo a favor
    monto_operador NUMERIC(12,2) NOT NULL DEFAULT 0,            -- Lo que queda para el operador (solo en no_show)
    
    -- Tipo: 'cancelacion' (antes de 8PM, todo pagado → saldo) o 'no_show' (después de 8PM, solo rest+entradas → saldo)
    tipo_cancelacion VARCHAR(30) NOT NULL DEFAULT 'cancelacion',
    
    -- Metadata
    motivo TEXT,
    notas TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),
    
    -- Un file solo puede cancelarse una vez
    CONSTRAINT uq_cancelacion_file UNIQUE (id_file)
);

-- Tabla de no_shows: detalle del desglose cuando es no_show
-- Solo se crea cuando tipo_cancelacion = 'no_show'
CREATE TABLE IF NOT EXISTS no_shows (
    id SERIAL PRIMARY KEY,
    id_cancelacion INTEGER NOT NULL REFERENCES cancelaciones(id),
    id_file INTEGER NOT NULL REFERENCES files(id),
    id_agencia INTEGER NOT NULL REFERENCES agencias(id),
    
    -- Desglose de costos reembolsables (van a saldo a favor)
    monto_restaurantes NUMERIC(12,2) NOT NULL DEFAULT 0,    -- Suma de precios de file_restaurantes del file
    monto_entradas NUMERIC(12,2) NOT NULL DEFAULT 0,        -- Suma de (cantidad × precio) de file_entradas del file
    monto_saldo_favor NUMERIC(12,2) NOT NULL DEFAULT 0,     -- = monto_restaurantes + monto_entradas
    
    -- Lo que retiene el operador
    monto_operador NUMERIC(12,2) NOT NULL DEFAULT 0,        -- = monto_pagado - monto_saldo_favor
    
    -- Fecha/hora de referencia
    fecha_inicio_file DATE NOT NULL,                         -- fecha_inicio más temprana de los file_tours
    hora_corte TIMESTAMPTZ NOT NULL DEFAULT NOW(),           -- Momento en que se registró el no_show
    
    -- Metadata
    notas TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),
    
    -- Un file solo puede tener un registro de no_show
    CONSTRAINT uq_no_show_file UNIQUE (id_file),
    CONSTRAINT uq_no_show_cancelacion UNIQUE (id_cancelacion)
);

-- Tabla de saldos a favor por agencia
-- Acumula los saldos generados por cancelaciones para uso en futuros files
CREATE TABLE IF NOT EXISTS saldos_favor (
    id SERIAL PRIMARY KEY,
    id_agencia INTEGER NOT NULL REFERENCES agencias(id),
    
    -- Saldo acumulado
    saldo_disponible NUMERIC(12,2) NOT NULL DEFAULT 0,     -- Saldo actual disponible
    saldo_utilizado NUMERIC(12,2) NOT NULL DEFAULT 0,      -- Total histórico utilizado
    saldo_total_generado NUMERIC(12,2) NOT NULL DEFAULT 0, -- Total histórico generado
    
    -- Metadata
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Una agencia tiene un solo registro de saldo
    CONSTRAINT uq_saldo_favor_agencia UNIQUE (id_agencia)
);

-- Movimientos de saldo a favor (historial detallado)
CREATE TABLE IF NOT EXISTS movimientos_saldo_favor (
    id SERIAL PRIMARY KEY,
    id_saldo_favor INTEGER NOT NULL REFERENCES saldos_favor(id),
    id_agencia INTEGER NOT NULL REFERENCES agencias(id),
    
    -- Tipo: 'ingreso' (por cancelación) o 'uso' (aplicado a un file)
    tipo VARCHAR(20) NOT NULL CHECK (tipo IN ('ingreso', 'uso')),
    monto NUMERIC(12,2) NOT NULL,
    
    -- Referencias
    id_cancelacion INTEGER REFERENCES cancelaciones(id),    -- Si tipo='ingreso', referencia a la cancelación
    id_file_destino INTEGER REFERENCES files(id),           -- Si tipo='uso', referencia al file donde se aplicó
    id_pago_file INTEGER REFERENCES pagos_files(id),        -- Si tipo='uso', referencia al pago_file donde se usó
    
    -- Saldo al momento de la operación
    saldo_anterior NUMERIC(12,2) NOT NULL DEFAULT 0,
    saldo_posterior NUMERIC(12,2) NOT NULL DEFAULT 0,
    
    concepto TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id)
);

-- Trigger para crear saldo_favor automáticamente cuando se crea una agencia
CREATE OR REPLACE FUNCTION crear_saldo_favor_agencia()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO saldos_favor (id_agencia, saldo_disponible, saldo_utilizado, saldo_total_generado)
    VALUES (NEW.id, 0, 0, 0)
    ON CONFLICT (id_agencia) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_crear_saldo_favor_agencia
    AFTER INSERT ON agencias
    FOR EACH ROW
    EXECUTE FUNCTION crear_saldo_favor_agencia();

-- Backfill: crear saldos_favor para agencias existentes
INSERT INTO saldos_favor (id_agencia, saldo_disponible, saldo_utilizado, saldo_total_generado)
SELECT id, 0, 0, 0 FROM agencias
ON CONFLICT (id_agencia) DO NOTHING;

-- Índices
CREATE INDEX IF NOT EXISTS idx_cancelaciones_agencia ON cancelaciones(id_agencia);
CREATE INDEX IF NOT EXISTS idx_cancelaciones_file ON cancelaciones(id_file);
CREATE INDEX IF NOT EXISTS idx_cancelaciones_tipo ON cancelaciones(tipo_cancelacion);
CREATE INDEX IF NOT EXISTS idx_no_shows_agencia ON no_shows(id_agencia);
CREATE INDEX IF NOT EXISTS idx_no_shows_file ON no_shows(id_file);
CREATE INDEX IF NOT EXISTS idx_no_shows_cancelacion ON no_shows(id_cancelacion);
CREATE INDEX IF NOT EXISTS idx_saldos_favor_agencia ON saldos_favor(id_agencia);
CREATE INDEX IF NOT EXISTS idx_mov_saldo_favor_agencia ON movimientos_saldo_favor(id_agencia);
CREATE INDEX IF NOT EXISTS idx_mov_saldo_favor_saldo ON movimientos_saldo_favor(id_saldo_favor);
CREATE INDEX IF NOT EXISTS idx_mov_saldo_favor_tipo ON movimientos_saldo_favor(tipo);
