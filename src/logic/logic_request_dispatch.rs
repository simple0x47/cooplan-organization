use async_channel::{Receiver, Sender};

use crate::error::ErrorKind;
use crate::logic::executors::{organization, organization_root, user};
use crate::logic::logic_request::LogicRequest;
use crate::logic::storage_request::StorageRequest;

pub struct LogicRequestDispatch {
    request_receiver: Receiver<LogicRequest>,
    storage_request_sender: Sender<StorageRequest>,
}

impl LogicRequestDispatch {
    pub fn new(
        request_receiver: Receiver<LogicRequest>,
        storage_request_sender: Sender<StorageRequest>,
    ) -> LogicRequestDispatch {
        LogicRequestDispatch {
            request_receiver,
            storage_request_sender,
        }
    }

    pub async fn run(self) {
        loop {
            match self.request_receiver.recv().await {
                Ok(request) => {
                    log::info!("received logic request");

                    match request {
                        LogicRequest::Organization(organization_action) => {
                            match organization::execute(
                                organization_action,
                                &self.storage_request_sender,
                            )
                            .await
                            {
                                Ok(_) => (),
                                Err(error) => {
                                    log::info!("failed to execute organization action: {}", error);
                                    continue;
                                }
                            }
                        }
                        LogicRequest::OrganizationRoot(organization_root_action) => {
                            match organization_root::execute(
                                organization_root_action,
                                &self.storage_request_sender,
                            )
                            .await
                            {
                                Ok(_) => (),
                                Err(error) => {
                                    log::info!(
                                        "failed to execute organization root action: {}",
                                        error
                                    );
                                    continue;
                                }
                            }
                        }
                        LogicRequest::User(user_action) => {
                            match user::execute(user_action, &self.storage_request_sender).await {
                                Ok(_) => (),
                                Err(error) => {
                                    log::info!("failed to execute user action: {}", error);
                                    continue;
                                }
                            }
                        }
                    };
                }
                Err(error) => {
                    // TODO: Block requests until the channel has been reinitialized.
                    log::error!("failed to receive request: {}", error);
                }
            }
        }
    }
}
