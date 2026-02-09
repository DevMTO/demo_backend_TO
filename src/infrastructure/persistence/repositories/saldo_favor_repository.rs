//! Repositorio para Saldos a Favor, Cancelaciones y No Shows

use async_trait::async_trait;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use tracing::instrument;

use crate::application::dtos::{
    CancelacionResponse, SaldoFavorResponse, MovimientoSaldoFavorResponse,
    NoShowResponse,
};
use crate::domain::errors::ApplicationError;
use crate::infrastructure::persistence::database::DatabasePool;
use crate::infrastructure::persistence::models::{
    CancelacionModel, NewCancelacionModel,
    SaldoFavorModel,
    MovimientoSaldoFavorModel, NewMovimientoSaldoFavorModel,
    NoShowModel, NewNoShowModel,
};
use crate::infrastructure::persistence::schema::{cancelaciones, saldos_favor, movimientos_saldo_favor, no_shows};

/// Costos de restaurantes y entradas de un file
pub struct FileCosts {
    pub monto_restaurantes: BigDecimal,
    pub monto_entradas: BigDecimal,
    pub fecha_inicio_min: Option<NaiveDate>,
}

// ==================== PORT ====================

#[async_trait]
pub trait SaldoFavorRepositoryPort: Send + Sync {
    async fn get_saldo_by_agencia(&self, id_agencia: i32) -> Result<Option<SaldoFavorModel>, ApplicationError>;
    async fn update_saldo(&self, id: i32, saldo_disponible: BigDecimal, saldo_utilizado: BigDecimal, saldo_total_generado: BigDecimal) -> Result<SaldoFavorModel, ApplicationError>;
    
    async fn create_cancelacion(&self, data: NewCancelacionModel) -> Result<CancelacionModel, ApplicationError>;
    async fn list_cancelaciones(&self, id_agencia: Option<i32>, limit: i64, offset: i64) -> Result<Vec<CancelacionResponse>, ApplicationError>;
    async fn count_cancelaciones(&self, id_agencia: Option<i32>) -> Result<i64, ApplicationError>;
    async fn find_cancelacion_by_file(&self, id_file: i32) -> Result<Option<CancelacionModel>, ApplicationError>;
    
    async fn create_no_show(&self, data: NewNoShowModel) -> Result<NoShowModel, ApplicationError>;
    async fn list_no_shows(&self, id_agencia: Option<i32>, limit: i64, offset: i64) -> Result<Vec<NoShowResponse>, ApplicationError>;
    async fn count_no_shows(&self, id_agencia: Option<i32>) -> Result<i64, ApplicationError>;
    
    async fn create_movimiento(&self, data: NewMovimientoSaldoFavorModel) -> Result<MovimientoSaldoFavorModel, ApplicationError>;
    async fn list_movimientos(&self, id_agencia: Option<i32>, tipo: Option<&str>, limit: i64, offset: i64) -> Result<Vec<MovimientoSaldoFavorResponse>, ApplicationError>;
    
    async fn list_all_saldos(&self) -> Result<Vec<SaldoFavorResponse>, ApplicationError>;
    
    /// Calcula costos de restaurantes, entradas y la fecha_inicio más temprana del file
    async fn calculate_file_costs(&self, id_file: i32) -> Result<FileCosts, ApplicationError>;
}

// ==================== IMPLEMENTATION ====================

pub struct PostgresSaldoFavorRepository {
    pool: DatabasePool,
}

