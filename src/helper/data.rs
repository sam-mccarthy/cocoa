use crate::Error;

pub async fn fetch_lastfm_username(
    database: &sqlx::SqlitePool,
    user_id: String,
) -> Result<String, Error> {
    sqlx::query!("SELECT username FROM lastfm WHERE user_id = ?", user_id)
        .fetch_one(database)
        .await?
        .username
        .ok_or(Error::from("User not linked."))
}
