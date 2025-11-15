use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

/// Information about a user in a room
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub addr: SocketAddr,
    pub user_id: u8,
    pub username: String,
    pub color: [u8; 3],
}

/// Represents a board room where users collaborate
#[derive(Debug)]
pub struct Room {
    /// Board ID
    board_id: u16,

    /// Map of socket addresses to user info
    users: HashMap<SocketAddr, UserInfo>,

    /// Set of available user IDs (0-255)
    available_ids: HashSet<u8>,

    /// Set of currently assigned user IDs
    assigned_ids: HashSet<u8>,
}

impl Room {
    /// Create a new room
    pub fn new(board_id: u16) -> Self {
        // Initialize all 256 IDs as available (0-255)
        let available_ids: HashSet<u8> = (0..=255).collect();

        Self {
            board_id,
            users: HashMap::new(),
            available_ids,
            assigned_ids: HashSet::new(),
        }
    }

    /// Get board ID
    pub fn board_id(&self) -> u16 {
        self.board_id
    }

    /// Assign a user ID (returns lowest available ID)
    pub fn assign_user_id(&mut self) -> Option<u8> {
        // Find the lowest available ID
        let id = (0..=255u8).find(|id| self.available_ids.contains(id))?;

        self.available_ids.remove(&id);
        self.assigned_ids.insert(id);

        Some(id)
    }

    /// Release a user ID back to the pool
    fn release_user_id(&mut self, id: u8) {
        self.assigned_ids.remove(&id);
        self.available_ids.insert(id);
    }

    /// Add a user to the room
    pub fn add_user(&mut self, addr: SocketAddr, user_id: u8, username: String, color: [u8; 3]) {
        let user_info = UserInfo {
            addr,
            user_id,
            username,
            color,
        };
        self.users.insert(addr, user_info);
    }

    /// Remove a user from the room
    pub fn remove_user(&mut self, addr: SocketAddr) {
        if let Some(user_info) = self.users.remove(&addr) {
            self.release_user_id(user_info.user_id);
        }
    }

    /// Get user info by address
    pub fn get_user(&self, addr: &SocketAddr) -> Option<&UserInfo> {
        self.users.get(addr)
    }

    /// Get all user addresses in the room
    pub fn user_addresses(&self) -> Vec<SocketAddr> {
        self.users.keys().copied().collect()
    }

    /// Get user count
    pub fn user_count(&self) -> usize {
        self.users.len()
    }

    /// Check if room is empty
    pub fn is_empty(&self) -> bool {
        self.users.is_empty()
    }

    /// Get all users
    pub fn users(&self) -> impl Iterator<Item = &UserInfo> {
        self.users.values()
    }

    /// Check if a user is in the room
    pub fn contains_user(&self, addr: &SocketAddr) -> bool {
        self.users.contains_key(addr)
    }

    /// Get available ID count
    #[allow(dead_code)]
    pub fn available_id_count(&self) -> usize {
        self.available_ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_room_creation() {
        let room = Room::new(1);
        assert_eq!(room.board_id(), 1);
        assert_eq!(room.user_count(), 0);
        assert!(room.is_empty());
        assert_eq!(room.available_id_count(), 256);
    }

    #[test]
    fn test_user_id_assignment() {
        let mut room = Room::new(1);

        // Assign IDs - should get 0, 1, 2, etc.
        let id1 = room.assign_user_id().unwrap();
        let id2 = room.assign_user_id().unwrap();
        let id3 = room.assign_user_id().unwrap();

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id3, 2);

        assert_eq!(room.available_id_count(), 253);
    }

    #[test]
    fn test_user_id_recycling() {
        let mut room = Room::new(1);

        // Assign an ID
        let id1 = room.assign_user_id().unwrap();
        assert_eq!(id1, 0);

        // Release it
        room.release_user_id(id1);

        // Assign again - should get the same ID
        let id2 = room.assign_user_id().unwrap();
        assert_eq!(id2, 0);
    }

    #[test]
    fn test_add_remove_user() {
        let mut room = Room::new(1);
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

        // Add user
        let user_id = room.assign_user_id().unwrap();
        room.add_user(addr, user_id, "Alice".to_string(), [255, 0, 0]);

        assert_eq!(room.user_count(), 1);
        assert!(room.contains_user(&addr));

        let user_info = room.get_user(&addr).unwrap();
        assert_eq!(user_info.user_id, user_id);
        assert_eq!(user_info.username, "Alice");
        assert_eq!(user_info.color, [255, 0, 0]);

        // Remove user
        room.remove_user(addr);
        assert_eq!(room.user_count(), 0);
        assert!(!room.contains_user(&addr));

        // ID should be recycled
        let new_id = room.assign_user_id().unwrap();
        assert_eq!(new_id, user_id);
    }

    #[test]
    fn test_max_users() {
        let mut room = Room::new(1);

        // Assign all 256 IDs
        for _ in 0..256 {
            assert!(room.assign_user_id().is_some());
        }

        // Next assignment should fail
        assert!(room.assign_user_id().is_none());
        assert_eq!(room.available_id_count(), 0);
    }

    #[test]
    fn test_user_addresses() {
        let mut room = Room::new(1);

        let addr1 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
        let addr2 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);
        let addr3 = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8082);

        let id1 = room.assign_user_id().unwrap();
        let id2 = room.assign_user_id().unwrap();
        let id3 = room.assign_user_id().unwrap();

        room.add_user(addr1, id1, "Alice".to_string(), [255, 0, 0]);
        room.add_user(addr2, id2, "Bob".to_string(), [0, 255, 0]);
        room.add_user(addr3, id3, "Charlie".to_string(), [0, 0, 255]);

        let addresses = room.user_addresses();
        assert_eq!(addresses.len(), 3);
        assert!(addresses.contains(&addr1));
        assert!(addresses.contains(&addr2));
        assert!(addresses.contains(&addr3));
    }
}
