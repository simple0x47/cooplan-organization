use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;
use std::panic;

/// Warning: it is not supposed to work with specific international prefixes such as '00'.
/// The '+' sign must be used.
pub fn is_telephone_valid(telephone: &str) -> bool {
    match panic::catch_unwind(|| match phonenumber::parse(None, telephone.to_string()) {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }) {
        Ok(result) => result.is_ok(),
        Err(_) => false,
    }
}

pub async fn is_telephone_being_used(
    telephone: &str,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<bool, Error> {
    let (replier, listener) = tokio::sync::oneshot::channel();

    match storage_request_sender
        .send(StorageRequest::Organization(
            OrganizationStorageAction::FindByTelephone {
                telephone: telephone.to_string(),
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
