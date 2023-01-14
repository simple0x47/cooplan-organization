pub async fn execute(
    action: OrganizationLogicAction,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<(), Error> {
    match action {
        OrganizationLogicAction::Create {
            name,
            country,
            address,
            telephone,
            replier,
        } => {
            create(
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
    name: String,
    country: String,
    address: String,
    telephone: String,
    storage_request_sender: &Sender<StorageRequest>,
    replier: tokio::sync::oneshot::Sender<Result<Organization, Error>>,
) -> Result<(), Error> {
    if !is_country_code_valid(&country) {
        let error = Error::new(ErrorKind::InvalidCountry, "invalid country code detected");

        return create_handle_error(replier, error);
    }

    if !is_telephone_valid(&telephone) {
        let error = Error::new(ErrorKind::InvalidTelephone, "invalid telephone detected");

        return create_handle_error(replier, error);
    }

    match is_name_already_used(&name, &storage_request_sender).await {
        Ok(is_used) => {
            if is_used {
                let error = Error::new(ErrorKind::NameAlreadyTaken, "name is already being used");

                return create_handle_error(replier, error);
            }
        }
        Err(error) => return create_handle_error(replier, error),
    }

    match is_telephone_being_used(&telephone, &storage_request_sender).await {
        Ok(is_used) => {
            if is_used {
                let error = Error::new(
                    ErrorKind::TelephoneAlreadyInUse,
                    "telephone is already being used",
                );

                return create_handle_error(replier, error);
            }
        }
        Err(error) => return create_handle_error(replier, error),
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

            return create_handle_error(replier, error);
        }
    }

    let organization = match storage_listener.await {
        Ok(result) => match result {
            Ok(organization) => organization,
            Err(error) => return create_handle_error(replier, error),
        },
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!(
                    "failed to receive response for a storage request: {}",
                    error
                ),
            );

            return create_handle_error(replier, error);
        }
    };

    match replier.send(Ok(organization)) {
        Ok(_) => (),
        Err(error) => {
            log::error!("failed to send response to api");

            return Err(Error::new(
                ErrorKind::InternalFailure,
                "failed to send response to api",
            ));
        }
    }

    Ok(())
}

fn create_handle_error(
    replier: tokio::sync::oneshot::Sender<Result<Organization, Error>>,
    error: Error,
) -> Result<(), Error> {
    match replier.send(Err(error.clone())) {
        Ok(_) => (),
        Err(_) => log::error!("failed to reply to api"),
    }

    Err(error)
}

use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_logic_action::OrganizationLogicAction;
use crate::logic::actions::organization_storage_action::OrganizationStorageAction;
use crate::logic::elements::organization::Organization;
use crate::logic::storage_request::StorageRequest;
use crate::logic::validation::country::is_country_code_valid;
use crate::logic::validation::name::is_name_already_used;
use crate::logic::validation::telephone::{is_telephone_being_used, is_telephone_valid};
use async_channel::Sender;
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
