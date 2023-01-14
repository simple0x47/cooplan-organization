use crate::logic::actions::organization_storage_action::OrganizationStorageAction;

pub enum StorageRequest {
    OrganizationRequest(OrganizationStorageAction),
}
