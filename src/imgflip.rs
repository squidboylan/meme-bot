use reqwest::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize)]
struct CaptionPost<'a> {
    template_id: &'a str,
    username: &'a str,
    password: &'a str,
    text0: &'a str,
    text1: &'a str,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CaptionResponse {
    pub success: bool,
    #[serde(flatten)]
    pub data: CaptionResponseData,
}

#[derive(Clone, Debug, Deserialize)]
pub enum CaptionResponseData {
    #[serde(rename = "data")]
    Data { url: String, page_url: String },
    #[serde(rename = "error_message")]
    ErrorMessage(String),
}

pub struct ImgflipClient {
    pub username: String,
    pub password: String,
    meme_map: HashMap<&'static str, &'static str>,
}

impl ImgflipClient {
    pub fn new(username: String, password: String) -> Self {
        let meme_map: HashMap<&'static str, &'static str> = [
            ("drake", "181913649"),
            ("always has been", "252600902"),
            ("distracted boyfriend", "112126428"),
            ("spongebob", "102156234"),
            ("sharp turn", "124822590"),
            ("big brain", "93895088"),
            ("aliens", "101470"),
            ("two buttons", "87743020"),
        ]
        .iter()
        .cloned()
        .collect();
        ImgflipClient {
            username,
            password,
            meme_map,
        }
    }

    pub async fn caption_image(
        &self,
        template_name: &str,
        text: &[&str],
    ) -> Result<CaptionResponse> {
        let client = reqwest::Client::new();
        let lowercase_template_name = template_name.to_lowercase();
        let id = self.meme_map.get::<str>(&lowercase_template_name).unwrap();
        let mut params: Vec<(String, &str)> = vec![
            ("template_id".to_string(), *id),
            ("username".to_string(), &self.username),
            ("password".to_string(), &self.password),
        ];
        for (n, l) in text.iter().enumerate() {
            params.push((format!("boxes[{}][text]", n), l));
        }
        client
            .post("https://api.imgflip.com/caption_image")
            .query(&params)
            .send()
            .await?
            .json::<CaptionResponse>()
            .await
    }

    pub fn list_memes<'a>(&'a self) -> Vec<&'a str> {
        self.meme_map.keys().copied().collect()
    }
}
