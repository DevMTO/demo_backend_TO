# Guía de la Sección de Contabilidad - Sistema Tour Operador

## 1. Visión General

La sección de **Contabilidad** gestiona todos los flujos financieros del sistema de tour operador. Se divide en dos grandes módulos:

| Módulo | Descripción | Tablas principales |
|--------|-------------|-------------------|
| **Pagos** | Registro y verificación de pagos de agencias y proveedores | `pagos_files`, `pagos_proveedores` |
| **Saldo a Favor** | Cancelaciones, no-shows y créditos a favor de agencias | `cancelaciones`, `no_shows`, `saldos_favor`, `movimientos_saldo_favor` |

### Flujo general simplificado:

```
Agencia crea File → Se genera pago_file (deuda)
                   → Agencia sube comprobante de pago
                   → Admin verifica pago
                   
Si File se cancela → Se calcula saldo a favor
                   → Se acredita en saldos_favor de la agencia
                   → Puede usarse para pagar otro file
                   
Proveedores (guías, transportes, restaurantes) → Se generan pagos_proveedores
                                                → Admin registra pago al proveedor
```

### Flujo de una request HTTP típica:

```
HTTP Request → Route → Handler → Service → Repository (Port) → PostgreSQL
                                                ↑
                                    Implementación concreta
                                    (Infrastructure Layer)
```

---

## 3. Base de Datos - Tablas

### 3.1 `pagos_files` - Pagos de Agencias por File

Cada vez que una agencia crea un file (reserva de tour), se genera un registro en esta tabla representando la **deuda** de la agencia.

| Columna | Tipo | Descripción |
|---------|------|-------------|
| `id` | INT | ID autoincremental |
| `id_file` | INT | FK al file que se está pagando |
| `id_agencia` | INT | FK a la agencia que debe pagar |
| `monto_total` | NUMERIC | Monto total a pagar |
| `monto_pagado` | NUMERIC | Cuánto ha pagado la agencia hasta ahora |
| `estado` | VARCHAR(20) | `pendiente`, `parcial`, `pagado`, `verificado`, `rechazado`, `cancelado`, `no_show` |
| `fecha_vencimiento` | DATE | Fecha límite de pago (opcional) |
| `comprobante_url` | TEXT | URL del comprobante subido a Tigris (storage) |
| `comprobante_key` | TEXT | Key del archivo en Tigris |
| `verificado_por` | INT | FK al usuario admin que verificó |
| `verificado_at` | TIMESTAMPTZ | Cuándo se verificó |
| `notas` | TEXT | Notas adicionales |
| `created_by` | INT | FK al usuario que lo creó |

**Estados del flujo:**
```
pendiente → parcial → pagado → verificado
                             → rechazado (admin rechaza el comprobante)
         → cancelado (si el file se cancela)
         → no_show (si el file tuvo no-show)
```

### 3.2 `pagos_proveedores` - Pagos a Proveedores

Registra cuánto se le debe (o se le ha pagado) a cada proveedor por sus servicios en los files.

| Columna | Tipo | Descripción |
|---------|------|-------------|
| `tipo_proveedor` | VARCHAR(20) | `transporte`, `restaurante`, `guia` |
| `id_transporte` / `id_restaurante` / `id_guia` | INT | FK al proveedor específico |
| `id_file_tour` | INT | FK al file_tour donde prestó servicio |
| `id_file_vehiculo` / `id_file_restaurante` / `id_file_guia` | INT | FK a la asignación específica |
| `monto` | NUMERIC | Monto a pagar |
| `estado` | VARCHAR(20) | `pendiente`, `pagado` |
| `fecha_pago` | TIMESTAMPTZ | Cuándo se realizó el pago |
| `comprobante_url` / `comprobante_key` | TEXT | Comprobante de pago |
| `pagado_by` | INT | FK al usuario que registró el pago |

### 3.3 `cancelaciones` - Registro de Cancelaciones

