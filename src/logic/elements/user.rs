use crate::logic::elements::user_organization::UserOrganization;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct User {
    pub id: String,
    pub organizations: Vec<UserOrganization>,
}

impl User {
    pub fn new(user_id: String) -> User {
        User {
            id: user_id,
            organizations: vec![UserOrganization {
                organization_id: "".to_string(),
                permissions: vec![
                    "create:organization".to_string(),
                    "join:organization".to_string(),
                ],
            }],
        }
    }
}
