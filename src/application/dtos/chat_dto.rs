use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ChatNoteRequest {
    pub nota: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ChatNoteResponse {
    pub notas: String,
}

/// Request para actualizar una nota específica en file_tour (JSONB)
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct UpdateChatNoteRequest {
    pub nota: String,
}

/// Response JSONB para notas de file_tours
#[derive(Debug, Clone, Deserialize, Serialize, TS)]
#[ts(export)]
#[ts(export_to = "../../frontend/src/domain/contracts/")]
pub struct ChatNoteJsonResponse {
    #[ts(type = "any")]
    pub notas: JsonValue,
}