Cuando un file se cancela (por la agencia o por no-show), se crea un registro aquí con el desglose financiero.

| Columna | Tipo | Descripción |
|---------|------|-------------|
| `id_file` | INT | FK al file cancelado |
| `id_agencia` | INT | FK a la agencia |
| `monto_total_file` | NUMERIC | Monto total original del file |
| `monto_pagado` | NUMERIC | Lo que ya había pagado la agencia |
| `monto_saldo_favor` | NUMERIC | Monto que se convierte en saldo a favor |
| `monto_operador` | NUMERIC | Monto que se queda el operador como penalización |
| `tipo_cancelacion` | VARCHAR(30) | `cancelacion` o `no_show` |
| `motivo` | TEXT | Razón de la cancelación |

### 3.4 `no_shows` - Detalle de No-Shows

Se crea cuando un no-show ocurre (pasajeros no se presentaron). Contiene el desglose de costos irrecuperables.

| Columna | Tipo | Descripción |
|---------|------|-------------|
| `id_cancelacion` | INT | FK a la cancelación padre |
| `id_file` | INT | FK al file |
| `monto_restaurantes` | NUMERIC | Costo de restaurantes ya consumidos |
| `monto_entradas` | NUMERIC | Costo de entradas ya compradas |
| `monto_saldo_favor` | NUMERIC | Lo que va a saldo a favor |
| `monto_operador` | NUMERIC | Lo que se queda el operador (restaurantes + entradas) |
| `hora_corte` | TIMESTAMPTZ | Hora a partir de la cual aplican penalizaciones |

### 3.5 `saldos_favor` - Saldo a Favor por Agencia

Cada agencia tiene **un único registro** aquí que actúa como su "billetera" de créditos.

| Columna | Tipo | Descripción |
|---------|------|-------------|
| `id_agencia` | INT | FK a la agencia (único) |
| `saldo_disponible` | NUMERIC | Cuánto puede usar ahora |
| `saldo_utilizado` | NUMERIC | Cuánto ha usado en total |
| `saldo_total_generado` | NUMERIC | Cuánto se ha generado en total (cancelaciones) |

**Relación:** `saldo_total_generado = saldo_disponible + saldo_utilizado`

### 3.6 `movimientos_saldo_favor` - Historial de Movimientos

Cada vez que se genera o consume saldo a favor, se registra un movimiento.

| Columna | Tipo | Descripción |
|---------|------|-------------|
| `tipo` | VARCHAR(20) | `generado` (por cancelación) o `utilizado` (pago de file) |
| `monto` | NUMERIC | Monto del movimiento |
| `id_cancelacion` | INT | FK si fue generado por cancelación |
| `id_file_destino` | INT | FK al file donde se usó el saldo |
| `saldo_anterior` | NUMERIC | Saldo antes del movimiento |
| `saldo_posterior` | NUMERIC | Saldo después del movimiento |

---

## 4. Backend - Capa por Capa

### 4.1 Schema (`schema.rs`)

Generado automáticamente por Diesel. Define la estructura de cada tabla en Rust. **No editar manualmente** — se regenera con `diesel print-schema`.

### 4.2 Models (`infrastructure/persistence/models/`)

Archivos relevantes:
- **`contabilidad_model.rs`**: `PagoFileModel`, `NewPagoFileModel`, `UpdatePagoFileModel`, `PagoProveedorModel`, `NewPagoProveedorModel`, `UpdatePagoProveedorModel`
- **`saldo_favor_model.rs`**: `CancelacionModel`, `NewCancelacionModel`, `NoShowModel`, `NewNoShowModel`, `SaldoFavorModel`, `MovimientoSaldoFavorModel`, `NewMovimientoSaldoFavorModel`

Cada tabla tiene 3 structs:
1. **Model** (con `Queryable`): Para leer de la DB
2. **NewModel** (con `Insertable`): Para insertar registros nuevos
3. **UpdateModel** (con `AsChangeset`): Para actualizar campos parcialmente

