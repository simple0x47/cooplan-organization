use mongodb::bson::{doc, Bson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserOrganization {
    pub organization_id: String,
    pub permissions: Vec<String>,
}

impl Into<Bson> for UserOrganization {
    fn into(self) -> Bson {
        Bson::Document(doc! {
            "organization_id": self.organization_id,
            "permissions": self.permissions,
        })
    }
}
