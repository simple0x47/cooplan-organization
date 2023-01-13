use crate::logic::actions::example_storage_action::ExampleStorageAction;

pub enum StorageRequest {
    ExampleRequest(ExampleStorageAction),
}
