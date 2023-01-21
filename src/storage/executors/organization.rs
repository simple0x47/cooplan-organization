use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::storage::elements::organization;
use crate::storage::elements::organization::Organization;
use crate::{logic, storage};
use cooplan_util::error_handler::ErrorHandler;
use mongodb::bson::oid::ObjectId;
use mongodb::bson::{doc, Bson};
use mongodb::Client;
use tokio::sync::oneshot::Sender;

pub async fn execute(action: OrganizationStorageAction, client: &Client) -> Result<(), Error> {
    match action {
        OrganizationStorageAction::Create {
            name,
            country,
            address,
            telephone,
            replier,
        } => create(name, country, address, telephone, replier, client).await?,
        OrganizationStorageAction::Delete { id, replier } => delete(id, replier, client).await?,
        OrganizationStorageAction::FindById { id, replier } => {
            find_by_id(id, replier, client).await?
        }
        OrganizationStorageAction::FindByName { name, replier } => {
            find_by_key_and_value("name", &name, replier, client).await?
        }
        OrganizationStorageAction::FindByTelephone { telephone, replier } => {
            find_by_key_and_value("telephone", &telephone, replier, client).await?
        }
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                format!("unhandled action detected: {:?}", action),
            ));
        }
    }

    Ok(())
}

async fn create(
    name: String,
    country: String,
    address: String,
    telephone: String,
    replier: Sender<Result<logic::elements::organization::Organization, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let organization_id = match client
        .database(storage::elements::organization::DATABASE)
        .collection(storage::elements::organization::COLLECTION)
        .insert_one(
            doc! {
                "name": &name,
                "country": &country,
                "address": &address,
                "telephone": &telephone,
                "permissions": []
            },
            None,
        )
        .await
    {
        Ok(result) => match result.inserted_id.as_object_id() {
            Some(organization_id) => organization_id.to_string(),
            None => {
                return match client
                    .database(storage::elements::organization::DATABASE)
                    .collection::<Organization>(storage::elements::organization::COLLECTION)
                    .delete_one(
                        doc! {
                            "name": &name,
                            "country": &country,
                            "address": &address,
                            "telephone": &telephone,
                        },
                        None,
                    )
                    .await
                {
                    Ok(_) => {
                        let error = Error::new(
                            ErrorKind::ProcessReversion,
                            "failed to get organization id, process reverted",
                        );

                        create_handle_error(replier, error)
                    }
                    Err(error) => {
                        let error = Error::new(
                            ErrorKind::InternalFailure,
                            format!("failed to revert create organization: {}", error),
                        );

                        create_handle_error(replier, error)
                    }
                }
            }
        },
        Err(error) => {
            return create_handle_error(
                replier,
                Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to insert organization: {}", error),
                ),
            )
        }
    };

    let organization = logic::elements::organization::Organization {
        id: organization_id,
        name,
        country,
        address,
        telephone,
        permissions: Vec::new(),
    };

    match replier.send(Ok(organization)) {
        Ok(_) => (),
        Err(_) => {
            log::error!("failed to send response to logic");

            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to logic",
            ));
        }
    }

    Ok(())
}

fn create_handle_error(
    replier: Sender<Result<logic::elements::organization::Organization, Error>>,
    error: Error,
) -> Result<(), Error> {
    match replier.send(Err(error.clone())) {
        Ok(_) => (),
        Err(_) => log::error!("failed to reply to logic"),
    }

    Err(error)
}

async fn delete(
    id: String,
    replier: Sender<Result<(), Error>>,
    client: &Client,
) -> Result<(), Error> {
    let organization_id = match ObjectId::parse_str(&id) {
        Ok(organization_id) => organization_id,
        Err(error) => {
            return delete_handle_error(
                replier,
                Error::new(
                    ErrorKind::InvalidArgument,
                    format!("failed to parse organization id: {}", error),
                ),
            )
        }
    };

    match client
        .database(storage::elements::organization::DATABASE)
        .collection::<Organization>(storage::elements::organization::COLLECTION)
        .delete_one(
            doc! {
                "_id": organization_id,
            },
            None,
        )
        .await
    {
        Ok(_) => (),
        Err(error) => {
            return delete_handle_error(
                replier,
                Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to delete organization: {}", error),
                ),
            )
        }
    }

    match replier.send(Ok(())) {
        Ok(_) => (),
        Err(_) => {
            log::error!("failed to send response to logic");

            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to logic",
            ));
        }
    }

    Ok(())
}

