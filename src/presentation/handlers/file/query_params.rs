//! Query parameters para File

use serde::Deserialize;
use chrono::NaiveDate;

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery { 
    pub from: NaiveDate, 
    pub to: NaiveDate 
}
