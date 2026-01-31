# Implementación de Geolocalización con PostGIS

## Resumen

Este documento describe la implementación de geolocalización para tours y file_tours utilizando PostGIS en PostgreSQL y Leaflet en el frontend.

---

## 1. Estructura de Datos Propuesta

### 1.1 Campos a agregar/modificar

#### Tabla `tours`:
```sql
-- Campos actuales (VARCHAR)
lugar_inicio VARCHAR(200) NOT NULL  -- Mantener para nombre descriptivo
lugar_fin VARCHAR(200) NOT NULL     -- Mantener para nombre descriptivo

-- Nuevos campos geográficos (NULLABLE - opcionales)
coordenadas_inicio GEOGRAPHY(Point, 4326)  -- Lat/Lng del punto de inicio
coordenadas_fin GEOGRAPHY(Point, 4326)      -- Lat/Lng del punto de fin
ruta_tour GEOGRAPHY(LineString, 4326)       -- Ruta completa opcional
```

#### Tabla `file_tours`:
```sql
-- Campo actual (VARCHAR)
lugar_recojo VARCHAR(200)  -- Mantener para nombre descriptivo

-- Nuevos campos geográficos (NULLABLE - opcionales)
coordenadas_recojo GEOGRAPHY(Point, 4326)  -- Lat/Lng del punto de recojo
```

---

## 2. ¿Por qué GEOGRAPHY en lugar de GEOMETRY?

| Característica | GEOMETRY | GEOGRAPHY |
|----------------|----------|-----------|
| Unidad | Unidades de coordenadas (planas) | Metros (esférica) |
| Cálculo de distancias | Requiere proyección | Automático en esfera |
| Precisión global | Distorsión en grandes distancias | Preciso globalmente |
| Performance | Más rápido | Ligeramente más lento |

**Decisión**: GEOGRAPHY es mejor para turismo porque:
- Los tours pueden ser en cualquier parte del mundo
- Las distancias se calculan correctamente en metros/km
- No requiere gestionar proyecciones

---

## 3. Sistema de Coordenadas: SRID 4326

- **SRID 4326** = WGS84 (World Geodetic System 1984)
- Es el estándar de GPS y mapas web
- Leaflet usa este sistema por defecto
- Formato: `POINT(longitud latitud)` (ojo: lon ANTES de lat)

---

## 4. Migraciones Diesel

### 4.1 Migración: Habilitar PostGIS y agregar columnas

```sql
-- UP
CREATE EXTENSION IF NOT EXISTS postgis;

-- Tours: Agregar coordenadas opcionales
ALTER TABLE tours ADD COLUMN coordenadas_inicio GEOGRAPHY(Point, 4326);
ALTER TABLE tours ADD COLUMN coordenadas_fin GEOGRAPHY(Point, 4326);
ALTER TABLE tours ADD COLUMN ruta_tour GEOGRAPHY(LineString, 4326);

-- File Tours: Agregar coordenadas de recojo
ALTER TABLE file_tours ADD COLUMN coordenadas_recojo GEOGRAPHY(Point, 4326);

-- Hacer lugar_inicio y lugar_fin nullable (opcional ahora)
ALTER TABLE tours ALTER COLUMN lugar_inicio DROP NOT NULL;
ALTER TABLE tours ALTER COLUMN lugar_fin DROP NOT NULL;

-- Índices espaciales para búsquedas eficientes
CREATE INDEX idx_tours_coordenadas_inicio ON tours USING GIST(coordenadas_inicio);
CREATE INDEX idx_tours_coordenadas_fin ON tours USING GIST(coordenadas_fin);
CREATE INDEX idx_file_tours_coordenadas_recojo ON file_tours USING GIST(coordenadas_recojo);

-- Comentarios
COMMENT ON COLUMN tours.coordenadas_inicio IS 'Coordenadas geográficas del punto de inicio (lon, lat)';
COMMENT ON COLUMN tours.coordenadas_fin IS 'Coordenadas geográficas del punto de fin (lon, lat)';
COMMENT ON COLUMN tours.ruta_tour IS 'Ruta completa del tour como línea geográfica';
COMMENT ON COLUMN file_tours.coordenadas_recojo IS 'Coordenadas geográficas del punto de recojo';
```

