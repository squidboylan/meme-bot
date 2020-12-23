use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::From;
use std::error::Error as ErrorTrait;
use std::fmt;
use std::result::Result as stdResult;

type Result<T> = stdResult<T, Error>;

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
    meme_map: HashMap<&'static str, &'static str>,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Imgflip(ImgflipError),
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Reqwest(error)
    }
}

impl From<ImgflipError> for Error {
    fn from(error: ImgflipError) -> Self {
        Error::Imgflip(error)
    }
}

#[derive(Debug)]
pub struct ImgflipError {
    text: String,
}

impl ErrorTrait for ImgflipError {}

impl fmt::Display for ImgflipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ImgflipClient {
    pub fn new(username: String, password: String) -> Self {
        let mut meme_map: HashMap<&'static str, &'static str> = [
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
        let id = self.meme_map.get(template_name).unwrap();
        let mut params: Vec<(String, &str)> = vec![
            ("template_id".to_string(), *id),
            ("username".to_string(), &self.username),
            ("password".to_string(), &self.password),
        ];
        for (n, l) in text.iter().enumerate() {
            params.push((format!("boxes[{}][text]", n), l));
        }
        println!("{:#?}", params);
        let res = client
            .post("https://api.imgflip.com/caption_image")
            .query(&params)
            .send()
            .await?;
        let body = res.text().await?;
        let json_result: serde_json::Result<CaptionResponse> = serde_json::from_str(&body);
        match json_result {
            Ok(x) => Ok(x.into()),
            Err(_) => Err(Error::Imgflip(ImgflipError { text: body })),
        }
    }

    pub fn list_memes<'a>(&'a self) -> Vec<&'a str> {
        // why does this work??????
        self.meme_map.keys().map(|x| x.as_ref()).collect()
    }
}
