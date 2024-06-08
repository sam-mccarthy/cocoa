use crate::Error;
use mongodb::bson::{doc, Bson};
use mongodb;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub user_id: u64,
    pub currency: u64,
    pub experience: u64,
    pub command_count: u64,
}

pub async fn insert_user(
    database: &mongodb::Collection<User>,
    user_id: u64,
) -> Result<User, Error> {
    let user = User {
        user_id,
        currency: 0,
        experience: 0,
        command_count: 0,
    };

    database.insert_one(&user, None).await?;
    Ok(user)
}

pub async fn find_user(database: &mongodb::Collection<User>, user_id: u64) -> Result<User, Error> {
    let filter = doc! { "_id": user_id as i64 };
    let user = database.find_one(filter, None).await?;

    user.ok_or(Error::from("Couldn't find user."))
}

pub async fn update_user<T: Into<Bson>>(
    database: &mongodb::Collection<User>,
    user_id: u64,
    property_name: &str,
    property_value: T,
) -> Result<(), Error> {
    // We filter by user_id, and directly set the property value.
    let filter = doc! { "_id": user_id as i64 };
    let update = doc! { "$set": doc!{property_name: property_value}};

    database.update_one(filter, update, None).await?;
    Ok(())
}
