use inotify::{Inotify, WatchMask};
use std::env;
use std::sync::Arc;
use tokio::stream::StreamExt;
use tokio::sync::RwLock;

use serenity::prelude::*;

mod handler;
mod imgflip;

pub struct ImgflipClientContainer;

impl TypeMapKey for ImgflipClientContainer {
    type Value = Arc<RwLock<imgflip::ImgflipClient>>;
}

async fn update_template_map(
    template_map_file: String,
    imgflip_client: Arc<RwLock<imgflip::ImgflipClient>>,
) -> ! {
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");

    inotify
        .add_watch(
            template_map_file.clone(),
            WatchMask::CREATE | WatchMask::MODIFY,
        )
        .unwrap();

    let mut buffer = [0; 32];
    let mut stream = inotify.event_stream(&mut buffer).unwrap();
    loop {
        while let Some(event_or_error) = stream.next().await {
            let mut lock = imgflip_client.write().await;
            lock.update_template_map(&template_map_file).await;
            println!("event: {:?}", event_or_error.unwrap());
        }
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Please set DISCORD_TOKEN");
    let username = env::var("IMGFLIP_USERNAME").expect("Please set IMGFLIP_USERNAME");
    let password = env::var("IMGFLIP_PASSWORD").expect("Please set IMGFLIP_PASSWORD");
    let template_map = env::var("TEMPLATE_MAP")
        .or::<std::env::VarError>(Ok("template_map.json".to_string()))
        .unwrap();

    let imgflip_client = Arc::new(RwLock::new(
        imgflip::ImgflipClient::new(username.to_string(), password.to_string(), &template_map)
            .await,
    ));
    tokio::spawn(update_template_map(
        template_map.clone(),
        Arc::clone(&imgflip_client),
    ));

    // Create a new instance of the Client, logging in as a bot. This will
    // automatically prepend your bot token with "Bot ", which is a requirement
    // by Discord for bot users.
    let mut client = Client::builder(&token)
        .event_handler(handler::Handler)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ImgflipClientContainer>(imgflip_client);
    }

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform
    // exponential backoff until it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
