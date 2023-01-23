use crate::error::Error;
use crate::logic::elements::organization_root::OrganizationRoot;
use tokio::sync::oneshot::Sender;

pub enum OrganizationRootLogicAction {
    Read {
        organization_id: String,
        replier: Sender<Result<OrganizationRoot, Error>>,
    },
}
