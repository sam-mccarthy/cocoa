use crate::Error;
use serde_json::Value;

pub async fn get_from_lastfm(api_key: &str, method: &str, user: &String) -> Result<Value, Error> {
    let user_param_name = match method {
        "user.getInfo" => "user",
        "user.getRecentTracks" => "user",
        "track.getInfo" => "username",
        _ => "user",
    };

    let endpoint = format!(
        "https://ws.audioscrobbler.com/2.0?format=json&api_key={}&method={}&{}={}",
        api_key, method, user_param_name, user
    );

    get_from_endpoint(&endpoint, user).await
}

pub async fn get_from_endpoint(url: &String, user: &String) -> Result<Value, Error> {
    let json = reqwest::get(url).await?.text().await?;
    let parsed: Value = serde_json::from_str(&json)?;

    if parsed["error"].is_null() {
        Ok(parsed)
    } else {
        Err(Error::from("Failed endpoint request."))
    }
}