### 4.3 Ports (Traits) (`application/ports/`)

Definen las **interfaces** que deben implementar los repositorios. Esto permite cambiar la base de datos sin tocar la lógica de negocio.

**`contabilidad_repository.rs`** define:
```rust
trait PagoFileRepositoryPort {
    async fn find_all(...) -> Result<Vec<PagoFileModel>>;
    async fn find_by_id(id) -> Result<Option<PagoFileModel>>;
    async fn find_by_file(id_file) -> Result<Option<PagoFileModel>>;
    async fn create(...) -> Result<PagoFileModel>;
    async fn update(id, data) -> Result<PagoFileModel>;
    // ... más métodos
}

trait PagoProveedorRepositoryPort { /* similar */ }
```

**`saldo_favor_repository.rs`** define:
```rust
trait SaldoFavorRepositoryPort {
    // Saldos
    async fn find_saldo_by_agencia(id_agencia) -> ...;
    async fn create_or_update_saldo(...) -> ...;
    
    // Cancelaciones
    async fn create_cancelacion(...) -> ...;
    async fn find_cancelaciones(...) -> ...;
    
    // No-shows
    async fn create_no_show(...) -> ...;
    
    // Movimientos
    async fn create_movimiento(...) -> ...;
    
    // Cálculos de costos del file
    async fn calculate_file_restaurant_costs(id_file) -> ...;
    async fn calculate_file_entrance_costs(id_file) -> ...;
}
```

### 4.4 Repository Implementations (`infrastructure/persistence/repositories/`)

Implementan los ports usando **Diesel** para ejecutar queries contra PostgreSQL.

- **`contabilidad_repository.rs`**: `PostgresPagoFileRepository`, `PostgresPagoProveedorRepository`
- **`saldo_favor_repository.rs`**: `PostgresSaldoFavorRepository`

Cada repositorio recibe un `DatabasePool` y obtiene una conexión con `self.pool.get_connection().await?`.

### 4.5 Services (`application/services/`)

Contienen la **lógica de negocio**. Usan los repositorios (via ports) para operar sobre la DB.

#### `contabilidad_service.rs` - Operaciones principales:

| Método | Descripción |
|--------|-------------|
| `get_agencia_dashboard` | Retorna resumen financiero de una agencia |
| `list_pagos_files` | Lista paginada de pagos de files con filtros |
| `registrar_pago_file` | La agencia sube comprobante (base64 → Tigris) y registra monto |
| `verificar_pago_file` | Admin aprueba o rechaza un pago |
| `list_pagos_proveedores` | Lista paginada de pagos a proveedores |
| `create_pago_proveedor` | Crea registro de pago pendiente a proveedor |
| `marcar_pago_proveedor_pagado` | Admin marca como pagado con comprobante |

#### `saldo_favor_service.rs` - Operaciones principales:

| Método | Qué hace |
|--------|----------|
| `cancelar_file` | Cancela un file. Si la agencia ya pagó, genera saldo a favor por el monto pagado. Actualiza el pago_file a estado `cancelado`. |
| `registrar_no_show` | Registra no-show. Calcula costos de restaurantes y entradas (irrecuperables). Solo esos van al operador, el resto a saldo a favor. |
| `usar_saldo` | Permite a la agencia usar su saldo a favor para pagar otro file. Descuenta del `saldo_disponible` y suma a `saldo_utilizado`. |
| `get_dashboard` | Dashboard con saldo actual, cancelaciones recientes y movimientos |
| `list_cancelaciones` | Lista cancelaciones con filtros y paginación |
| `list_no_shows` | Lista no-shows |
| `list_movimientos` | Historial de movimientos del saldo |

### 4.6 DTOs (`application/dtos/`)

Los DTOs (Data Transfer Objects) son los structs que se envían/reciben como JSON en los endpoints. Usan `#[ts(export)]` para generar automáticamente los tipos TypeScript en el frontend.

