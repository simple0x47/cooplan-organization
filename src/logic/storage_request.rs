use crate::logic::actions::invitation_code_storage_action::InvitationStorageAction;
use crate::logic::actions::organization_root_storage_action::OrganizationRootStorageAction;
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::logic::actions::user_storage_action::UserStorageAction;

pub enum StorageRequest {
    OrganizationRequest(OrganizationStorageAction),
    UserRequest(UserStorageAction),
    InvitationRequest(InvitationStorageAction),
    OrganizationRootRequest(OrganizationRootStorageAction),
}
