use log::debug;
use mongodb;
use mongodb::bson::{doc, Document};
use serde::{Deserialize, Serialize};

use crate::Error;

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
    debug!("Inserting new user {}", user_id);
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
    debug!("Finding user {}", user_id);
    let filter = doc! { "_id": user_id as i64 };
    let user = database.find_one(filter, None).await?;

    user.ok_or(Error::from("Couldn't find user."))
}

pub async fn update_user_set(
    database: &mongodb::Collection<User>,
    user_id: u64,
    change_doc: Document,
) -> Result<(), Error> {
    debug!("Setting user {}'s fields: {:?}", user_id, change_doc);
    // We filter by user_id, and directly set the property value.
    let filter = doc! { "_id": user_id as i64 };
    let update = doc! { "$set": change_doc };

    database.update_one(filter, update, None).await?;
    Ok(())
}

pub async fn update_user_inc(
    database: &mongodb::Collection<User>,
    user_id: u64,
    change_doc: Document,
) -> Result<(), Error> {
    debug!("Incrementing user {}'s fields: {:?}", user_id, change_doc);
    // We filter by user_id, and directly set the property value.
    let filter = doc! { "_id": user_id as i64 };
    let update = doc! { "$inc": change_doc };

    database.update_one(filter, update, None).await?;
    Ok(())
}