impl PostgresSaldoFavorRepository {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SaldoFavorRepositoryPort for PostgresSaldoFavorRepository {
    #[instrument(skip(self))]
    async fn get_saldo_by_agencia(&self, id_agencia: i32) -> Result<Option<SaldoFavorModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        saldos_favor::table
            .filter(saldos_favor::id_agencia.eq(id_agencia))
            .select(SaldoFavorModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    #[instrument(skip(self))]
    async fn update_saldo(&self, id: i32, saldo_disponible: BigDecimal, saldo_utilizado: BigDecimal, saldo_total_generado: BigDecimal) -> Result<SaldoFavorModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::update(saldos_favor::table.filter(saldos_favor::id.eq(id)))
            .set((
                saldos_favor::saldo_disponible.eq(&saldo_disponible),
                saldos_favor::saldo_utilizado.eq(&saldo_utilizado),
                saldos_favor::saldo_total_generado.eq(&saldo_total_generado),
                saldos_favor::updated_at.eq(chrono::Utc::now()),
            ))
            .returning(SaldoFavorModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error actualizando saldo: {}", e)))
    }
    
    #[instrument(skip(self))]
    async fn create_cancelacion(&self, data: NewCancelacionModel) -> Result<CancelacionModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::insert_into(cancelaciones::table)
            .values(&data)
            .returning(CancelacionModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error creando cancelación: {}", e)))
    }
    
    #[instrument(skip(self))]
    async fn list_cancelaciones(&self, id_agencia: Option<i32>, limit: i64, offset: i64) -> Result<Vec<CancelacionResponse>, ApplicationError> {
        use diesel::sql_types::{Integer, Nullable, Text, Numeric, Timestamptz};
        
        let mut conn = self.pool.get_connection().await?;
        
        let where_clause = if let Some(ag_id) = id_agencia {
            format!("WHERE c.id_agencia = {}", ag_id)
        } else {
            String::new()
        };
        
        let sql = format!(r#"
            SELECT c.id, c.id_file, c.id_agencia,
                c.monto_total_file, c.monto_pagado, c.monto_saldo_favor, c.monto_operador,
                c.tipo_cancelacion, c.motivo, c.notas,
                c.created_at, c.created_by,
                f.file_code, a.nombre as agencia_nombre
            FROM cancelaciones c
            INNER JOIN files f ON f.id = c.id_file
            INNER JOIN agencias a ON a.id = c.id_agencia
            {}
            ORDER BY c.created_at DESC
            LIMIT {} OFFSET {}
        "#, where_clause, limit, offset);
        
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Integer)] id: i32,
            #[diesel(sql_type = Integer)] id_file: i32,
            #[diesel(sql_type = Integer)] id_agencia: i32,
            #[diesel(sql_type = Numeric)] monto_total_file: BigDecimal,
            #[diesel(sql_type = Numeric)] monto_pagado: BigDecimal,
            #[diesel(sql_type = Numeric)] monto_saldo_favor: BigDecimal,
            #[diesel(sql_type = Numeric)] monto_operador: BigDecimal,
            #[diesel(sql_type = Text)] tipo_cancelacion: String,
            #[diesel(sql_type = Nullable<Text>)] motivo: Option<String>,
            #[diesel(sql_type = Nullable<Text>)] notas: Option<String>,
            #[diesel(sql_type = Timestamptz)] created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = Nullable<Integer>)] created_by: Option<i32>,
            #[diesel(sql_type = Nullable<Text>)] file_code: Option<String>,
            #[diesel(sql_type = Text)] agencia_nombre: String,
        }
        
        let rows: Vec<Row> = diesel::sql_query(sql)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error listando cancelaciones: {}", e)))?;
        
        Ok(rows.into_iter().map(|r| {
            use bigdecimal::ToPrimitive;
            CancelacionResponse {
                id: r.id,
                id_file: r.id_file,
                id_agencia: r.id_agencia,
                monto_total_file: r.monto_total_file.to_f64().unwrap_or(0.0),
                monto_pagado: r.monto_pagado.to_f64().unwrap_or(0.0),
                monto_saldo_favor: r.monto_saldo_favor.to_f64().unwrap_or(0.0),
                monto_operador: r.monto_operador.to_f64().unwrap_or(0.0),
                tipo_cancelacion: r.tipo_cancelacion,
                motivo: r.motivo,
                notas: r.notas,
                created_at: r.created_at,
                created_by: r.created_by,
                file_code: r.file_code,
                agencia_nombre: Some(r.agencia_nombre),
            }
        }).collect())
    }
    
    #[instrument(skip(self))]
    async fn count_cancelaciones(&self, id_agencia: Option<i32>) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = cancelaciones::table.into_boxed();
        if let Some(ag_id) = id_agencia {
            query = query.filter(cancelaciones::id_agencia.eq(ag_id));
        }
        