fn delete_handle_error(replier: Sender<Result<(), Error>>, error: Error) -> Result<(), Error> {
    match replier.send(Err(error.clone())) {
        Ok(_) => (),
        Err(_) => log::error!("failed to reply to logic"),
    }

    Err(error)
}

async fn find_by_id(
    id: String,
    replier: Sender<Result<Option<logic::elements::organization::Organization>, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let id = match ObjectId::parse_str(&id) {
        Ok(id) => id,
        Err(error) => {
            return replier.handle_error(Error::new(
                ErrorKind::InvalidArgument,
                format!("failed to parse organization id: {}", error),
            ))
        }
    };

    find_by_key_and_value("_id", id, replier, client).await
}

async fn find_by_key_and_value<ValueType: Into<Bson>>(
    key: &str,
    value: ValueType,
    replier: Sender<Result<Option<logic::elements::organization::Organization>, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let result: Option<logic::elements::organization::Organization> = match client
        .database(storage::elements::organization::DATABASE)
        .collection::<Organization>(storage::elements::organization::COLLECTION)
        .find_one(doc! { key: value.into() }, None)
        .await
    {
        Ok(result) => result.map(|organization| organization.into()),
        Err(error) => {
            return find_by_key_and_value_handle_error(
                replier,
                Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to find organization: {}", error),
                ),
            )
        }
    };

    match replier.send(Ok(result)) {
        Ok(_) => (),
        Err(_) => {
            log::error!("failed to send response to logic");

            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to logic",
            ));
        }
    }

    Ok(())
}

fn find_by_key_and_value_handle_error(
    replier: Sender<Result<Option<logic::elements::organization::Organization>, Error>>,
    error: Error,
) -> Result<(), Error> {
    match replier.send(Err(error.clone())) {
        Ok(_) => (),
        Err(_) => log::error!("failed to reply to logic"),
    }

    Err(error)
}

#[cfg(test)]
async fn setup() -> Client {
    let uri = match std::env::var("MONGODB_URI") {
        Ok(uri) => uri,
        Err(_) => {
            panic!("MONGODB_URI environment variable not set");
        }
    };

    let client = Client::with_uri_str(uri).await.unwrap();

    organization::initialize(&client).await.unwrap();

    client
}

#[tokio::test]
#[ignore]
async fn create_organization_successfully() {
    let client = setup().await;
    let (replier, receiver) = tokio::sync::oneshot::channel();

    let result = create(
        "test".to_string(),
        "test".to_string(),
        "test".to_string(),
        "+40753313640".to_string(),
        replier,
        &client,
    )
    .await;

    // Avoid test failures if repeated on the same MongoDB instance.
    client
        .database(storage::elements::organization::DATABASE)
        .collection::<Organization>(storage::elements::organization::COLLECTION)
        .drop(None)
        .await
        .unwrap();

    assert!(result.is_ok());
}

#[tokio::test]
#[ignore]
async fn create_and_delete_organization_successfully() {
    let client = setup().await;
    let (replier, receiver) = tokio::sync::oneshot::channel();

    let cloned_client = client.clone();
    tokio::spawn(async move {
        create(
            "test".to_string(),
            "test".to_string(),
            "test".to_string(),
            "+40753313640".to_string(),
            replier,
            &cloned_client,
        )
        .await
        .unwrap();
    });

    let organization = receiver.await.unwrap().unwrap();
    let (replier, receiver) = tokio::sync::oneshot::channel();

    delete(organization.id, replier, &client).await;
    let delete_result = receiver.await.unwrap();

    assert!(delete_result.is_ok());
}
