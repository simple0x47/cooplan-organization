use crate::error::Error;
use crate::logic::storage_request::StorageRequest;

pub enum ExampleLogicAction {
    Example {},
}

pub async fn execute(
    action: ExampleLogicAction,
    storage_request_sender: &async_channel::Sender<StorageRequest>,
) -> Result<(), Error> {
    match action {
        ExampleLogicAction::Example {} => todo!(),
    }
}
