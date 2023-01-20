use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_logic_action::OrganizationLogicAction;
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::logic::elements::organization::Organization;
use crate::logic::storage_request::StorageRequest;
use crate::logic::validation::country::is_country_code_valid;
use crate::logic::validation::name::is_name_already_used;
use crate::logic::validation::telephone::{is_telephone_being_used, is_telephone_valid};
use crate::logic::validation::user::can_user_create_organization;
use async_channel::{Receiver, Sender};
use cooplan_util::error_handler::ErrorHandler;

pub async fn execute(
    action: OrganizationLogicAction,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<(), Error> {
    match action {
        OrganizationLogicAction::Create {
            user_id,
            name,
            country,
            address,
            telephone,
            replier,
        } => {
            create(
                user_id,
                name,
                country,
                address,
                telephone,
                storage_request_sender,
                replier,
            )
            .await
        }
    }
}

async fn create(
    user_id: String,
    name: String,
    country: String,
    address: String,
    telephone: String,
    storage_request_sender: &Sender<StorageRequest>,
    replier: tokio::sync::oneshot::Sender<Result<Organization, Error>>,
) -> Result<(), Error> {
    if !is_country_code_valid(&country) {
        let error = Error::new(ErrorKind::InvalidCountry, "invalid country code detected");

        return replier.handle_error(error);
    }

    if !is_telephone_valid(&telephone) {
        let error = Error::new(ErrorKind::InvalidTelephone, "invalid telephone detected");

        return replier.handle_error(error);
    }

    match can_user_create_organization(&user_id, storage_request_sender).await {
        Ok(can_create) => {
            if !can_create {
                let error = Error::new(
                    ErrorKind::UserCannotCreateOrganization,
                    "user cannot create an organization",
                );

                return replier.handle_error(error);
            }
        }
        Err(error) => return replier.handle_error(error),
    }

    match is_name_already_used(&name, storage_request_sender).await {
        Ok(is_used) => {
            if is_used {
                let error = Error::new(ErrorKind::NameAlreadyTaken, "name is already being used");

                return replier.handle_error(error);
            }
        }
        Err(error) => return replier.handle_error(error),
    }

    match is_telephone_being_used(&telephone, storage_request_sender).await {
        Ok(is_used) => {
            if is_used {
                let error = Error::new(
                    ErrorKind::TelephoneAlreadyInUse,
                    "telephone is already being used",
                );

                return replier.handle_error(error);
            }
        }
        Err(error) => return replier.handle_error(error),
    }

    let (storage_replier, storage_listener) = tokio::sync::oneshot::channel();

    match storage_request_sender
        .send(StorageRequest::OrganizationRequest(
            OrganizationStorageAction::Create {
                name,
                country,
                address,
                telephone,
                replier: storage_replier,
            },
        ))
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

    let organization = match storage_listener.await {
        Ok(result) => match result {
            Ok(organization) => organization,
            Err(error) => return replier.handle_error(error),
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!(
                    "failed to receive response for a storage request: {}",
                    error
                ),
            );

            return replier.handle_error(error);
        }
    };

    let user_organization = UserOrganization {
        organization_id: organization.id.clone(),
        permissions: logic::permission::organization_creator_permissions(),
    };

    let (storage_replier, storage_listener) = tokio::sync::oneshot::channel();

    let create_user_request = StorageRequest::UserRequest(UserStorageAction::Create {
        id: user_id,
        organization: user_organization,
        replier: storage_replier,
    });

    match storage_request_sender.send(create_user_request).await {
        Ok(_) => (),
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!("failed to send storage request: {}", error),
            );

            restore_create_organization(organization.id, storage_request_sender).await;

            return replier.handle_error(error);
        }
    }

    let user = match storage_listener.await {
        Ok(result) => match result {
            Ok(user) => user,
            Err(error) => {
                restore_create_organization(organization.id, storage_request_sender).await;

                return replier.handle_error(error);
            }
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!(
                    "failed to receive response for a storage request: {}",
                    error
                ),
            );

            restore_create_organization(organization.id, storage_request_sender).await;

            return replier.handle_error(error);
        }
    };

    match replier.send(Ok(organization)) {
        Ok(_) => (),
        Err(_) => {
            log::error!("failed to send response to api");

            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to api",
            ));
        }
    }

    Ok(())
}

