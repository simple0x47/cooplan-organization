use crate::logic;
use mongodb::bson::oid::ObjectId;
use serde::Deserialize;

pub const DATABASE: &str = "local";
pub const COLLECTION: &str = "invitation";

#[derive(Debug, Deserialize)]
pub struct Invitation {
    pub _id: ObjectId,
    pub code: String,
    pub organization_id: String,
    pub permissions: Vec<String>,
    /// Unix timestamp, seconds after the UNIX EPOCH
    pub created_at: u64,
    pub expires_after: u64,
}

impl Into<logic::elements::invitation::Invitation> for Invitation {
    fn into(self) -> logic::elements::invitation::Invitation {
        logic::elements::invitation::Invitation {
            code: self.code,
            organization_id: self.organization_id,
            permissions: self.permissions,
            created_at: self.created_at,
            expires_after: self.expires_after,
        }
    }
}
