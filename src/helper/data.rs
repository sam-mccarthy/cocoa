use crate::Error;
use mongodb::bson::{doc, Bson};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    #[serde(rename = "_id")]
    pub user_id: u64,
    pub currency: u64,
    pub experience: u64,
    pub command_count: u64,
    pub lastfm_user: Option<String>,
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
        lastfm_user: None,
    };

    database.insert_one(&user, None).await?;
    Ok(user)
}

pub async fn find_user(database: &mongodb::Collection<User>, user_id: u64) -> Result<User, Error> {
    let filter = doc! { "_id": user_id as i64 };
    let user = database.find_one(filter, None).await?;

    match user {
        Some(user) => Ok(user),
        None => Ok(insert_user(database, user_id).await?),
    }
}

pub async fn update_user<T: Into<Bson>>(
    database: &mongodb::Collection<User>,
    user_id: u64,
    property_name: &str,
    property_value: T,
) -> Result<(), Error> {
    let filter = doc! { "_id": user_id as i64 };
    let update = doc! { "$set": doc!{property_name: property_value}};

    if let Err(_) = find_user(database, user_id).await {
        insert_user(database, user_id).await?;
    }

    database.update_one(filter, update, None).await?;
    Ok(())
}
