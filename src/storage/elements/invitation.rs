use crate::error::{Error, ErrorKind};
use crate::logic;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::IndexOptions;
use mongodb::{Client, IndexModel};
use serde::Deserialize;

pub const DATABASE: &str = "local";
pub const COLLECTION: &str = "invitation";

#[derive(Debug, Deserialize)]
pub struct Invitation {
    pub _id: ObjectId,
    pub code: String,
    pub organization_id: ObjectId,
    pub permissions: Vec<String>,
    /// Unix timestamp, seconds after the UNIX EPOCH
    pub created_at: u64,
    pub expires_after: u64,
}

impl Into<logic::elements::invitation::Invitation> for Invitation {
    fn into(self) -> logic::elements::invitation::Invitation {
        logic::elements::invitation::Invitation {
            code: self.code,
            organization_id: self.organization_id.to_string(),
            permissions: self.permissions,
            created_at: self.created_at,
            expires_after: self.expires_after,
        }
    }
}

pub async fn initialize(client: &Client) -> Result<(), Error> {
    let options = IndexOptions::builder().unique(true).build();

    let index = IndexModel::builder()
        .keys(doc! { "code": "text" })
        .options(Some(options))
        .build();

    match client
        .database(DATABASE)
        .collection::<Invitation>(COLLECTION)
        .create_index(index, None)
        .await
    {
        Ok(_) => (),
        Err(error) => return Err(Error::new(ErrorKind::InternalFailure, error.to_string())),
    }

    Ok(())
}
