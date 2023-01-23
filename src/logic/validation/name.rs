use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;

pub async fn is_name_already_used(
    name: &str,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<bool, Error> {
    let (replier, listener) = tokio::sync::oneshot::channel();

    match storage_request_sender
        .send(StorageRequest::Organization(
            OrganizationStorageAction::FindByName {
                name: name.to_string(),
                replier,
            },
        ))
        .await
    {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!("failed to send storage request: {}", error),
            ))
        }
    }

    let result = match listener.await {
        Ok(result) => match result {
            Ok(optional_organization) => optional_organization.is_some(),
            Err(error) => return Err(error),
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!(
                    "failed to receive response for a storage request: {}",
                    error
                ),
            ))
        }
    };

    Ok(result)
}
