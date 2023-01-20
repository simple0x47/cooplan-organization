use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct UserOrganization {
    pub organization_id: String,
    pub permissions: Vec<String>,
}
