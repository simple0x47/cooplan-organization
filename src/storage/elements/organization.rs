use crate::error::{Error, ErrorKind};
use crate::logic;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::options::IndexOptions;
use mongodb::{Client, IndexModel};
use serde::Deserialize;

// Values are used within the integration tests.
// Therefore, any update must be reflected in the tests.
pub const DATABASE: &str = "local";
pub const COLLECTION: &str = "organization";

#[derive(Debug, Deserialize)]
pub struct Organization {
    pub _id: ObjectId,
    pub name: String,
    pub country: String,
    pub address: String,
    pub telephone: String,
    pub permissions: Vec<String>,
}

impl Into<logic::elements::organization::Organization> for Organization {
    fn into(self) -> logic::elements::organization::Organization {
        logic::elements::organization::Organization {
            id: self._id.to_string(),
            name: self.name,
            country: self.country,
            address: self.address,
            telephone: self.telephone,
            permissions: self.permissions,
        }
    }
}

pub async fn initialize(client: &Client) -> Result<(), Error> {
    let options = IndexOptions::builder().unique(true).build();

    let indexes = vec![
        IndexModel::builder()
            .keys(doc! { "name": "text" })
            .options(Some(options.clone()))
            .build(),
        IndexModel::builder()
            .keys(doc! { "telephone": 1u32 })
            .options(Some(options))
            .build(),
    ];

    match client
        .database(DATABASE)
        .collection::<Organization>(COLLECTION)
        .create_indexes(indexes, None)
        .await
    {
        Ok(_) => (),
        Err(error) => return Err(Error::new(ErrorKind::InternalFailure, error.to_string())),
    }

    Ok(())
}

#[cfg(test)]
#[tokio::test]
#[ignore]
async fn create_index_does_not_return_error_if_indexes_already_exist() {
    let uri = match std::env::var("MONGODB_URI") {
        Ok(uri) => uri,
        Err(_) => {
            panic!("MONGODB_URI environment variable not set");
        }
    };

    let client = Client::with_uri_str(uri).await.unwrap();

    let result = initialize(&client).await;
    let second_result = initialize(&client).await;

    assert!(result.is_ok());
    assert!(second_result.is_ok());
}
