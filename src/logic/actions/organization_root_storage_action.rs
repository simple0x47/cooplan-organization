use crate::error::Error;
use crate::logic::elements::organization_root::OrganizationRoot;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum OrganizationRootStorageAction {
    Read {
        organization_id: String,
        replier: Sender<Result<OrganizationRoot, Error>>,
    },
}
