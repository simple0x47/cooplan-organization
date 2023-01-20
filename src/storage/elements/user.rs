use crate::error::{Error, ErrorKind};
use crate::logic::elements::organization::Organization;
use mongodb::bson::doc;
use mongodb::options::IndexOptions;
use mongodb::{Client, IndexModel};

pub const DATABASE: &str = "local";
pub const COLLECTION: &str = "user";

pub async fn initialize(client: &Client) -> Result<(), Error> {
    let options = IndexOptions::builder().unique(true).build();

    let index = IndexModel::builder()
        .keys(doc! { "id": 1u32 })
        .options(Some(options.clone()))
        .build();

    match client
        .database(DATABASE)
        .collection::<Organization>(COLLECTION)
        .create_index(index, None)
        .await
    {
        Ok(_) => (),
        Err(error) => return Err(Error::new(ErrorKind::InternalFailure, error.to_string())),
    }

    Ok(())
}
