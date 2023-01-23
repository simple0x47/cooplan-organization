use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_root_logic_action::OrganizationRootLogicAction;
use crate::logic::actions::organization_root_storage_action::OrganizationRootStorageAction;
use crate::logic::elements::organization_root::OrganizationRoot;
use crate::logic::storage_request::StorageRequest;
use async_channel::Sender;
use cooplan_util::error_handler::ErrorHandler;

pub async fn execute(
    action: OrganizationRootLogicAction,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<(), Error> {
    match action {
        OrganizationRootLogicAction::Read {
            organization_id,
            replier,
        } => {
            read(organization_id, replier, storage_request_sender).await?;
        }
    }
    Ok(())
}

async fn read(
    organization_id: String,
    replier: tokio::sync::oneshot::Sender<Result<OrganizationRoot, Error>>,
    storage_request_sender: &Sender<StorageRequest>,
) -> Result<(), Error> {
    let (storage_replier, storage_receiver) = tokio::sync::oneshot::channel();

    match storage_request_sender
        .send(StorageRequest::OrganizationRoot(
            OrganizationRootStorageAction::Read {
                organization_id,
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

    let result = match storage_receiver.await {
        Ok(result) => result,
        Err(error) => {
            let error = Error::new(
                ErrorKind::InternalFailure,
                format!("failed to receive storage result: {}", error),
            );

            return replier.handle_error(error);
        }
    };

    match replier.send(result) {
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
