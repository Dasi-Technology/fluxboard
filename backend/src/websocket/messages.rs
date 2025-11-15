use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::{Board, Card, Column, Label};

/// WebSocket message types for real-time updates
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessageType {
    // Board events
    BoardCreated(Board),
    BoardUpdated(Board),
    BoardDeleted {
        id: Uuid,
    },

    // Column events
    ColumnCreated(Column),
    ColumnUpdated(Column),
    ColumnDeleted {
        id: Uuid,
    },
    ColumnsReordered {
        board_id: Uuid,
        column_positions: Vec<(Uuid, i32)>,
    },

    // Card events
    CardCreated(Card),
    CardUpdated(Card),
    CardDeleted {
        id: Uuid,
    },
    CardMoved {
        id: Uuid,
        column_id: Uuid,
        position: i32,
    },
    CardsReordered {
        column_id: Uuid,
        card_positions: Vec<(Uuid, i32)>,
    },

    // Label events
    LabelCreated(Label),
    LabelUpdated(Label),
    LabelDeleted {
        id: Uuid,
    },

    // Connection events
    UserJoined {
        board_id: Uuid,
        user_count: usize,
    },
    UserLeft {
        board_id: Uuid,
        user_count: usize,
    },
    Error {
        message: String,
    },
}

/// WebSocket message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    pub board_id: Uuid,
    pub message_type: WsMessageType,
}

impl WsMessage {
    /// Create a new WebSocket message
    pub fn new(board_id: Uuid, message_type: WsMessageType) -> Self {
        Self {
            board_id,
            message_type,
        }
    }

    /// Convert message to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Parse message from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}
