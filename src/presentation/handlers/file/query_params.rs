//! Query parameters para File

use serde::Deserialize;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery { 
    pub from: NaiveDate, 
    pub to: NaiveDate 
}

#[derive(Debug, Deserialize, Default)]
pub struct EntidadQuery {
    pub entidad: Option<String>,
}