---

## 5. Integración con Diesel (Rust)

### 5.1 Dependencias en Cargo.toml
```toml
[dependencies]
postgis = "0.9"
diesel-geography = "0.2"  # O manejar como JSONB
```

### 5.2 Alternativa: Almacenar como JSONB

Si PostGIS complica la integración con Diesel, podemos usar JSONB:

```sql
-- Almacenar coordenadas como JSONB
ALTER TABLE tours ADD COLUMN geo_inicio JSONB;
ALTER TABLE tours ADD COLUMN geo_fin JSONB;
ALTER TABLE file_tours ADD COLUMN geo_recojo JSONB;

-- Formato del JSONB:
-- { "lat": -13.5250, "lng": -71.9653, "zoom": 15 }
```

**Ventajas de JSONB**:
- Diesel soporta JSONB nativamente
- Más simple de implementar
- Serde se encarga de serializar/deserializar

**Desventajas**:
- No podemos usar funciones espaciales de PostGIS (ST_Distance, ST_Within, etc.)
- Si no necesitas búsquedas espaciales, JSONB es suficiente

---

## 6. DTOs para Frontend

### 6.1 Estructura de Coordenadas
```rust
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GeoPoint {
    pub lat: f64,
    pub lng: f64,
    pub zoom: Option<i32>,  // Nivel de zoom sugerido para el mapa
    pub label: Option<String>,  // Etiqueta para mostrar en el marcador
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct GeoRoute {
    pub points: Vec<GeoPoint>,
    pub color: Option<String>,  // Color de la línea
    pub weight: Option<i32>,    // Grosor de la línea
}
```

### 6.2 Tour con Geolocalización
```rust
pub struct TourResponse {
    pub id: i32,
    pub nombre: String,
    pub lugar_inicio: Option<String>,  // Nombre descriptivo
    pub lugar_fin: Option<String>,
    pub geo_inicio: Option<GeoPoint>,  // Coordenadas
    pub geo_fin: Option<GeoPoint>,
    pub ruta: Option<GeoRoute>,        // Ruta en el mapa
    // ... otros campos
}
```

---

## 7. Endpoints API para Geolocalización

### 7.1 Actualizar coordenadas de un tour
```
PATCH /api/v1/tours/{id}/geolocation
Body: {
    "geo_inicio": { "lat": -13.5250, "lng": -71.9653, "zoom": 15 },
    "geo_fin": { "lat": -13.1631, "lng": -72.5450, "zoom": 14 },
    "ruta": [
        { "lat": -13.5250, "lng": -71.9653 },
        { "lat": -13.3, "lng": -72.1 },
        { "lat": -13.1631, "lng": -72.5450 }
    ]
}
```

### 7.2 Buscar tours cercanos (con PostGIS)
```
GET /api/v1/tours/nearby?lat=-13.5&lng=-71.9&radius=50000
Response: Tours dentro de 50km del punto dado
```

### 7.3 Actualizar punto de recojo de file_tour
```
PATCH /api/v1/file-tours/{id}/pickup-location
Body: {
    "lugar_recojo": "Hotel Libertador",
    "geo_recojo": { "lat": -13.5190, "lng": -71.9785 }
}
```

---

## 8. Geolocalización en Tiempo Real

Para tracking en tiempo real de conductores/vehículos:

### 8.1 Tabla sugerida: `tracking_vehiculos`
```sql
CREATE TABLE tracking_vehiculos (
    id SERIAL PRIMARY KEY,
    id_vehiculo INTEGER REFERENCES vehiculos(id),
    id_conductor INTEGER REFERENCES conductores(id),
    id_file_tour INTEGER REFERENCES file_tours(id),
    coordenadas GEOGRAPHY(Point, 4326) NOT NULL,
    velocidad DECIMAL(5,2),  -- km/h
    heading DECIMAL(5,2),    -- Dirección en grados
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    bateria_porcentaje INTEGER,
    is_active BOOLEAN DEFAULT TRUE
);

-- Índice espacial
CREATE INDEX idx_tracking_vehiculos_coords ON tracking_vehiculos USING GIST(coordenadas);

-- Índice temporal para consultas recientes
CREATE INDEX idx_tracking_vehiculos_timestamp ON tracking_vehiculos(timestamp DESC);

-- Particionar por fecha para mejor performance
```

