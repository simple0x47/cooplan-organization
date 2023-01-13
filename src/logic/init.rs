use async_channel::{Receiver, Sender};

use crate::error::Error;
use crate::logic::logic_request::LogicRequest;
use crate::logic::logic_request_dispatch::LogicRequestDispatch;
use crate::logic::storage_request::StorageRequest;

pub async fn initialize(
    concurrent_dispatchers: u16,
    logic_request_receiver: Receiver<LogicRequest>,
    storage_request_sender: Sender<StorageRequest>,
) -> Result<(), Error> {
    for _ in 0..concurrent_dispatchers {
        let logic_request_dispatch = LogicRequestDispatch::new(
            logic_request_receiver.clone(),
            storage_request_sender.clone(),
        );

        tokio::spawn(logic_request_dispatch.run());
    }

    Ok(())
}
