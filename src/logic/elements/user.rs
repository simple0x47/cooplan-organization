use crate::logic::elements::user_organization::UserOrganization;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: String,
    pub organizations: Vec<UserOrganization>,
}