### 8.2 Endpoint WebSocket para tiempo real
```
WS /api/v1/tracking/subscribe?file_tour_id=123
```

El frontend se suscribe y recibe actualizaciones cada X segundos.

### 8.3 Endpoint para enviar ubicación (desde app conductor)
```
POST /api/v1/tracking/update
Body: {
    "id_file_tour": 123,
    "lat": -13.5250,
    "lng": -71.9653,
    "velocidad": 45.5,
    "heading": 180,
    "bateria": 85
}
```

---

## 9. Integración con Leaflet (Frontend)

### 9.1 Componente de Mapa para Tour
```tsx
// El developer de frontend implementará esto
interface TourMapProps {
    geoInicio?: GeoPoint;
    geoFin?: GeoPoint;
    ruta?: GeoRoute;
    onGeoInicioChange?: (point: GeoPoint) => void;
    onGeoFinChange?: (point: GeoPoint) => void;
    editable?: boolean;
}
```

### 9.2 Mapa de Tracking en Tiempo Real
```tsx
interface TrackingMapProps {
    fileTourId: number;
    geoRecojo: GeoPoint;
    currentLocation?: GeoPoint;  // Actualizado vía WebSocket
}
```

---

## 10. Plan de Implementación

### Fase 1: Backend (Actual)
1. ✅ Crear migración para agregar columnas geo (JSONB)
2. ✅ Actualizar modelos Diesel
3. ✅ Actualizar DTOs con GeoPoint
4. ✅ Crear endpoints para actualizar/consultar geolocalización
5. ✅ Tests de integración

### Fase 2: Frontend (Otro Developer)
1. Integrar Leaflet
2. Componente de mapa para tours
3. Componente de mapa para file_tours
4. Selector de punto en mapa (click to pick)
5. Visualización de rutas

### Fase 3: Tracking Tiempo Real (Futuro)
1. Crear tabla tracking_vehiculos
2. Implementar WebSocket/SSE
3. App para conductores (envía ubicación)
4. Dashboard de tracking en frontend

---

## 11. Decisión Final: JSONB vs PostGIS

**Recomendación: Usar JSONB inicialmente**

Razones:
1. Más simple de implementar con Diesel
2. Suficiente para mostrar puntos en mapas
3. Se puede migrar a PostGIS después si se necesitan búsquedas espaciales
4. El frontend (Leaflet) trabaja igual con ambos

Si en el futuro necesitamos:
- "Buscar tours a menos de 50km de mi ubicación"
- "Encontrar el tour más cercano"
- "Calcular distancia total de la ruta"

Entonces migramos a PostGIS.

---

## 12. Resumen de Cambios en Base de Datos

```sql
-- MIGRACIÓN FINAL (usando JSONB para simplicidad)
ALTER TABLE tours ADD COLUMN geo_inicio JSONB;
ALTER TABLE tours ADD COLUMN geo_fin JSONB;
ALTER TABLE tours ADD COLUMN geo_ruta JSONB;
ALTER TABLE tours ALTER COLUMN lugar_inicio DROP NOT NULL;
ALTER TABLE tours ALTER COLUMN lugar_fin DROP NOT NULL;

ALTER TABLE file_tours ADD COLUMN geo_recojo JSONB;

-- Índices GIN para búsquedas en JSONB
CREATE INDEX idx_tours_geo_inicio ON tours USING GIN(geo_inicio);
CREATE INDEX idx_tours_geo_fin ON tours USING GIN(geo_fin);
CREATE INDEX idx_file_tours_geo_recojo ON file_tours USING GIN(geo_recojo);
```

Formato del JSONB:
```json
{
    "lat": -13.5250,
    "lng": -71.9653,
    "zoom": 15,
    "label": "Plaza de Armas de Cusco"
}
```
