use async_channel::Receiver;
use mongodb::Client;

use crate::logic::storage_request::StorageRequest;

pub struct MongoDbRequestDispatch {
    client: Client,
    request_receiver: Receiver<StorageRequest>,
}

impl MongoDbRequestDispatch {
    pub fn new(
        client: Client,
        request_receiver: Receiver<StorageRequest>,
    ) -> MongoDbRequestDispatch {
        MongoDbRequestDispatch {
            client,
            request_receiver,
        }
    }

    pub async fn run(self) {
        loop {
            match self.request_receiver.recv().await {
                Ok(request) => {
                    log::info!("received storage request");

                    let result = match request {
                        StorageRequest::OrganizationRequest(action) => {
                            crate::storage::executors::organization::execute(action, &self.client)
                                .await
                        }
                        StorageRequest::UserRequest(action) => {
                            crate::storage::executors::user::execute(action, &self.client).await
                        },
                        StorageRequest::InvitationRequest(action) => {
                            crate::storage::executors::invitation::execute(action, &self.client).await
                        }
                    };

                    if let Err(error) = result {
                        log::info!("failed to execute storage request: {}", error);
                    }
                }
                Err(error) => {
                    log::error!("failed to receive request: {}", error);
                }
            }
        }
    }
}
