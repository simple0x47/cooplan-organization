use crate::logic::elements::invitation::Invitation;
use crate::logic::elements::organization::Organization;
use crate::logic::elements::user::User;

#[derive(Debug)]
pub struct OrganizationRoot {
    pub organization: Organization,
    pub users: Vec<User>,
    pub invitations: Vec<Invitation>,
}
