use crate::error::{Error, ErrorKind};
use crate::logic::actions::user_storage_action::UserStorageAction;
use crate::logic::elements::user::User;
use crate::storage;
use mongodb::bson::doc;
use mongodb::Client;
use tokio::sync::oneshot::Sender;

pub async fn execute(action: UserStorageAction, client: &Client) -> Result<(), Error> {
    match action {
        UserStorageAction::Create { user_id, replier } => todo!(),
        UserStorageAction::FindUserById { user_id, replier } => {
            find_user_by_id(user_id, replier, client).await?;
        }
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!("unhandled action detected: {:?}", action,),
            ))
        }
    }

    Ok(())
}

async fn find_user_by_id(
    user_id: String,
    replier: Sender<Result<Option<User>, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let filter = doc! {
        "id": user_id,
    };

    let result = match client
        .database(storage::elements::user::DATABASE)
        .collection::<User>(storage::elements::user::COLLECTION)
        .find_one(Some(filter), None)
        .await
    {
        Ok(result) => Ok(result),
        Err(error) => Err(Error::new(
            ErrorKind::StorageFailure,
            format!("failed to find user by id: {}", error),
        )),
    };

    match replier.send(result) {
        Ok(_) => (),
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to logic",
            ))
        }
    }

    Ok(())
}
