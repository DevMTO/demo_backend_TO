# Diseño del Módulo de Contabilidad

## Resumen Ejecutivo

El sistema de contabilidad maneja los flujos financieros entre:
- **Admin/Operador**: Recibe pagos de agencias, paga a proveedores (transportes, restaurantes, guías)
- **Agencias**: Pagan por los files que generan
- **Proveedores**: Reciben pagos por los servicios prestados en file_tours

## Nuevos Roles de Usuario

Se agregan roles específicos para contabilidad:

```rust
pub enum UserRole {
    SuperAdmin,        // SYSCO - acceso total
    Admin,             // Operador - también maneja contabilidad general
    Agencias,          // Gestión de agencias
    AgenciasContador,  // Contador de agencia específica (NEW)
    Transportes,       // Gestión de transporte
    Conductores,       // Conductor individual
    Guias,             // Guía individual
    Restaurantes,      // Gestión de restaurante
}
```

> **Nota**: El rol `AgenciasContador` está vinculado obligatoriamente a una agencia (`id_entidad`).
> El Admin maneja la contabilidad general del operador.

## Arquitectura de Datos

### 1. Tabla `cuentas` (Cuentas Financieras)
```sql
CREATE TABLE cuentas (
    id SERIAL PRIMARY KEY,
    nombre VARCHAR(100) NOT NULL,
    tipo VARCHAR(20) NOT NULL,  -- 'admin', 'agencia'
    id_agencia INTEGER REFERENCES agencias(id),
    saldo_actual DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),
    
    CONSTRAINT chk_cuenta_tipo CHECK (tipo IN ('admin', 'agencia')),
    CONSTRAINT uq_cuenta_agencia UNIQUE (id_agencia) -- Una cuenta por agencia
);
```

### 2. Tabla `movimientos` (Ingresos/Egresos)
```sql
CREATE TABLE movimientos (
    id SERIAL PRIMARY KEY,
    id_cuenta INTEGER NOT NULL REFERENCES cuentas(id),
    tipo VARCHAR(10) NOT NULL,  -- 'ingreso', 'egreso'
    monto DECIMAL(15,2) NOT NULL,
    concepto VARCHAR(255) NOT NULL,
    referencia_tipo VARCHAR(50),  -- 'file', 'pago_proveedor', 'ajuste', etc.
    referencia_id INTEGER,
    fecha_movimiento TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    saldo_anterior DECIMAL(15,2) NOT NULL,
    saldo_posterior DECIMAL(15,2) NOT NULL,
    notas TEXT,
    -- Evidencia de pago (Tigris Storage)
    comprobante_url TEXT,
    comprobante_key TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),
    
    CONSTRAINT chk_movimiento_tipo CHECK (tipo IN ('ingreso', 'egreso'))
);
```

### 3. Tabla `pagos_files` (Pagos de Agencias por Files)
```sql
CREATE TABLE pagos_files (
    id SERIAL PRIMARY KEY,
    id_file INTEGER NOT NULL REFERENCES files(id),
    id_agencia INTEGER NOT NULL REFERENCES agencias(id),
    monto_total DECIMAL(15,2) NOT NULL,
    monto_pagado DECIMAL(15,2) NOT NULL DEFAULT 0.00,
    estado VARCHAR(20) NOT NULL DEFAULT 'pendiente',  -- 'pendiente', 'parcial', 'pagado'
    fecha_vencimiento DATE,
    -- Evidencia de pago
    comprobante_url TEXT,
    comprobante_key TEXT,
    verificado_por INTEGER REFERENCES users(id),
    verificado_at TIMESTAMPTZ,
    notas TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),
    
    CONSTRAINT chk_pago_file_estado CHECK (estado IN ('pendiente', 'parcial', 'pagado', 'vencido'))
);
```

### 4. Tabla `pagos_proveedores` (Pagos del Admin a Proveedores)
```sql
CREATE TABLE pagos_proveedores (
    id SERIAL PRIMARY KEY,
    tipo_proveedor VARCHAR(20) NOT NULL,  -- 'transporte', 'restaurante', 'guia'
    id_proveedor INTEGER NOT NULL,  -- ID de transporte, restaurante o id_persona de guía
    -- Relación con file_tour específico
    id_file_tour INTEGER REFERENCES file_tours(id),
    id_file_vehiculo INTEGER REFERENCES file_vehiculos(id),
    id_file_restaurante INTEGER REFERENCES file_restaurantes(id),
    id_file_guia INTEGER REFERENCES file_guias(id),
    monto DECIMAL(15,2) NOT NULL,
    estado VARCHAR(20) NOT NULL DEFAULT 'pendiente',  -- 'pendiente', 'pagado'
    fecha_pago TIMESTAMPTZ,
    -- Evidencia de pago
    comprobante_url TEXT,
    comprobante_key TEXT,
    notas TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),
    pagado_by INTEGER REFERENCES users(id),
    
    CONSTRAINT chk_tipo_proveedor CHECK (tipo_proveedor IN ('transporte', 'restaurante', 'guia'))
);
```

