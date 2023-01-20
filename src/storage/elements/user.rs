use crate::error::{Error, ErrorKind};
use crate::logic;
use crate::logic::elements::user_organization::UserOrganization;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::IndexOptions;
use mongodb::{Client, IndexModel};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

pub const DATABASE: &str = "local";
pub const COLLECTION: &str = "user";

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub _id: ObjectId,
    pub id: String,
    pub organizations: Vec<UserOrganization>,
}

impl Into<logic::elements::user::User> for User {
    fn into(self) -> logic::elements::user::User {
        logic::elements::user::User {
            id: self.id,
            organizations: self.organizations,
        }
    }
}

pub async fn initialize(client: &Client) -> Result<(), Error> {
    let options = IndexOptions::builder().unique(true).build();

    let index = IndexModel::builder()
        .keys(doc! { "id": 1u32 })
        .options(Some(options.clone()))
        .build();

    match client
        .database(DATABASE)
        .collection::<User>(COLLECTION)
        .create_index(index, None)
        .await
    {
        Ok(_) => (),
        Err(error) => return Err(Error::new(ErrorKind::InternalFailure, error.to_string())),
    }

    Ok(())
}
