use crate::error::{Error, ErrorKind};
use crate::logic::actions::invitation_code_storage_action::InvitationStorageAction;
use crate::logic::elements::invitation::Invitation;
use crate::storage;
use cooplan_util::error_handler::ErrorHandler;
use mongodb::bson::doc;
use mongodb::Client;
use tokio::sync::oneshot::Sender;

pub async fn execute(action: InvitationStorageAction, client: &Client) -> Result<(), Error> {
    match action {
        InvitationStorageAction::Delete { code, replier } => {
            delete(code, replier, client).await?;
        }
        InvitationStorageAction::FindByCode { code, replier } => {
            find_by_code(code, replier, client).await?;
        }
    }

    Ok(())
}

async fn find_by_code(
    code: String,
    replier: Sender<Result<Option<Invitation>, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let filter = doc! { "code": code };

    let result = match client
        .database(storage::elements::invitation::DATABASE)
        .collection::<storage::elements::invitation::Invitation>(
            storage::elements::invitation::COLLECTION,
        )
        .find_one(Some(filter), None)
        .await
    {
        Ok(result) => match result {
            Some(invitation) => {
                let invitation: Invitation = invitation.into();
                Some(invitation)
            }
            None => None,
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::StorageFailure,
                format!("failed to find invitation by code: {}", error),
            );

            return replier.handle_error(error);
        }
    };

    match replier.send(Ok(result)) {
        Ok(_) => {}
        Err(_) => {
            return Err(Error::new(ErrorKind::StorageFailure, "failed to reply"));
        }
    }

    Ok(())
}

async fn delete(
    code: String,
    replier: Sender<Result<(), Error>>,
    client: &Client,
) -> Result<(), Error> {
    let filter = doc! { "code": code };

    match client
        .database(storage::elements::invitation::DATABASE)
        .collection::<storage::elements::invitation::Invitation>(
            storage::elements::invitation::COLLECTION,
        )
        .delete_one(filter, None)
        .await
    {
        Ok(_) => {}
        Err(error) => {
            let error = Error::new(
                ErrorKind::StorageFailure,
                format!("failed to delete invitation by code: {}", error),
            );

            return replier.handle_error(error);
        }
    }

    match replier.send(Ok(())) {
        Ok(_) => {}
        Err(_) => {
            return Err(Error::new(ErrorKind::StorageFailure, "failed to reply"));
        }
    }

    Ok(())
}
