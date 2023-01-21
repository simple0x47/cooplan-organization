use crate::error::{Error, ErrorKind};
use crate::logic::actions::invitation_code_storage_action::InvitationStorageAction;
use crate::logic::elements::invitation::Invitation;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;

pub async fn get_code_if_valid(
    invitation_code: String,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<Invitation, Error> {
    let (replier, receiver) = tokio::sync::oneshot::channel();

    match storage_request_sender
        .send(StorageRequest::InvitationRequest(
            InvitationStorageAction::FindByCode {
                code: invitation_code,
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

    let invitation = match receiver.await {
        Ok(result) => match result {
            Ok(optional_invitation) => match optional_invitation {
                Some(invitation) => invitation,
                None => {
                    let error =
                        Error::new(ErrorKind::InvitationNotFound, "invitation code not found");

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

    if invitation.expired() {
        return Err(Error::new(
            ErrorKind::InvitationHasExpired,
            "invitation code has expired",
        ));
    }

    Ok(invitation)
}
