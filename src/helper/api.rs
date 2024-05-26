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

    get_from_endpoint(&endpoint).await
}

pub async fn get_from_endpoint(url: &String) -> Result<Value, Error> {
    let req = reqwest::get(url).await?;
    if req.status() == 200 {
        let text = &req.text().await?;
        Ok(serde_json::from_str(&text)?)
    } else {
        Err(Error::from("Failed endpoint request."))
    }
}
