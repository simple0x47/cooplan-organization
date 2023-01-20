use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub country: String,
    pub address: String,
    pub telephone: String,
}
