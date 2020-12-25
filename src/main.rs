use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use serenity::prelude::*;

mod handler;
mod imgflip;

pub struct ImgflipClientContainer;

impl TypeMapKey for ImgflipClientContainer {
    type Value = Arc<RwLock<imgflip::ImgflipClient>>;
}
#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    let username = env::var("IMGFLIP_USERNAME").expect("Expected a token in the environment");
    let password = env::var("IMGFLIP_PASSWORD").expect("Expected a token in the environment");

    let imgflip_client = Arc::new(RwLock::new(imgflip::ImgflipClient::new(
        username.to_string(),
        password.to_string(),
    )));

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