- **`contabilidad_dto.rs`**: `PagoFileResponse`, `RegistrarPagoFileRequest`, `VerificarPagoFileRequest`, `PagoProveedorResponse`, `CreatePagoProveedorRequest`, etc.
- **`saldo_favor_dto.rs`**: `CancelacionResponse`, `CancelarFileRequest`, `RegistrarNoShowRequest`, `NoShowResponse`, `SaldoFavorResponse`, etc.

**Para generar los tipos TS**: Ejecutar `cargo test export_bindings` desde el directorio del backend. Los archivos se generan en `frontend/src/domain/contracts/`.

### 4.7 Handlers (`presentation/handlers/`)

Los handlers son las funciones que **Axum** ejecuta cuando llega un request HTTP. Cada handler:
1. Extrae datos del request (body, path params, query params)
2. Valida los datos
3. Llama al servicio correspondiente
4. Retorna la respuesta JSON

Organizados en:
- **`contabilidad/get.rs`**: Handlers GET (listar, dashboard)
- **`contabilidad/post.rs`**: Handlers POST (registrar, verificar, crear, pagar)
- **`contabilidad/query_params.rs`**: Structs de query parameters (`?page=1&estado=pendiente`)
- **`saldo_favor/get.rs`**: Handlers GET para saldo a favor
- **`saldo_favor/post.rs`**: Handlers POST (cancelar, no-show, usar saldo)

### 4.8 Routes (`presentation/routes/`)

Mapean **URLs a handlers**. Definen qué método HTTP y qué path corresponde a cada handler.

#### Contabilidad Routes (`/api/v1/contabilidad/`):

| Método | Ruta | Handler | Descripción |
|--------|------|---------|-------------|
| GET | `/dashboard/agencia/{id}` | `get_agencia_dashboard` | Dashboard financiero de agencia |
| GET | `/pagos-files` | `list_pagos_files` | Listar pagos de files |
| POST | `/pagos-files/registrar` | `registrar_pago_file` | Registrar pago con comprobante |
| POST | `/pagos-files/verificar` | `verificar_pago_file` | Admin verifica pago |
| GET | `/pagos-proveedores` | `list_pagos_proveedores` | Listar pagos a proveedores |
| POST | `/pagos-proveedores` | `create_pago_proveedor` | Crear pago a proveedor |
| POST | `/pagos-proveedores/{id}/pagar` | `marcar_pago_proveedor_pagado` | Marcar proveedor como pagado |

#### Saldo a Favor Routes (`/api/v1/saldo-favor/`):

| Método | Ruta | Handler | Descripción |
|--------|------|---------|-------------|
| GET | `/` | `list_saldos` | Listar todos los saldos |
| GET | `/agencia/{id}` | `get_saldo_agencia` | Saldo de una agencia |
| GET | `/dashboard/{id}` | `get_dashboard` | Dashboard saldo a favor |
| POST | `/cancelar` | `cancelar_file` | Cancelar file |
| GET | `/cancelaciones` | `list_cancelaciones` | Listar cancelaciones |
| POST | `/no-show` | `registrar_no_show` | Registrar no-show |
| GET | `/no-shows` | `list_no_shows` | Listar no-shows |
| POST | `/usar` | `usar_saldo` | Usar saldo para pagar file |
| GET | `/movimientos` | `list_movimientos` | Historial movimientos |

---

## 5. Frontend - Capa por Capa

### 5.1 Domain Contracts (`domain/contracts/`)

Son los **tipos TypeScript** generados automáticamente por `ts-rs` desde los DTOs del backend. Cada archivo corresponde a un DTO:

```
PagoFileResponse.ts          → Respuesta de pago de file
PagoProveedorResponse.ts     → Respuesta de pago a proveedor
RegistrarPagoFileRequest.ts  → Request para registrar pago
CancelacionResponse.ts       → Respuesta de cancelación
NoShowResponse.ts             → Respuesta de no-show
SaldoFavorResponse.ts         → Respuesta de saldo a favor
MovimientoSaldoFavorResponse.ts → Movimiento de saldo
...
```

