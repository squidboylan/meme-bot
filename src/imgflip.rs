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
    pub data: CaptionResponseData,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CaptionResponseData {
    pub url: String,
    pub page_url: String,
}

pub struct ImgflipClient {
    pub username: String,
    pub password: String,
    meme_id_map: HashMap<&'static str, &'static str>,
}

impl ImgflipClient {
    pub fn new(username: String, password: String) -> Self {
        let mut meme_id_map: HashMap<&'static str, &'static str> =
            [("drake", "181913649"), ("always has been", "252600902")]
                .iter()
                .cloned()
                .collect();
        ImgflipClient {
            username,
            password,
            meme_id_map,
        }
    }
    pub async fn caption_image(
        &self,
        template_id: &str,
        text0: &str,
        text1: &str,
    ) -> reqwest::Result<CaptionResponse> {
        let client = reqwest::Client::new();
        let res = client
            .post("https://api.imgflip.com/caption_image")
            .query(&[
                ("template_id", template_id),
                ("username", &self.username),
                ("password", &self.password),
                ("text0", text0),
                ("text1", text1),
            ])
            .send()
            .await?;
        res.json::<CaptionResponse>().await
    }

    pub fn get_meme_id(&self, name: &str) -> Option<&str> {
        self.meme_id_map.get(name).map(|x| *x)
    }

    pub fn list_memes<'a>(&'a self) -> Vec<&'a str> {
        // why does this work??????
        self.meme_id_map.keys().map(|x| x.as_ref()).collect()
    }
}
