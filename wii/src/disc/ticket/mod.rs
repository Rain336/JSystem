mod key;

use aes::Aes128;
pub use key::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TimeLimit(Option<u32>);

impl TimeLimit {
    pub fn with_limit(seconds: u32) -> Self {
        TimeLimit(Some(seconds))
    }

    pub fn is_enabled(&self) -> bool {
        self.0.is_some()
    }

    pub fn seconds(&self) -> u32 {
        self.0.unwrap_or_default()
    }

    pub fn set_seconds(&mut self, seconds: u32) {
        self.0 = Some(seconds)
    }

    pub fn disable(&mut self) {
        self.0 = None
    }
}

impl From<Option<u32>> for TimeLimit {
    fn from(value: Option<u32>) -> Self {
        TimeLimit(value)
    }
}

impl From<u32> for TimeLimit {
    fn from(value: u32) -> Self {
        TimeLimit(Some(value))
    }
}

impl Default for TimeLimit {
    fn default() -> Self {
        TimeLimit(None)
    }
}

pub struct Ticket {
    signature: [u8; 256],
    issuer: [u8; 64],
    ecdh_data: [u8; 60],
    title_key: Aes128,
    ticket_id: u64,
    console_id: u32,
    title_id: u64,
    ticket_title_version: u16,
    permitted_titles_mask: u32,
    permit_mask: u32,
    title_export_allowed: bool,
    common_key_index: u8,
    content_access_permissions: [u8; 64],
    time_limit: [TimeLimit; 8],
}