**⚠️ No editar estos archivos manualmente.** Se regeneran con `cargo test export_bindings`.

### 5.2 Adapters (`infrastructure/api/`)

Los adapters son la capa que **comunica el frontend con el backend** via HTTP. Cada método del adapter hace una llamada fetch/axios a un endpoint del backend.

#### `contabilidad-adapter.ts`:

```typescript
contabilidadAdapter.getAgenciaDashboard(idAgencia)     // GET /contabilidad/dashboard/agencia/{id}
contabilidadAdapter.listPagosFiles(filters)             // GET /contabilidad/pagos-files
contabilidadAdapter.registrarPagoFile(request)          // POST /contabilidad/pagos-files/registrar
contabilidadAdapter.verificarPagoFile(request)          // POST /contabilidad/pagos-files/verificar
contabilidadAdapter.listPagosProveedores(filters)       // GET /contabilidad/pagos-proveedores
contabilidadAdapter.createPagoProveedor(request)        // POST /contabilidad/pagos-proveedores
contabilidadAdapter.marcarPagoProveedorPagado(id, req)  // POST /contabilidad/pagos-proveedores/{id}/pagar
```

#### `saldo-favor-adapter.ts`:

```typescript
saldoFavorAdapter.getDashboard(idAgencia)     // GET /saldo-favor/dashboard/{id}
saldoFavorAdapter.listSaldos()                 // GET /saldo-favor/
saldoFavorAdapter.getSaldoAgencia(idAgencia)   // GET /saldo-favor/agencia/{id}
saldoFavorAdapter.cancelarFile(request)        // POST /saldo-favor/cancelar
saldoFavorAdapter.registrarNoShow(request)     // POST /saldo-favor/no-show
saldoFavorAdapter.usarSaldo(request)           // POST /saldo-favor/usar
saldoFavorAdapter.listCancelaciones(filters)   // GET /saldo-favor/cancelaciones
saldoFavorAdapter.listNoShows(filters)         // GET /saldo-favor/no-shows
saldoFavorAdapter.listMovimientos(filters)     // GET /saldo-favor/movimientos
```

### 5.3 Hooks (`presentation/hooks/`)

Los hooks usan **React Query (TanStack Query)** para manejar el estado de las llamadas al server. Cada hook encapsula:
- **Query keys** para cacheo e invalidación
- **Queries** (useQuery) para leer datos
- **Mutations** (useMutation) para crear/actualizar datos

#### `use-contabilidad-files.ts`:
```typescript
useContabilidadFiles(filters)     // Carga paginada de pagos files
useAllContabilidadFiles()          // Todos los pagos files
useContabilidadFilesByEstado(est)  // Filtrados por estado
```

#### `use-saldo-favor.ts`:
```typescript
useSaldoFavorDashboard(idAgencia)  // Dashboard del saldo a favor
useSaldosFavor()                    // Listar todos los saldos
useSaldoAgencia(idAgencia)          // Saldo de una agencia
useCancelaciones(filters)           // Listar cancelaciones
useNoShows(filters)                 // Listar no-shows
useMovimientosSaldo(filters)        // Historial movimientos
useCancelarFile()                   // Mutación: cancelar file
useRegistrarNoShow()                // Mutación: registrar no-show
useUsarSaldo()                      // Mutación: usar saldo para pagar
```

### 5.4 Pages (`app/(protected)/contabilidad/`)

