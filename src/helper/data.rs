use crate::Error;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct User {
    pub user_id: u64,
    pub currency: u64,
    pub experience: u64,
    pub command_count: u64,
    pub lastfm_user: Option<String>,
}

pub async fn fetch_lastfm_username(
    database: &mongodb::Collection<User>,
    user_id: String,
) -> Result<String, Error> {
    let filter = doc! { "user_id": user_id };
    let user = database.find_one(filter, None).await?;

    user.ok_or("No user linked.")?
        .lastfm_user
        .ok_or(Error::from("No user linked."))
}
