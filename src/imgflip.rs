use reqwest::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::fs;

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
    meme_map: HashMap<String, String>,
    client: reqwest::Client,
}

async fn load_template_map(template_map_file: &str) -> Option<HashMap<String, String>> {
    let contents_result = fs::read_to_string(template_map_file).await;
    if let Ok(contents) = contents_result {
        let tmp_meme_map = serde_json::from_str::<HashMap<String, String>>(&contents);
        if let Ok(meme_map) = tmp_meme_map {
            Some(meme_map)
        } else {
            None
        }
    } else {
        None
    }
}

impl ImgflipClient {
    pub async fn new(username: String, password: String, template_map_file: &str) -> Self {
        let meme_map = load_template_map(&template_map_file)
            .await
            .expect("Failed to load template_map");
        ImgflipClient {
            username,
            password,
            meme_map,
            client: reqwest::Client::new(),
        }
    }

    pub async fn update_template_map(&mut self, template_map_file: &str) {
        let meme_map_o = load_template_map(&template_map_file).await;
        if let Some(meme_map) = meme_map_o {
            self.meme_map = meme_map;
            println!("Loaded new template_map");
        } else {
            println!("Failed to load new template_map, still using the old one");
        }
    }

    pub async fn caption_image(
        &self,
        template_name: &str,
        text: &[&str],
    ) -> Result<CaptionResponse> {
        let lowercase_template_name = template_name.to_lowercase();
        let id = self.meme_map.get::<str>(&lowercase_template_name).unwrap();
        let mut params: Vec<(String, &str)> = vec![
            ("template_id".to_string(), &id),
            ("username".to_string(), &self.username),
            ("password".to_string(), &self.password),
        ];
        for (n, l) in text.iter().enumerate() {
            params.push((format!("boxes[{}][text]", n), l));
        }
        self.client
            .post("https://api.imgflip.com/caption_image")
            .query(&params)
            .send()
            .await?
            .json::<CaptionResponse>()
            .await
    }

    pub fn list_memes<'a>(&'a self) -> Vec<&'a str> {
        self.meme_map.keys().map(|x| x.as_ref()).collect()
    }
}
