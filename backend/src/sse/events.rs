use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// SSE event types that mirror the WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SseEvent {
    // Board events
    BoardUpdated {
        board: crate::models::board::Board,
    },

    // Column events
    ColumnCreated {
        column: crate::models::column::Column,
    },
    ColumnUpdated {
        column: crate::models::column::Column,
    },
    ColumnDeleted {
        column_id: Uuid,
    },
    ColumnReordered {
        column_id: Uuid,
        new_position: i32,
    },

    // Card events
    CardCreated {
        card: crate::models::card::Card,
    },
    CardUpdated {
        card: crate::models::card::Card,
    },
    CardDeleted {
        card_id: Uuid,
    },
    CardMoved {
        card_id: Uuid,
        from_column_id: Uuid,
        to_column_id: Uuid,
        new_position: i32,
    },
    CardReordered {
        card_id: Uuid,
        column_id: Uuid,
        new_position: i32,
    },

    // Label events
    LabelCreated {
        label: crate::models::label::Label,
    },
    LabelUpdated {
        label: crate::models::label::Label,
    },
    LabelDeleted {
        label_id: Uuid,
    },
    LabelAssigned {
        card_id: Uuid,
        label_id: Uuid,
    },
    LabelUnassigned {
        card_id: Uuid,
        label_id: Uuid,
    },
}

impl SseEvent {
    /// Get the event name for the SSE stream
    pub fn event_name(&self) -> &'static str {
        match self {
            SseEvent::BoardUpdated { .. } => "board:updated",
            SseEvent::ColumnCreated { .. } => "column:created",
            SseEvent::ColumnUpdated { .. } => "column:updated",
            SseEvent::ColumnDeleted { .. } => "column:deleted",
            SseEvent::ColumnReordered { .. } => "column:reordered",
            SseEvent::CardCreated { .. } => "card:created",
            SseEvent::CardUpdated { .. } => "card:updated",
            SseEvent::CardDeleted { .. } => "card:deleted",
            SseEvent::CardMoved { .. } => "card:moved",
            SseEvent::CardReordered { .. } => "card:reordered",
            SseEvent::LabelCreated { .. } => "label:created",
            SseEvent::LabelUpdated { .. } => "label:updated",
            SseEvent::LabelDeleted { .. } => "label:deleted",
            SseEvent::LabelAssigned { .. } => "label:assigned",
            SseEvent::LabelUnassigned { .. } => "label:unassigned",
        }
    }

    /// Serialize the event to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
