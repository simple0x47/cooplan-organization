use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::logic::actions::user_storage_action::UserStorageAction;

pub enum StorageRequest {
    OrganizationRequest(OrganizationStorageAction),
    UserRequest(UserStorageAction),
}
