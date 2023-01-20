use crate::error::{Error, ErrorKind};
use crate::logic::actions::user_storage_action::UserStorageAction;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;

pub async fn has_user_no_organization(
    user_id: &str,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<bool, Error> {
    let (replier, receiver) = tokio::sync::oneshot::channel();

    match storage_request_sender
        .send(StorageRequest::UserRequest(
            UserStorageAction::FindUserById {
                user_id: user_id.to_string(),
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
            ));
        }
    }

    let user = match receiver.await {
        Ok(result) => match result {
            Ok(optional_user) => optional_user,
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

    // User can only create an organization if it has no organizations.
    let result = match user {
        Some(user) => user.organizations.is_empty(),
        None => true,
    };

    Ok(result)
}
