-- ========================================================================
-- MÓDULO DE CONTABILIDAD
-- Migración: create_contabilidad_module
-- ========================================================================

-- ========================================================================
-- 1. TABLA: cuentas (Cuentas financieras del sistema)
-- ========================================================================
CREATE TABLE cuentas (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(100) NOT NULL,
    tipo VARCHAR(20) NOT NULL,  -- 'admin', 'agencia'
    id_agencia INTEGER REFERENCES agencias(id) ON DELETE SET NULL,
    saldo_actual DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT chk_cuenta_tipo CHECK (tipo IN ('admin', 'agencia')),
    CONSTRAINT uq_cuenta_agencia UNIQUE (id_agencia) -- Una sola cuenta por agencia
);

-- Índices
CREATE INDEX idx_cuentas_tipo ON cuentas(tipo);
CREATE INDEX idx_cuentas_agencia ON cuentas(id_agencia) WHERE id_agencia IS NOT NULL;

-- Trigger para actualizar updated_at
CREATE OR REPLACE FUNCTION update_cuentas_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_cuentas_updated_at
    BEFORE UPDATE ON cuentas
    FOR EACH ROW
    EXECUTE FUNCTION update_cuentas_updated_at();

-- ========================================================================
-- 2. TABLA: movimientos (Registro de ingresos y egresos)
-- ========================================================================
CREATE TABLE movimientos (
    id SERIAL PRIMARY KEY,
    id_cuenta INTEGER NOT NULL REFERENCES cuentas(id) ON DELETE RESTRICT,
    tipo VARCHAR(10) NOT NULL,  -- 'ingreso', 'egreso'
    monto DECIMAL(15,2) NOT NULL,
    concepto VARCHAR(255) NOT NULL,
    referencia_tipo VARCHAR(50),  -- 'file', 'pago_proveedor', 'ajuste', 'inicial'
    referencia_id INTEGER,
    fecha_movimiento TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    saldo_anterior DECIMAL(15,2) NOT NULL,
    saldo_posterior DECIMAL(15,2) NOT NULL,
    notas TEXT,
    -- Evidencia de pago (Tigris Storage)
    comprobante_url TEXT,
    comprobante_key TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT chk_movimiento_tipo CHECK (tipo IN ('ingreso', 'egreso')),
    CONSTRAINT chk_monto_positivo CHECK (monto > 0)
);

-- Índices
CREATE INDEX idx_movimientos_cuenta ON movimientos(id_cuenta);
CREATE INDEX idx_movimientos_tipo ON movimientos(tipo);
CREATE INDEX idx_movimientos_fecha ON movimientos(fecha_movimiento DESC);
CREATE INDEX idx_movimientos_referencia ON movimientos(referencia_tipo, referencia_id) 
    WHERE referencia_tipo IS NOT NULL AND referencia_id IS NOT NULL;

-- ========================================================================
-- 3. TABLA: pagos_files (Pagos de agencias por files)
-- ========================================================================
CREATE TABLE pagos_files (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    id_agencia INTEGER NOT NULL REFERENCES agencias(id) ON DELETE RESTRICT,
    monto_total DECIMAL(15,2) NOT NULL,
    monto_pagado DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    estado VARCHAR(20) NOT NULL DEFAULT 'pendiente',  -- 'pendiente', 'parcial', 'pagado', 'vencido'
    fecha_vencimiento DATE,
    -- Evidencia de pago
    comprobante_url TEXT,
    comprobante_key TEXT,
    -- Verificación por admin
    verificado_por INTEGER REFERENCES users(id) ON DELETE SET NULL,
    verificado_at TIMESTAMPTZ,
    notas TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT chk_pago_file_estado CHECK (estado IN ('pendiente', 'parcial', 'pagado', 'vencido')),
    CONSTRAINT chk_monto_pagado CHECK (monto_pagado >= 0 AND monto_pagado <= monto_total)
);

-- Índices
CREATE INDEX idx_pagos_files_file ON pagos_files(id_file);
CREATE INDEX idx_pagos_files_agencia ON pagos_files(id_agencia);
CREATE INDEX idx_pagos_files_estado ON pagos_files(estado);
CREATE INDEX idx_pagos_files_vencimiento ON pagos_files(fecha_vencimiento) WHERE estado IN ('pendiente', 'parcial');

-- Trigger para updated_at
CREATE TRIGGER trigger_pagos_files_updated_at
    BEFORE UPDATE ON pagos_files
    FOR EACH ROW
    EXECUTE FUNCTION update_cuentas_updated_at();

-- ========================================================================
-- 4. TABLA: pagos_proveedores (Pagos del admin a proveedores)
-- ========================================================================
CREATE TABLE pagos_proveedores (
    id SERIAL PRIMARY KEY,
    tipo_proveedor VARCHAR(20) NOT NULL,  -- 'transporte', 'restaurante', 'guia'
    -- ID de la entidad proveedora
    id_transporte INTEGER REFERENCES transportes(id) ON DELETE SET NULL,
    id_restaurante INTEGER REFERENCES restaurantes(id) ON DELETE SET NULL,
    id_guia INTEGER REFERENCES guias(id) ON DELETE SET NULL,  -- guia.id_persona vincula a personas
    -- Relación con el servicio específico del file
    id_file_tour INTEGER REFERENCES file_tours(id) ON DELETE CASCADE,
    id_file_vehiculo INTEGER REFERENCES file_vehiculos(id) ON DELETE SET NULL,
    id_file_restaurante INTEGER REFERENCES file_restaurantes(id) ON DELETE SET NULL,
    id_file_guia INTEGER REFERENCES file_guias(id) ON DELETE SET NULL,
    -- Datos del pago
    monto DECIMAL(15,2) NOT NULL,
    estado VARCHAR(20) NOT NULL DEFAULT 'pendiente',  -- 'pendiente', 'pagado'
    fecha_pago TIMESTAMPTZ,
    -- Evidencia de pago
    comprobante_url TEXT,
    comprobante_key TEXT,
    notas TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    pagado_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT chk_tipo_proveedor CHECK (tipo_proveedor IN ('transporte', 'restaurante', 'guia')),
    CONSTRAINT chk_pago_proveedor_estado CHECK (estado IN ('pendiente', 'pagado')),
    -- Asegurar que se vincule con la entidad correcta según tipo
    CONSTRAINT chk_proveedor_entidad CHECK (
        (tipo_proveedor = 'transporte' AND id_transporte IS NOT NULL) OR
        (tipo_proveedor = 'restaurante' AND id_restaurante IS NOT NULL) OR
        (tipo_proveedor = 'guia' AND id_guia IS NOT NULL)
    )
);

