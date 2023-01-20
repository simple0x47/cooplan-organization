use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::storage::elements::organization::Organization;
use crate::{logic, storage};
use mongodb::bson::doc;
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
                            format!("failed to revert process: {}", error),
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

async fn find_by_key_and_value(
    key: &str,
    value: &str,
    replier: Sender<Result<Option<logic::elements::organization::Organization>, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let result: Option<logic::elements::organization::Organization> = match client
        .database(storage::elements::organization::DATABASE)
        .collection(storage::elements::organization::COLLECTION)
        .find_one(doc! { key: value }, None)
        .await
    {
        Ok(result) => result,
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
#[tokio::test]
#[ignore]
async fn create_organization_successfully() {
    let uri = match std::env::var("MONGODB_URI") {
        Ok(uri) => uri,
        Err(_) => {
            panic!("MONGODB_URI environment variable not set");
        }
    };

    let client = Client::with_uri_str(uri).await.unwrap();
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
