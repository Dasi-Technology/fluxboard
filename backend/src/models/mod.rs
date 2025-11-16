//! Database models module
//!
//! This module contains all database models and their associated methods.
//! Each model corresponds to a database table and provides CRUD operations.

pub mod board;
pub mod card;
pub mod column;
pub mod label;

// Re-export models for easier imports
pub use board::{Board, BoardWithRelations, CreateBoardInput, UpdateBoardInput};
pub use card::{Card, CreateCardInput, UpdateCardInput};
pub use column::{Column, CreateColumnInput, UpdateColumnInput};
pub use label::{BoardLabel, CardLabel, CreateBoardLabelInput, UpdateBoardLabelInput};
