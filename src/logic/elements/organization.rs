use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub country: String,
    pub address: String,
    pub telephone: String,
    pub permissions: Vec<String>,
}
