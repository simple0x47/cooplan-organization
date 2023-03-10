use crate::error::Error;
use crate::logic::elements::organization::Organization;
use tokio::sync::oneshot::Sender;

#[derive(Debug)]
pub enum OrganizationStorageAction {
    Create {
        name: String,
        country: String,
        address: String,
        telephone: String,
        replier: Sender<Result<Organization, Error>>,
    },
    Delete {
        id: String,
        replier: Sender<Result<(), Error>>,
    },
    FindById {
        id: String,
        replier: Sender<Result<Option<Organization>, Error>>,
    },
    FindByName {
        name: String,
        replier: Sender<Result<Option<Organization>, Error>>,
    },
    FindByTelephone {
        telephone: String,
        replier: Sender<Result<Option<Organization>, Error>>,
    },
}