        query.count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    #[instrument(skip(self))]
    async fn find_cancelacion_by_file(&self, id_file: i32) -> Result<Option<CancelacionModel>, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        cancelaciones::table
            .filter(cancelaciones::id_file.eq(id_file))
            .select(CancelacionModel::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    // ==================== NO SHOWS ====================
    
    #[instrument(skip(self))]
    async fn create_no_show(&self, data: NewNoShowModel) -> Result<NoShowModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::insert_into(no_shows::table)
            .values(&data)
            .returning(NoShowModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error creando no_show: {}", e)))
    }
    
    #[instrument(skip(self))]
    async fn list_no_shows(&self, id_agencia: Option<i32>, limit: i64, offset: i64) -> Result<Vec<NoShowResponse>, ApplicationError> {
        use diesel::sql_types::{Integer, Nullable, Text, Numeric, Date, Timestamptz};
        
        let mut conn = self.pool.get_connection().await?;
        
        let where_clause = if let Some(ag_id) = id_agencia {
            format!("WHERE ns.id_agencia = {}", ag_id)
        } else {
            String::new()
        };
        
        let sql = format!(r#"
            SELECT ns.id, ns.id_cancelacion, ns.id_file, ns.id_agencia,
                ns.monto_restaurantes, ns.monto_entradas, ns.monto_saldo_favor, ns.monto_operador,
                ns.fecha_inicio_file, ns.hora_corte, ns.notas,
                ns.created_at, ns.created_by,
                f.file_code, a.nombre as agencia_nombre
            FROM no_shows ns
            INNER JOIN files f ON f.id = ns.id_file
            INNER JOIN agencias a ON a.id = ns.id_agencia
            {}
            ORDER BY ns.created_at DESC
            LIMIT {} OFFSET {}
        "#, where_clause, limit, offset);
        
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Integer)] id: i32,
            #[diesel(sql_type = Integer)] id_cancelacion: i32,
            #[diesel(sql_type = Integer)] id_file: i32,
            #[diesel(sql_type = Integer)] id_agencia: i32,
            #[diesel(sql_type = Numeric)] monto_restaurantes: BigDecimal,
            #[diesel(sql_type = Numeric)] monto_entradas: BigDecimal,
            #[diesel(sql_type = Numeric)] monto_saldo_favor: BigDecimal,
            #[diesel(sql_type = Numeric)] monto_operador: BigDecimal,
            #[diesel(sql_type = Date)] fecha_inicio_file: NaiveDate,
            #[diesel(sql_type = Timestamptz)] hora_corte: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = Nullable<Text>)] notas: Option<String>,
            #[diesel(sql_type = Timestamptz)] created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = Nullable<Integer>)] created_by: Option<i32>,
            #[diesel(sql_type = Nullable<Text>)] file_code: Option<String>,
            #[diesel(sql_type = Text)] agencia_nombre: String,
        }
        
        let rows: Vec<Row> = diesel::sql_query(sql)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error listando no_shows: {}", e)))?;
        
