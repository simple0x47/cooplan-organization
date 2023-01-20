use crate::error::{Error, ErrorKind};
use crate::logic::actions::user_storage_action::UserStorageAction;
use crate::logic::elements::user_organization::UserOrganization;
use crate::storage::elements::user::User;
use crate::{logic, storage};
use cooplan_util::error_handler::ErrorHandler;
use mongodb::bson::{doc, Bson};
use mongodb::Client;
use tokio::sync::oneshot::Sender;

pub async fn execute(action: UserStorageAction, client: &Client) -> Result<(), Error> {
    match action {
        UserStorageAction::Create {
            id,
            organization,
            replier,
        } => create(id, organization, replier, client).await?,
        UserStorageAction::FindUserById { user_id, replier } => {
            find_user_by_id(user_id, replier, client).await?;
        }
    }

    Ok(())
}

async fn create(
    id: String,
    organization: UserOrganization,
    replier: Sender<Result<logic::elements::user::User, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let bson_organization: Bson = organization.clone().into();
    match client
        .database(user::DATABASE)
        .collection(user::COLLECTION)
        .insert_one(
            doc! {
                "id": &id,
                "organizations": [
                    bson_organization
                ],
            },
            None,
        )
        .await
    {
        Ok(result) => match result.inserted_id.as_object_id() {
            Some(_) => (),
            None => {
                return match client
                    .database(user::DATABASE)
                    .collection::<User>(user::COLLECTION)
                    .delete_one(doc! {"id": &id}, None)
                    .await
                {
                    Ok(_) => {
                        let error = Error::new(
                            ErrorKind::ProcessReversion,
                            "failed to get user id, process reverted",
                        );

                        replier.handle_error(error)
                    }
                    Err(error) => {
                        let error = Error::new(
                            ErrorKind::InternalFailure,
                            format!("failed to revert create user: {}", error),
                        );

                        replier.handle_error(error)
                    }
                }
            }
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::InvalidArgument,
                format!("failed to create user: {}", error),
            );

            return replier.handle_error(error);
        }
    };

    let user = logic::elements::user::User {
        id,
        organizations: vec![organization],
    };

    match replier.send(Ok(user)) {
        Ok(_) => (),
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to logic",
            ))
        }
    };

    Ok(())
}

async fn find_user_by_id(
    user_id: String,
    replier: Sender<Result<Option<logic::elements::user::User>, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let filter = doc! {
        "id": user_id,
    };

    let result = match client
        .database(user::DATABASE)
        .collection::<User>(user::COLLECTION)
        .find_one(Some(filter), None)
        .await
    {
        Ok(result) => result.map(|user| user.into()),
        Err(error) => {
            return replier.handle_error(Error::new(
                ErrorKind::StorageFailure,
                format!("failed to find user by id: {}", error),
            ));
        }
    };

    match replier.send(Ok(result)) {
        Ok(_) => (),
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to logic",
            ))
        }
    }

    Ok(())
}

async fn delete(
    id: String,
    replier: Sender<Result<(), Error>>,
    client: &Client,
) -> Result<(), Error> {
    let filter = doc! {
        "id": id,
    };

    match client
        .database(user::DATABASE)
        .collection::<User>(user::COLLECTION)
        .delete_one(filter, None)
        .await
    {
        Ok(_) => (),
        Err(error) => {
            return replier.handle_error(Error::new(
                ErrorKind::StorageFailure,
                format!("failed to delete user: {}", error),
            ));
        }
    }

    match replier.send(Ok(())) {
        Ok(_) => (),
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to logic",
            ))
        }
    }

    Ok(())
}

#[cfg(test)]
use crate::logic::permission::organization_creator_permissions;
use crate::storage::elements::user;

async fn setup() -> Client {
    let uri = match std::env::var("MONGODB_URI") {
        Ok(uri) => uri,
        Err(_) => {
            panic!("MONGODB_URI environment variable not set");
        }
    };

    let client = Client::with_uri_str(uri).await.unwrap();

    user::initialize(&client).await.unwrap();

    client
}

#[tokio::test]
#[ignore]
async fn correctly_create_new_user_and_delete() {
    const TEST_ID: &str = "test|1234851928";
    const TEST_ORGANIZATION_ID: &str = "org|123910921";

    let client = setup().await;
    let user_organization: UserOrganization = UserOrganization {
        organization_id: TEST_ORGANIZATION_ID.to_string(),
        permissions: organization_creator_permissions(),
    };

    let (replier, receiver) =
        tokio::sync::oneshot::channel::<Result<logic::elements::user::User, Error>>();

    create(TEST_ID.to_string(), user_organization, replier, &client).await;
    let create_result = receiver.await.unwrap();

    let (replier, receiver) = tokio::sync::oneshot::channel::<Result<(), Error>>();
    delete(TEST_ID.to_string(), replier, &client).await.unwrap();
    let delete_result = receiver.await.unwrap();

    assert!(create_result.is_ok());
    assert!(delete_result.is_ok());
}
