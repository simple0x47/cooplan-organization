use crate::logic::elements::user_organization::UserOrganization;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub user_id: String,
    pub organizations: Vec<UserOrganization>,
}
