use crate::error::Error;
use crate::logic::elements::user::User;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum UserStorageAction {
    Create {
        user_id: String,
        replier: Sender<Result<User, Error>>,
    },
    FindUserById {
        user_id: String,
        replier: Sender<Result<Option<User>, Error>>,
    },
}