async fn restore_create_organization(
    organization_id: String,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<(), Error> {
    let (storage_replier, storage_listener) = tokio::sync::oneshot::channel();

    match storage_request_sender
        .send(StorageRequest::OrganizationRequest(
            OrganizationStorageAction::Delete {
                id: organization_id,
                replier: storage_replier,
            },
        ))
        .await
    {
        Ok(_) => (),
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!("failed to send storage request: {}", error),
            );

            log::error!("{}", error);
            return Err(error);
        }
    };

    match storage_listener.await {
        Ok(result) => match result {
            Ok(_) => Ok(()),
            Err(error) => {
                log::error!("failed to restore create organization: {}", error);
                Err(error)
            }
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!(
                    "failed to receive response for a storage request: {}",
                    error
                ),
            );

            log::error!("{}", error);
            Err(error)
        }
    }
}

use crate::logic;
use crate::logic::actions::user_storage_action::UserStorageAction;
use crate::logic::elements::user_organization::UserOrganization;
#[cfg(test)]
use phonenumber::country::RO;

#[test]
fn phone_validation_works_on_phone_numbers_with_prefix() {
    let valid_phone_with_prefix = "+40753313640";

    match phonenumber::parse(None, valid_phone_with_prefix) {
        Ok(phone_number) => {
            assert_eq!(
                phonenumber::country::RO,
                phone_number.country().id().unwrap()
            );
            assert_eq!("753313640", phone_number.national().to_string());
        }
        Err(e) => panic!("failed to parse valid phone number: {}", e),
    }

    let valid_phone_with_extended_prefix = "0040753313640";

    match phonenumber::parse(Some(RO), valid_phone_with_extended_prefix) {
        Ok(phone_number) => {
            assert_eq!(
                phonenumber::country::RO,
                phone_number.country().id().unwrap()
            );
            assert_eq!("753313640", phone_number.national().to_string());
        }
        Err(e) => panic!("failed to parse valid phone number: {}", e),
    }
}

async fn setup() -> (
    String,
    String,
    String,
    String,
    String,
    (Sender<StorageRequest>, Receiver<StorageRequest>),
) {
    (
        "USER_ID".to_string(),
        "NAME".to_string(),
        "RO".to_string(),
        "ADDRESS".to_string(),
        "+40753313640".to_string(),
        async_channel::bounded(100),
    )
}

#[tokio::test]
async fn detect_invalid_country() {
    const INVALID_COUNTRY: &str = "XX";

    let (user_id, name, _, address, phone, (storage_request_sender, _)) = setup().await;

    let (replier, listener) = tokio::sync::oneshot::channel();

    let function_result = create(
        user_id,
        name,
        INVALID_COUNTRY.to_string(),
        address,
        phone,
        &storage_request_sender,
        replier,
    )
    .await;

    assert!(function_result.is_err());
    assert_eq!(ErrorKind::InvalidCountry, function_result.unwrap_err().kind);

    let result = listener.await.unwrap();

    assert!(result.is_err());
    assert_eq!(ErrorKind::InvalidCountry, result.unwrap_err().kind);
}

#[tokio::test]
async fn detect_invalid_phone() {
    const INVALID_PHONE: &str = "INVALID_PHONE";

    let (user_id, name, country, address, _, (storage_request_sender, _)) = setup().await;

    let (replier, listener) = tokio::sync::oneshot::channel();

    let function_result = create(
        user_id,
        name,
        country,
        address,
        INVALID_PHONE.to_string(),
        &storage_request_sender,
        replier,
    )
    .await;

    assert!(function_result.is_err());
    assert_eq!(
        ErrorKind::InvalidTelephone,
        function_result.unwrap_err().kind
    );

    let result = listener.await.unwrap();

    assert!(result.is_err());
    assert_eq!(ErrorKind::InvalidTelephone, result.unwrap_err().kind);
}
