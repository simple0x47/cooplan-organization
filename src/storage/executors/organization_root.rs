use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_root_storage_action::OrganizationRootStorageAction;
use crate::logic::elements::organization_root::OrganizationRoot;
use crate::storage::elements::invitation::Invitation;
use crate::storage::elements::organization::Organization;
use crate::storage::elements::user::User;
use crate::{logic, storage};
use cooplan_util::error_handler::ErrorHandler;
use futures::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Client;

pub async fn execute(action: OrganizationRootStorageAction, client: &Client) -> Result<(), Error> {
    match action {
        OrganizationRootStorageAction::Read {
            organization_id,
            replier,
        } => {
            read(organization_id, replier, client).await?;
        }
    }
    Ok(())
}

async fn read(
    organization_id: String,
    replier: tokio::sync::oneshot::Sender<Result<OrganizationRoot, Error>>,
    client: &Client,
) -> Result<(), Error> {
    let organization_id = match ObjectId::parse_str(&organization_id) {
        Ok(organization_id) => organization_id,
        Err(error) => {
            let error = Error::new(
                ErrorKind::InvalidArgument,
                format!("failed to parse organization id as ObjectId: {}", error),
            );

            return replier.handle_error(error);
        }
    };

    let organization = match get_organization_by_id(&organization_id, client).await {
        Ok(organization) => organization,
        Err(error) => {
            return replier.handle_error(error);
        }
    };

    let users = match get_users_by_organization_id(&organization_id, client).await {
        Ok(users) => users,
        Err(error) => {
            return replier.handle_error(error);
        }
    };

    let invitations = match get_invitations_by_organization_id(&organization_id, client).await {
        Ok(invitations) => invitations,
        Err(error) => {
            return replier.handle_error(error);
        }
    };

    let organization_root = OrganizationRoot {
        organization,
        users,
        invitations,
    };

    match replier.send(Ok(organization_root)) {
        Ok(_) => (),
        Err(_) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send organization root as a reply",
            ));
        }
    }

    Ok(())
}

async fn get_organization_by_id(
    organization_id: &ObjectId,
    client: &Client,
) -> Result<logic::elements::organization::Organization, Error> {
    let organization = match client
        .database(storage::elements::organization::DATABASE)
        .collection::<Organization>(storage::elements::organization::COLLECTION)
        .find_one(
            doc! {
                "_id": organization_id,
            },
            None,
        )
        .await
    {
        Ok(optional_organization) => match optional_organization {
            Some(organization) => organization,
            None => {
                return Err(Error::new(
                    ErrorKind::InvalidArgument,
                    format!("organization with id '{}' not found", organization_id),
                ));
            }
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!("failed to read organization from database: {}", error),
            ));
        }
    };

    Ok(organization.into())
}

async fn get_users_by_organization_id(
    organization_id: &ObjectId,
    client: &Client,
) -> Result<Vec<logic::elements::user::User>, Error> {
    let users: Vec<User> = match client
        .database(storage::elements::user::DATABASE)
        .collection::<User>(storage::elements::user::COLLECTION)
        .find(
            doc! {
                "organizations.organization_id": organization_id.to_string(),
            },
            None,
        )
        .await
    {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(users) => users,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to read users from database: {}", error),
                ));
            }
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!("failed to read users from database: {}", error),
            ));
        }
    };

    Ok(users.into_iter().map(|user| user.into()).collect())
}

async fn get_invitations_by_organization_id(
    organization_id: &ObjectId,
    client: &Client,
) -> Result<Vec<logic::elements::invitation::Invitation>, Error> {
    let invitations: Vec<Invitation> = match client
        .database(storage::elements::invitation::DATABASE)
        .collection::<Invitation>(storage::elements::invitation::COLLECTION)
        .find(
            doc! {
                "organization_id": organization_id,
            },
            None,
        )
        .await
    {
        Ok(cursor) => match cursor.try_collect().await {
            Ok(invitations) => invitations,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to read invitations from database: {}", error),
                ));
            }
        },
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!("failed to read invitations from database: {}", error),
            ));
        }
    };

    Ok(invitations
        .into_iter()
        .map(|invitation| invitation.into())
        .collect())
}
