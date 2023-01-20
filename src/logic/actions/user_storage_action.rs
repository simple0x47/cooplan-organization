use crate::error::Error;
use crate::logic::elements::user::User;
use crate::logic::elements::user_organization::UserOrganization;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum UserStorageAction {
    Create {
        id: String,
        organization: UserOrganization,
        replier: Sender<Result<User, Error>>,
    },
    Delete {
        id: String,
        replier: Sender<Result<(), Error>>,
    },
    FindUserById {
        user_id: String,
        replier: Sender<Result<Option<User>, Error>>,
    },
}
