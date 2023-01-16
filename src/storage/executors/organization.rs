use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::logic::elements::organization::Organization;
use mongodb::bson::doc;
use mongodb::Client;
use tokio::sync::oneshot::Sender;

const DATABASE: &str = "local";
const COLLECTION: &str = "organization";

pub async fn execute(action: OrganizationStorageAction, client: &Client) -> Result<(), Error> {
    match action {
        OrganizationStorageAction::Create {
            name,
            country,
            address,
            telephone,
            replier,
        } => create(name, country, address, telephone, replier, client).await?,
        _ => todo!(),
    }

    Ok(())
}

async fn create(
    name: String,
    country: String,
    address: String,
    telephone: String,
    replier: Sender<Result<Organization, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let organization_id = match client
        .database(DATABASE)
        .collection(COLLECTION)
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
                let error = Error::new(ErrorKind::InternalFailure, "failed to get organization id");

                return create_handle_error(replier, error);
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

    let organization = Organization {
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
    replier: Sender<Result<Organization, Error>>,
    error: Error,
) -> Result<(), Error> {
    match replier.send(Err(error.clone())) {
        Ok(_) => (),
        Err(_) => log::error!("failed to reply to logic"),
    }

    Err(error)
}
