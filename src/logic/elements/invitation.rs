use serde::Serialize;
use std::ops::Sub;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

#[derive(Debug, Serialize)]
pub struct Invitation {
    pub code: String,
    pub organization_id: String,
    pub permissions: Vec<String>,
    /// Unix timestamp, seconds after the UNIX EPOCH
    pub created_at: u64,
    pub expires_after: u64,
}

impl Invitation {
    pub fn created_at(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.created_at)
    }

    pub fn expired(&self) -> bool {
        self.created_at() + Duration::from_secs(self.expires_after) <= SystemTime::now()
    }
}

#[cfg(test)]
#[test]
fn expires_correctly() {
    const EXPIRE_AFTER: u64 = 3;
    let created_at = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    let invitation = Invitation {
        code: "test".to_string(),
        organization_id: "test".to_string(),
        permissions: vec![],
        created_at: created_at.as_secs(),
        expires_after: EXPIRE_AFTER,
    };

    assert_eq!(false, invitation.expired());
    // Wait one second more in order to avoid possible time drifts.
    sleep(Duration::from_secs(EXPIRE_AFTER + 1));
    assert_eq!(true, invitation.expired());
}
