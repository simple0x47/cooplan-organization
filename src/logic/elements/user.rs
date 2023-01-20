use crate::logic::elements::user_organization::UserOrganization;

#[derive(Debug)]
pub struct User {
    pub id: String,
    pub organizations: Vec<UserOrganization>,
}
