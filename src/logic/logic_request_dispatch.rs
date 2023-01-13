use async_channel::{Receiver, Sender};

use crate::error::ErrorKind;
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

                    let result = match request {
                        LogicRequest::ExampleRequest(action) => todo!(),
                        _ => {}
                    };

                    todo!()
                }
                Err(error) => {
                    // TODO: Block requests until the channel has been reinitialized.
                    log::error!("failed to receive request: {}", error);
                }
            }
        }
    }
}
