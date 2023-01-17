use crate::error::{Error, ErrorKind};
use crate::logic::actions::organization_logic_action::OrganizationLogicAction;
use crate::logic::storage_request::StorageRequest;
use crate::logic::validation::country::is_country_code_valid;
use crate::logic::validation::telephone::is_telephone_valid;
use async_channel::Sender;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Organization {
    pub id: String,
    pub name: String,
    pub country: String,
    pub address: String,
    pub telephone: String,
}