| Página | Ruta | Quién la ve | Qué muestra |
|--------|------|-------------|-------------|
| Hub principal | `/contabilidad` | Admin | Tarjetas resumen con conteos de pagos-files y pagos-proveedores, links a cada sección |
| Pagos Files | `/contabilidad/pagos-files` | Admin | Tabla con todos los pagos. Modales para registrar pago (subir comprobante), verificar/rechazar, ver detalle |
| Pagos Proveedores | `/contabilidad/pagos-proveedores` | Admin | Tabla de pagos a proveedores. Modales para crear pago y marcar como pagado |
| Dashboard Agencia | `/contabilidad/agencia` | Agencia | Resumen financiero, files pendientes de pago, historial de pagos |
| Liquidación | `/contabilidad/liquidacion` | Agencia (contador) | Seleccionar files, generar PDF/Excel de liquidación |
| No-Show | `/contabilidad/no-show` | Agencia | Lista de files con estado no_show |
| Saldo a Favor | `/contabilidad/saldo-a-favor/agencia` | Agencia | Saldo disponible, lista de cancelaciones y movimientos |
| Mis Pagos | `/mis-pagos` | Proveedores | Guías, conductores y restaurantes ven sus pagos |

---

## 6. Flujos de Negocio Detallados

### 6.1 Flujo de Pago de File (Agencia → Operador)

```
1. Agencia crea un File con tours
2. Se genera automáticamente un registro en pagos_files con:
   - monto_total = monto del file
   - monto_pagado = 0
   - estado = 'pendiente'

3. Agencia entra a /contabilidad/agencia y ve sus files pendientes

4. Agencia sube comprobante de pago:
   - Sube imagen como base64
   - Backend lo sube a Tigris (storage S3-compatible)
   - Actualiza monto_pagado y comprobante_url
   - Estado → 'pagado' (si pagó completo) o 'parcial'

5. Admin entra a /contabilidad/pagos-files
   - Ve los pagos con estado 'pagado' pendientes de verificación
   - Revisa el comprobante
   - Aprueba → estado pasa a 'verificado'
   - Rechaza → estado pasa a 'rechazado' (agencia debe volver a pagar)
```

### 6.2 Flujo de Cancelación

```
1. Agencia cancela un file desde /misreservas (botón cancelar)
2. Frontend llama a saldoFavorAdapter.cancelarFile()
3. Backend (saldo_favor_service.cancelar_file):
   a. Cambia file.status a 'cancelado'
   b. Busca pagos_files del file
   c. Crea registro en cancelaciones con:
      - monto_saldo_favor = monto_pagado (lo que ya pagó)
      - monto_operador = 0 (cancelación normal no tiene penalización)
   d. Si monto_pagado > 0:
      - Busca o crea saldos_favor de la agencia
      - Suma monto al saldo_disponible y saldo_total_generado
      - Crea movimiento de tipo 'generado'
   e. Actualiza estado del pagos_files a 'cancelado'
4. Frontend invalida queries para refrescar datos
```

### 6.3 Flujo de No-Show

```
1. Admin registra no-show desde el panel de tours (NoShowButton)
2. Frontend llama a useRegistrarNoShow con el id_file
3. Backend (saldo_favor_service.registrar_no_show):
   a. Calcula costos irrecuperables:
      - monto_restaurantes = suma de precios de restaurantes asignados al file
      - monto_entradas = suma de (precio × cantidad) de entradas asignadas
   b. Monto operador = monto_restaurantes + monto_entradas
   c. Monto saldo favor = monto_pagado - monto_operador (si es positivo)
   d. Crea cancelacion con tipo_cancelacion = 'no_show'
   e. Crea no_show con el desglose
   f. Acredita saldo a favor (si corresponde)
   g. Actualiza estados
```

### 6.4 Flujo de Uso de Saldo a Favor

```
1. Agencia tiene saldo disponible (de cancelaciones previas)
2. Quiere pagar un nuevo file usando ese saldo
3. Llama a saldoFavorAdapter.usarSaldo({
     id_agencia, id_file_destino, id_pago_file, monto, concepto
   })
4. Backend:
   a. Verifica que la agencia tiene suficiente saldo
   b. Descuenta del saldo_disponible
   c. Suma al saldo_utilizado
   d. Crea movimiento de tipo 'utilizado'
   e. Actualiza monto_pagado del pago_file destino
```

---

