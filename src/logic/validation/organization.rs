use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::logic::elements::organization::Organization;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;

pub async fn get_organization_if_exists(
    id: String,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<Organization, Error> {
    let (replier, receiver) = tokio::sync::oneshot::channel();

    let request =
        StorageRequest::OrganizationRequest(OrganizationStorageAction::FindById { id, replier });

    match storage_request_sender.send(request).await {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!("failed to send storage request: {}", error),
            ));
        }
    }

    let organization = match receiver.await {
        Ok(result) => match result {
            Ok(optional_organization) => match optional_organization {
                Some(organization) => organization,
                None => {
                    let error =
                        Error::new(ErrorKind::OrganizationNotFound, "organization not found");

                    return Err(error);
                }
            },
            Err(error) => return Err(error),
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!(
                    "failed to receive response for a storage request: {}",
                    error
                ),
            ));
        }
    };

    Ok(organization)
}
