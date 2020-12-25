use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

use crate::imgflip;

pub struct Handler;

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
        let response = if msg.content == "!ping" {
            Some("!Pong".to_owned())
        } else if msg.content.starts_with("!source") {
            Some("source available at: https://github.com/squidboylan/meme-bot".to_owned())
        } else if msg.content.starts_with("!meme") {
            let split_msg: Vec<&str> = msg.content.splitn(2, " ").collect();
            let data = ctx.data.read().await;
            let line_count = msg.content.lines().count();
            if split_msg.len() == 1 || line_count == 1 {
                if let Some(imgflip_client) = data.get::<super::ImgflipClientContainer>() {
                    let client_lock = imgflip_client.read().await;
                    let mut response = "Available meme templates: ".to_string();
                    for meme in client_lock.list_memes() {
                        response.push_str(&format!("{}, ", meme));
                    }
                    response.push_str("\nuse the format: !meme <Name>\n<TEXT1>\n<TEXT2>\n...");
                    Some(response)
                } else {
                    Some("Failed to get imgflip client".to_owned())
                }
            } else {
                let command_data: Vec<&str> = split_msg[1].split("\n").collect();
                if let Some(imgflip_client) = data.get::<super::ImgflipClientContainer>() {
                    let client_lock = imgflip_client.read().await;
                    let client_res = client_lock
                        .caption_image(command_data[0].trim(), &command_data[1..])
                        .await;
                    match client_res {
                        Ok(res) => match res.data {
                            imgflip::CaptionResponseData::Data { url, page_url: _ } => {
                                Some(url.to_owned())
                            }
                            imgflip::CaptionResponseData::ErrorMessage(x) => Some(x.to_owned()),
                        },
                        Err(e) => Some(e.to_string()),
                    }
                } else {
                    Some("Failed to get imgflip client".to_owned())
                }
            }
        } else {
            None
        };
        if let Some(s) = response {
            send_msg(&ctx, msg, &s).await;
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
