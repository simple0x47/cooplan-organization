use crate::error::{Error, ErrorKind};
use crate::logic::actions::user_logic_action::UserLogicAction;
use crate::logic::actions::user_storage_action::UserStorageAction;
use crate::logic::elements::user::User;
use crate::logic::elements::user_organization::UserOrganization;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;
use cooplan_util::error_handler::ErrorHandler;

pub async fn execute(
    action: UserLogicAction,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<(), Error> {
    match action {
        UserLogicAction::Read { user_id, replier } => {
            read(user_id, replier, storage_request_sender).await?;
        }
    }
    Ok(())
}

async fn read(
    user_id: String,
    replier: tokio::sync::oneshot::Sender<Result<User, Error>>,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<(), Error> {
    let (storage_replier, storage_receiver) = tokio::sync::oneshot::channel();

    let action = UserStorageAction::FindUserById {
        user_id: user_id.clone(),
        replier: storage_replier,
    };

    match storage_request_sender
        .send(StorageRequest::User(action))
        .await
    {
        Ok(_) => (),
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!("failed to send storage request: {}", error),
            );

            return replier.handle_error(error);
        }
    }

    let user = match storage_receiver.await {
        Ok(result) => match result {
            Ok(optional_user) => match optional_user {
                Some(user) => user,
                None => User::new(user_id),
            },
            Err(error) => {
                let error = Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to receive storage result: {}", error),
                );

                return replier.handle_error(error);
            }
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!("failed to receive storage result: {}", error),
            );

            return replier.handle_error(error);
        }
    };

    match replier.send(Ok(user)) {
        Ok(_) => (),
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send logic result",
            ));
        }
    }

    Ok(())
}
