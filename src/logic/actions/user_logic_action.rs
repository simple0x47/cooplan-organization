use crate::error::Error;
use crate::logic::elements::user::User;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum UserLogicAction {
    Read {
        user_id: String,
        replier: Sender<Result<User, Error>>,
    },
}