-- Índices
CREATE INDEX idx_pagos_proveedores_tipo ON pagos_proveedores(tipo_proveedor);
CREATE INDEX idx_pagos_proveedores_transporte ON pagos_proveedores(id_transporte) WHERE id_transporte IS NOT NULL;
CREATE INDEX idx_pagos_proveedores_restaurante ON pagos_proveedores(id_restaurante) WHERE id_restaurante IS NOT NULL;
CREATE INDEX idx_pagos_proveedores_guia ON pagos_proveedores(id_guia) WHERE id_guia IS NOT NULL;
CREATE INDEX idx_pagos_proveedores_file_tour ON pagos_proveedores(id_file_tour);
CREATE INDEX idx_pagos_proveedores_estado ON pagos_proveedores(estado);

-- Trigger para updated_at
CREATE TRIGGER trigger_pagos_proveedores_updated_at
    BEFORE UPDATE ON pagos_proveedores
    FOR EACH ROW
    EXECUTE FUNCTION update_cuentas_updated_at();

-- ========================================================================
-- 5. TABLA: tarifas_servicios (Precios de venta vs costo para márgenes)
-- ========================================================================
CREATE TABLE tarifas_servicios (
    id SERIAL PRIMARY KEY,
    tipo_servicio VARCHAR(20) NOT NULL,  -- 'tour', 'entrada', 'restaurante', 'transporte', 'guia'
    id_servicio INTEGER NOT NULL,  -- ID de la entidad según tipo_servicio
    precio_venta DECIMAL(15,2) NOT NULL,  -- Precio que paga la agencia
    precio_costo DECIMAL(15,2) NOT NULL,  -- Costo que paga el admin al proveedor
    margen DECIMAL(15,2) GENERATED ALWAYS AS (precio_venta - precio_costo) STORED,
    vigente_desde DATE NOT NULL DEFAULT CURRENT_DATE,
    vigente_hasta DATE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT chk_tipo_servicio CHECK (tipo_servicio IN ('tour', 'entrada', 'restaurante', 'transporte', 'guia')),
    CONSTRAINT chk_precios_positivos CHECK (precio_venta >= 0 AND precio_costo >= 0)
);

-- Índices
CREATE INDEX idx_tarifas_tipo ON tarifas_servicios(tipo_servicio);
CREATE INDEX idx_tarifas_servicio ON tarifas_servicios(tipo_servicio, id_servicio);
CREATE INDEX idx_tarifas_vigencia ON tarifas_servicios(vigente_desde, vigente_hasta) WHERE is_active = TRUE;

-- Trigger para updated_at
CREATE TRIGGER trigger_tarifas_servicios_updated_at
    BEFORE UPDATE ON tarifas_servicios
    FOR EACH ROW
    EXECUTE FUNCTION update_cuentas_updated_at();

-- ========================================================================
-- 6. DATOS INICIALES: Crear cuenta admin del operador
-- ========================================================================
INSERT INTO cuentas (nombre, tipo, saldo_actual)
VALUES ('Cuenta Principal - Operador', 'admin', 0.00);

-- ========================================================================
-- 7. FUNCIÓN: Crear cuenta automáticamente cuando se crea una agencia
-- ========================================================================
CREATE OR REPLACE FUNCTION crear_cuenta_agencia()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO cuentas (nombre, tipo, id_agencia, saldo_actual, created_by)
    VALUES (
        'Cuenta - ' || NEW.nombre,
        'agencia',
        NEW.id,
        0.00,
        NEW.created_by
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_crear_cuenta_agencia
    AFTER INSERT ON agencias
    FOR EACH ROW
    EXECUTE FUNCTION crear_cuenta_agencia();

-- ========================================================================
-- 8. CREAR CUENTAS PARA AGENCIAS EXISTENTES (si las hay)
-- ========================================================================
INSERT INTO cuentas (nombre, tipo, id_agencia, saldo_actual, created_by)
SELECT 
    'Cuenta - ' || a.nombre,
    'agencia',
    a.id,
    0.00,
    a.created_by
FROM agencias a
WHERE NOT EXISTS (
    SELECT 1 FROM cuentas c WHERE c.id_agencia = a.id
);

-- ========================================================================
-- 9. FUNCIÓN: Calcular monto de file cuando se crean/actualizan file_tours
-- ========================================================================
-- Esta función se llamará desde el backend para mantener consistencia

COMMENT ON TABLE cuentas IS 'Cuentas financieras del sistema (admin y agencias)';
COMMENT ON TABLE movimientos IS 'Registro de ingresos y egresos de cada cuenta';
COMMENT ON TABLE pagos_files IS 'Control de pagos de agencias por sus files';
COMMENT ON TABLE pagos_proveedores IS 'Control de pagos del admin a proveedores de servicios';
COMMENT ON TABLE tarifas_servicios IS 'Precios de venta vs costo para calcular márgenes';
