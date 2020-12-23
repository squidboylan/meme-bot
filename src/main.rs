use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

mod imgflip;

pub struct ImgflipClientContainer;

impl TypeMapKey for ImgflipClientContainer {
    type Value = Arc<RwLock<imgflip::ImgflipClient>>;
}

struct Handler;

async fn send_msg(ctx: &Context, msg: Message, data: &str) {
    if let Err(why) = msg.channel_id.say(&ctx.http, data).await {
        println!("Error sending message: {:?}", why);
    }
}

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        println!("{}", msg.content);
        if msg.content == "!ping" {
            // Sending a message can fail, due to a network error, an
            // authentication error, or lack of permissions to post in the
            // channel, so log to stdout when some error happens, with a
            // description of it.
            send_msg(&ctx, msg, "Pong!").await;
        } else if msg.content.starts_with("!source") {
            send_msg(
                &ctx,
                msg,
                "source available at: https://github.com/squidboylan/meme-bot",
            )
            .await;
        } else if msg.content.starts_with("!meme") {
            let split_msg: Vec<&str> = msg.content.splitn(2, " ").collect();
            let data = ctx.data.read().await;
            if split_msg.len() == 1 {
                if let Some(imgflip_client) = data.get::<ImgflipClientContainer>() {
                    let client_lock = imgflip_client.read().await;
                    let mut response = "Available meme templates: ".to_string();
                    for meme in client_lock.list_memes() {
                        response.push_str(meme);
                        response.push_str(", ");
                    }
                    send_msg(&ctx, msg, &response).await;
                } else {
                    send_msg(&ctx, msg, "Failed to get imgflip client").await;
                }
            } else {
                let command_data: Vec<&str> = split_msg[1].split("\n").collect();
                if command_data.len() < 3 {
                    send_msg(&ctx, msg, "Please use !meme in the following format: !meme <MEME_NAME>\n<TEXT1>\n<TEXT2>").await;
                    return;
                } else {
                    if let Some(imgflip_client) = data.get::<ImgflipClientContainer>() {
                        let client_lock = imgflip_client.read().await;
                        let meme_id = client_lock.get_meme_id(command_data[0]);
                        let id = match meme_id {
                            Some(id) => id,
                            None => {
                                send_msg(&ctx, msg, "That meme does not exist").await;
                                return;
                            }
                        };
                        let client_res = client_lock
                            .caption_image(id, command_data[1], command_data[2])
                            .await;
                        match client_res {
                            Ok(res) => {
                                let response = res.data.url;
                                send_msg(&ctx, msg, &response).await;
                            }
                            Err(e) => send_msg(&ctx, msg, &e.to_string()).await,
                        }
                    } else {
                        send_msg(&ctx, msg, "Failed to get imgflip client").await;
                    }
                }
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
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
    let mut client = Client::new(&token)
        .event_handler(Handler)
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