## 7. Cómo Agregar Funcionalidad Nueva

### 7.1 Agregar un nuevo endpoint al backend

1. **DTO**: Crear Request/Response structs en `application/dtos/contabilidad_dto.rs` o `saldo_favor_dto.rs` con `#[ts(export)]`
2. **Port**: Agregar método al trait en `application/ports/`
3. **Repository**: Implementar el método en `infrastructure/persistence/repositories/`
4. **Service**: Agregar lógica de negocio en `application/services/`
5. **Handler**: Crear handler en `presentation/handlers/contabilidad/` o `saldo_favor/`
6. **Route**: Registrar en `presentation/routes/contabilidad.rs` o `saldo_favor.rs`
7. **Generar tipos TS**: Ejecutar `cargo test export_bindings`

### 7.2 Agregar funcionalidad al frontend

1. **Contract**: Se genera automáticamente con `cargo test export_bindings`
2. **Adapter**: Agregar método en `infrastructure/api/contabilidad-adapter.ts` o `saldo-favor-adapter.ts`
3. **Hook**: Crear/actualizar hook en `presentation/hooks/`
4. **Page**: Crear/actualizar componente de página en `app/(protected)/contabilidad/`

### 7.3 Ejemplo práctico: Agregar endpoint de "Anular pago de proveedor"

```
Backend:
1. DTO → AnularPagoProveedorRequest { id_pago_proveedor: i32, motivo: String }
2. Port → async fn anular_pago_proveedor(id: i32) -> Result<PagoProveedorModel>
3. Repo → DELETE o UPDATE estado='anulado' en pagos_proveedores
4. Service → anular_pago_proveedor(id, motivo, user_id)
5. Handler → pub async fn anular_pago_proveedor(State, Auth, Path, Json)
6. Route → .route("/pagos-proveedores/{id}/anular", post(...))

Frontend:
1. Contract → cargo test export_bindings
2. Adapter → contabilidadAdapter.anularPagoProveedor(id, request)
3. Hook → useAnularPagoProveedor() → useMutation(...)
4. Page → Botón "Anular" en la tabla de pagos-proveedores
```

---

## 8. Container / Dependency Injection

Todos los repositorios y servicios se inicializan en el **Container** (`infrastructure/container/`):

```rust
// container/repositories.rs
pub pago_file_repository: Arc<dyn PagoFileRepositoryPort>,
pub pago_proveedor_repository: Arc<dyn PagoProveedorRepositoryPort>,
pub saldo_favor_repository: Arc<dyn SaldoFavorRepositoryPort>,

// container/services.rs  
pub contabilidad_service: Arc<ContabilidadService>,
pub saldo_favor_service: Arc<SaldoFavorService>,
```

Los handlers acceden a todo via `state.container.contabilidad_service` o `state.container.saldo_favor_repository`.

---

## 9. Notas Importantes para Futuras Implementaciones

1. **Las tablas `pagos`, `cuentas`, `movimientos` y `tarifas_servicios` fueron eliminadas** en la migración `2026-02-10-000001`. No existen en la DB actual.

2. **Los comprobantes se suben a Tigris** (S3-compatible). El flujo es: base64 en el request → decodificar → subir a Tigris → guardar URL en la DB.

3. **El saldo a favor es por agencia**, no por file ni por usuario. Cada agencia tiene un único registro en `saldos_favor`.

4. **Los movimientos de saldo son inmutables**. Una vez creados, no se editan ni eliminan. Son un log financiero.

5. **El cálculo de costos de no-show** se hace con SQL raw queries porque involucra JOINs complejos entre `file_tours → file_restaurantes → restaurantes` y `file_tours → file_entradas → entradas`.

6. **Los tipos TypeScript se regeneran** con `cargo test export_bindings` — siempre ejecutar después de modificar DTOs en el backend.

7. **React Query** invalida automáticamente las queries relacionadas cuando una mutación tiene éxito (configurado en los hooks con `queryClient.invalidateQueries`).
