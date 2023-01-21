use crate::error::Error;
use crate::logic::elements::invitation::Invitation;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum InvitationStorageAction {
    Delete {
        code: String,
        replier: Sender<Result<(), Error>>,
    },
    FindByCode {
        code: String,
        replier: Sender<Result<Option<Invitation>, Error>>,
    },
}
