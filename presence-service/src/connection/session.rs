use std::collections::HashMap;
use std::net::SocketAddr;

/// Information about a user's participation in a specific board
#[derive(Debug, Clone)]
pub struct BoardInfo {
    pub user_id: u8,
    pub username: String,
    pub color: [u8; 3],
}

/// Represents a client session
#[derive(Debug, Clone)]
pub struct Session {
    /// Client socket address
    addr: SocketAddr,

    /// Map of board IDs to board-specific info
    boards: HashMap<u16, BoardInfo>,
}

impl Session {
    /// Create a new session
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            boards: HashMap::new(),
        }
    }

    /// Get client address
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Add a board to the session
    pub fn add_board(&mut self, board_id: u16, user_id: u8, username: String, color: [u8; 3]) {
        self.boards.insert(
            board_id,
            BoardInfo {
                user_id,
                username,
                color,
            },
        );
    }

    /// Remove a board from the session
    pub fn remove_board(&mut self, board_id: u16) {
        self.boards.remove(&board_id);
    }

    /// Get board info for a specific board
    pub fn get_board_info(&self, board_id: u16) -> Option<&BoardInfo> {
        self.boards.get(&board_id)
    }

    /// Get all board IDs this session is part of
    pub fn board_ids(&self) -> Vec<u16> {
        self.boards.keys().copied().collect()
    }

    /// Check if session is in a specific board
    pub fn is_in_board(&self, board_id: u16) -> bool {
        self.boards.contains_key(&board_id)
    }

    /// Get number of boards this session is part of
    pub fn board_count(&self) -> usize {
        self.boards.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_session_creation() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let session = Session::new(addr);

        assert_eq!(session.addr(), addr);
        assert_eq!(session.board_count(), 0);
    }

    #[test]
    fn test_add_remove_board() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut session = Session::new(addr);

        // Add a board
        session.add_board(1, 5, "Alice".to_string(), [255, 0, 0]);
        assert_eq!(session.board_count(), 1);
        assert!(session.is_in_board(1));

        // Get board info
        let info = session.get_board_info(1).unwrap();
        assert_eq!(info.user_id, 5);
        assert_eq!(info.username, "Alice");
        assert_eq!(info.color, [255, 0, 0]);

        // Remove the board
        session.remove_board(1);
        assert_eq!(session.board_count(), 0);
        assert!(!session.is_in_board(1));
    }

    #[test]
    fn test_multiple_boards() {
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let mut session = Session::new(addr);

        // Add multiple boards
        session.add_board(1, 5, "Alice".to_string(), [255, 0, 0]);
        session.add_board(2, 3, "Alice".to_string(), [0, 255, 0]);
        session.add_board(3, 7, "Alice".to_string(), [0, 0, 255]);

        assert_eq!(session.board_count(), 3);
        assert!(session.is_in_board(1));
        assert!(session.is_in_board(2));
        assert!(session.is_in_board(3));

        let board_ids = session.board_ids();
        assert_eq!(board_ids.len(), 3);
        assert!(board_ids.contains(&1));
        assert!(board_ids.contains(&2));
        assert!(board_ids.contains(&3));
    }
}
