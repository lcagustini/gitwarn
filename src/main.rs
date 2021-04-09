use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    utils::MessageBuilder,
};
use serenity::framework::standard::{
    StandardFramework,
    macros::{
        group
    }
};
use git2::Repository;
use std::thread;
use std::env;
use dotenv;

#[group]
struct General;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.content == "aping" {
            let channel = match msg.channel_id.to_channel(&context).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);

                    return;
                },
            };

            // The message builder allows for creating a message by
            // mentioning users dynamically, pushing "safe" versions of
            // content (such as bolding normalized content), displaying
            // emojis, and more.
            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&msg.author.name)
                .push(" used the 'ping' command in the ")
                .mention(&channel)
                .push(" channel")
                .build();

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    let framework = StandardFramework::new()
        //.configure(|c| c.prefix("!")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP);

    // Configure the client with your Discord bot token in the environment.
    dotenv::dotenv();
    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(&token)
        .framework(framework)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    let repo = match Repository::open("/home/lucas/Documents/blissys-client/") {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open: {}", e),
    };

    thread::spawn(move || {
        loop {
            thread::sleep_ms(1000);

            println!("{:?}", repo.head().unwrap().peel_to_commit().unwrap().id());
            repo.find_remote("origin").unwrap().fetch(&["master"], None, None);
            println!("{:?}", repo.head().unwrap().peel_to_commit().unwrap().id());
        }
    });

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