### 5. Tabla `tarifas_servicios` (Precios de Venta vs Costo)
```sql
CREATE TABLE tarifas_servicios (
    id SERIAL PRIMARY KEY,
    tipo_servicio VARCHAR(20) NOT NULL,  -- 'tour', 'entrada', 'restaurante', 'transporte', 'guia'
    id_servicio INTEGER NOT NULL,
    precio_venta DECIMAL(15,2) NOT NULL,  -- Precio que paga la agencia
    precio_costo DECIMAL(15,2) NOT NULL,  -- Costo que paga el admin
    margen DECIMAL(15,2) GENERATED ALWAYS AS (precio_venta - precio_costo) STORED,
    vigente_desde DATE NOT NULL DEFAULT CURRENT_DATE,
    vigente_hasta DATE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by INTEGER REFERENCES users(id),
    
    CONSTRAINT chk_tipo_servicio CHECK (tipo_servicio IN ('tour', 'entrada', 'restaurante', 'transporte', 'guia'))
);
```

## Flujos de Negocio

### Flujo 1: Agencia crea un File
1. Agencia crea file con file_tours
2. Sistema calcula `monto_total` basado en tarifas de venta
3. Se crea registro en `pagos_files` con estado 'pendiente'
4. Contador de agencia puede ver el monto a pagar

### Flujo 2: Agencia paga un File
1. Contador de agencia sube comprobante (Tigris Storage)
2. Se crea movimiento tipo 'ingreso' en cuenta Admin
3. Se crea movimiento tipo 'egreso' en cuenta Agencia
4. Admin verifica el pago y actualiza estado

### Flujo 3: Admin paga a Proveedor
1. Admin marca servicio como realizado en file_tour
2. Admin genera pago a proveedor (transporte/restaurante/guía)
3. Se crea movimiento tipo 'egreso' en cuenta Admin
4. Proveedor puede ver su estado de pago

### Flujo 4: Proveedores ven sus pagos
- Guías: Ven pagos de sus file_guias
- Transportes/Conductores: Ven pagos de sus file_vehiculos
- Restaurantes: Ven pagos de sus file_restaurantes

## Permisos por Rol

| Acción | SuperAdmin | Admin | AgenciasContador | Agencias | Transportes/Guías/Rest |
|--------|------------|-------|------------------|----------|------------------------|
| Ver cuenta admin | ✅ | ✅ | ❌ | ❌ | ❌ |
| Ver cuenta agencia | ✅ | ✅ | Solo propia | Solo propia | ❌ |
| Registrar pago file | ✅ | ✅ | Solo propia | ❌ | ❌ |
| Verificar pago file | ✅ | ✅ | ❌ | ❌ | ❌ |
| Pagar proveedores | ✅ | ✅ | ❌ | ❌ | ❌ |
| Ver mis pagos | ✅ | ✅ | ✅ | ✅ | ✅ |
| Subir comprobantes | ✅ | ✅ | ✅ | ❌ | ❌ |

## API Endpoints

### Contabilidad Admin
```
GET  /api/contabilidad/admin/dashboard      -- Resumen general
GET  /api/contabilidad/admin/movimientos    -- Lista movimientos
GET  /api/contabilidad/admin/pagos-pendientes  -- Files pendientes de cobro
GET  /api/contabilidad/admin/pagos-proveedores -- Proveedores pendientes de pago
POST /api/contabilidad/admin/verificar-pago -- Verificar pago de agencia
POST /api/contabilidad/admin/pagar-proveedor -- Registrar pago a proveedor
```

### Contabilidad Agencias
```
GET  /api/contabilidad/agencia/:id/dashboard -- Resumen de agencia
GET  /api/contabilidad/agencia/:id/files-pendientes -- Files por pagar
GET  /api/contabilidad/agencia/:id/movimientos  -- Historial
POST /api/contabilidad/agencia/:id/registrar-pago -- Subir comprobante
```

### Mis Pagos (Proveedores)
```
GET  /api/mis-pagos/guia       -- Guía ve sus pagos
GET  /api/mis-pagos/conductor  -- Conductor ve sus pagos
GET  /api/mis-pagos/transporte -- Transporte ve pagos de sus vehículos
GET  /api/mis-pagos/restaurante -- Restaurante ve sus pagos
```

## Integración con Tigris Storage

Los comprobantes de pago se almacenan en Tigris con la siguiente estructura:
```
comprobantes/
  agencias/
    {id_agencia}/
      files/
        {id_file}/
          {timestamp}_{filename}.{ext}
  proveedores/
    transportes/
      {id_transporte}/
        {timestamp}_{filename}.{ext}
    restaurantes/
      {id_restaurante}/
        {timestamp}_{filename}.{ext}
    guias/
      {id_persona}/
        {timestamp}_{filename}.{ext}
```

## Próximos Pasos de Implementación

1. ✅ Diseño del módulo
2. ⏳ Crear migraciones SQL
3. ⏳ Agregar rol `AgenciasContador` a UserRole
4. ⏳ Implementar modelos Diesel
5. ⏳ Crear DTOs y ports
6. ⏳ Implementar repositorios
7. ⏳ Crear servicios de contabilidad
8. ⏳ Implementar handlers y rutas
9. ⏳ Integrar upload de comprobantes con Tigris
10. ⏳ Frontend: Dashboard contabilidad admin
11. ⏳ Frontend: Vista contabilidad agencia
12. ⏳ Frontend: Vista "Mis Pagos" para proveedores
