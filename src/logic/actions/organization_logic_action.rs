use crate::error::Error;
use crate::logic::elements::organization::Organization;
use tokio::sync::oneshot::Sender;

pub enum OrganizationLogicAction {
    Create {
        user_id: String,
        name: String,
        country: String,
        address: String,
        telephone: String,
        replier: Sender<Result<Organization, Error>>,
    },
    Join {
        user_id: String,
        invitation_code: String,
        replier: Sender<Result<Organization, Error>>,
    },
}