        Ok(rows.into_iter().map(|r| {
            use bigdecimal::ToPrimitive;
            NoShowResponse {
                id: r.id,
                id_cancelacion: r.id_cancelacion,
                id_file: r.id_file,
                id_agencia: r.id_agencia,
                monto_restaurantes: r.monto_restaurantes.to_f64().unwrap_or(0.0),
                monto_entradas: r.monto_entradas.to_f64().unwrap_or(0.0),
                monto_saldo_favor: r.monto_saldo_favor.to_f64().unwrap_or(0.0),
                monto_operador: r.monto_operador.to_f64().unwrap_or(0.0),
                fecha_inicio_file: r.fecha_inicio_file,
                hora_corte: r.hora_corte,
                notas: r.notas,
                created_at: r.created_at,
                created_by: r.created_by,
                file_code: r.file_code,
                agencia_nombre: Some(r.agencia_nombre),
            }
        }).collect())
    }
    
    #[instrument(skip(self))]
    async fn count_no_shows(&self, id_agencia: Option<i32>) -> Result<i64, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        let mut query = no_shows::table.into_boxed();
        if let Some(ag_id) = id_agencia {
            query = query.filter(no_shows::id_agencia.eq(ag_id));
        }
        
        query.count()
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(e.to_string()))
    }
    
    // ==================== MOVIMIENTOS ====================
    
    #[instrument(skip(self))]
    async fn create_movimiento(&self, data: NewMovimientoSaldoFavorModel) -> Result<MovimientoSaldoFavorModel, ApplicationError> {
        let mut conn = self.pool.get_connection().await?;
        
        diesel::insert_into(movimientos_saldo_favor::table)
            .values(&data)
            .returning(MovimientoSaldoFavorModel::as_returning())
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error creando movimiento de saldo: {}", e)))
    }
    
    #[instrument(skip(self))]
    async fn list_movimientos(&self, id_agencia: Option<i32>, tipo: Option<&str>, limit: i64, offset: i64) -> Result<Vec<MovimientoSaldoFavorResponse>, ApplicationError> {
        use diesel::sql_types::{Integer, Nullable, Text, Numeric, Timestamptz};
        
        let mut conn = self.pool.get_connection().await?;
        
        let mut where_clauses = vec!["1=1".to_string()];
        if let Some(ag_id) = id_agencia {
            where_clauses.push(format!("m.id_agencia = {}", ag_id));
        }
        if let Some(t) = tipo {
            where_clauses.push(format!("m.tipo = '{}'", t));
        }
        
        let where_sql = where_clauses.join(" AND ");
        
        let sql = format!(r#"
            SELECT m.id, m.id_agencia, m.tipo, m.monto,
                m.id_cancelacion, m.id_file_destino, m.id_pago_file,
                m.saldo_anterior, m.saldo_posterior,
                m.concepto, m.created_at, m.created_by,
                f_origen.file_code as file_code_origen,
                f_destino.file_code as file_code_destino
            FROM movimientos_saldo_favor m
            LEFT JOIN cancelaciones c ON c.id = m.id_cancelacion
            LEFT JOIN files f_origen ON f_origen.id = c.id_file
            LEFT JOIN files f_destino ON f_destino.id = m.id_file_destino
            WHERE {}
            ORDER BY m.created_at DESC
            LIMIT {} OFFSET {}
        "#, where_sql, limit, offset);
        
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Integer)] id: i32,
            #[diesel(sql_type = Integer)] id_agencia: i32,
            #[diesel(sql_type = Text)] tipo: String,
            #[diesel(sql_type = Numeric)] monto: BigDecimal,
            #[diesel(sql_type = Nullable<Integer>)] id_cancelacion: Option<i32>,
            #[diesel(sql_type = Nullable<Integer>)] id_file_destino: Option<i32>,
            #[diesel(sql_type = Nullable<Integer>)] id_pago_file: Option<i32>,
            #[diesel(sql_type = Numeric)] saldo_anterior: BigDecimal,
            #[diesel(sql_type = Numeric)] saldo_posterior: BigDecimal,
            #[diesel(sql_type = Nullable<Text>)] concepto: Option<String>,
            #[diesel(sql_type = Timestamptz)] created_at: chrono::DateTime<chrono::Utc>,
            #[diesel(sql_type = Nullable<Integer>)] created_by: Option<i32>,
            #[diesel(sql_type = Nullable<Text>)] file_code_origen: Option<String>,
            #[diesel(sql_type = Nullable<Text>)] file_code_destino: Option<String>,
        }
        
        let rows: Vec<Row> = diesel::sql_query(sql)
            .load(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error listando movimientos: {}", e)))?;
        
        Ok(rows.into_iter().map(|r| {
            use bigdecimal::ToPrimitive;
            MovimientoSaldoFavorResponse {
                id: r.id,
                id_agencia: r.id_agencia,
                tipo: r.tipo,
                monto: r.monto.to_f64().unwrap_or(0.0),
                id_cancelacion: r.id_cancelacion,
                id_file_destino: r.id_file_destino,
                id_pago_file: r.id_pago_file,
                saldo_anterior: r.saldo_anterior.to_f64().unwrap_or(0.0),
                saldo_posterior: r.saldo_posterior.to_f64().unwrap_or(0.0),
                concepto: r.concepto,
                created_at: r.created_at,
                created_by: r.created_by,
                file_code_origen: r.file_code_origen,
                file_code_destino: r.file_code_destino,
            }
        }).collect())
    }
    
    // ==================== SALDOS ====================
    
    #[instrument(skip(self))]
    async fn list_all_saldos(&self) -> Result<Vec<SaldoFavorResponse>, ApplicationError> {
        use diesel::sql_types::{Integer, Text, Numeric, Timestamptz};
        
        let mut conn = self.pool.get_connection().await?;
        
        #[derive(QueryableByName)]
        struct Row {
            #[diesel(sql_type = Integer)] id: i32,
            #[diesel(sql_type = Integer)] id_agencia: i32,
            #[diesel(sql_type = Text)] agencia_nombre: String,
            #[diesel(sql_type = Numeric)] saldo_disponible: BigDecimal,
            #[diesel(sql_type = Numeric)] saldo_utilizado: BigDecimal,
            #[diesel(sql_type = Numeric)] saldo_total_generado: BigDecimal,
            #[diesel(sql_type = Timestamptz)] updated_at: chrono::DateTime<chrono::Utc>,
        }
        
        let rows: Vec<Row> = diesel::sql_query(r#"
            SELECT s.id, s.id_agencia, a.nombre as agencia_nombre,
                s.saldo_disponible, s.saldo_utilizado, s.saldo_total_generado,
                s.updated_at
            FROM saldos_favor s
            INNER JOIN agencias a ON a.id = s.id_agencia
            ORDER BY a.nombre ASC
        "#)
        .load(&mut conn)
        .await
        .map_err(|e| ApplicationError::Repository(format!("Error listando saldos: {}", e)))?;
        
        Ok(rows.into_iter().map(|r| {
            use bigdecimal::ToPrimitive;
            SaldoFavorResponse {
                id: r.id,
                id_agencia: r.id_agencia,
                agencia_nombre: Some(r.agencia_nombre),
                saldo_disponible: r.saldo_disponible.to_f64().unwrap_or(0.0),
                saldo_utilizado: r.saldo_utilizado.to_f64().unwrap_or(0.0),
                saldo_total_generado: r.saldo_total_generado.to_f64().unwrap_or(0.0),
                updated_at: r.updated_at,
            }
        }).collect())
    }
    
    // ==================== FILE COSTS ====================
    
    #[instrument(skip(self))]
    async fn calculate_file_costs(&self, id_file: i32) -> Result<FileCosts, ApplicationError> {
        use diesel::sql_types::{Numeric, Nullable, Date};
        
        let mut conn = self.pool.get_connection().await?;
        
        // Costo total de restaurantes del file
        #[derive(QueryableByName)]
        struct RestRow {
            #[diesel(sql_type = Numeric)]
            total: BigDecimal,
        }
        
        let rest_sql = format!(r#"
            SELECT COALESCE(SUM(fr.precio), 0) as total
            FROM file_restaurantes fr
            INNER JOIN file_tours ft ON ft.id = fr.id_file_tour
            WHERE ft.id_file = {} AND fr.status != 'anulado'
        "#, id_file);
        
        let rest_row: RestRow = diesel::sql_query(rest_sql)
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error calculando costos restaurantes: {}", e)))?;
        
        // Costo total de entradas del file (cantidad × precio)
        #[derive(QueryableByName)]
        struct EntRow {
            #[diesel(sql_type = Numeric)]
            total: BigDecimal,
        }
        
        let ent_sql = format!(r#"
            SELECT COALESCE(SUM(fe.cantidad * ep.precio), 0) as total
            FROM file_entradas fe
            INNER JOIN entrada_precios ep ON ep.id = fe.id_entrada_precio
            INNER JOIN file_tours ft ON ft.id = fe.id_file_tour
            WHERE ft.id_file = {} AND fe.status != 'anulado'
        "#, id_file);
        
        let ent_row: EntRow = diesel::sql_query(ent_sql)
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error calculando costos entradas: {}", e)))?;
        
        // Fecha de inicio más temprana de los file_tours
        #[derive(QueryableByName)]
        struct FechaRow {
            #[diesel(sql_type = Nullable<Date>)]
            fecha_min: Option<NaiveDate>,
        }
        
        let fecha_sql = format!(r#"
            SELECT MIN(ft.fecha_tour) as fecha_min
            FROM file_tours ft
            WHERE ft.id_file = {}
        "#, id_file);
        
        let fecha_row: FechaRow = diesel::sql_query(fecha_sql)
            .get_result(&mut conn)
            .await
            .map_err(|e| ApplicationError::Repository(format!("Error obteniendo fecha inicio: {}", e)))?;
        
        Ok(FileCosts {
            monto_restaurantes: rest_row.total,
            monto_entradas: ent_row.total,
            fecha_inicio_min: fecha_row.fecha_min,
        })
    }
}
