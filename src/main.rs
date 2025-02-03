use std::env;

use dotenv::dotenv;

use serenity::all::{ CreateInteractionResponse, CreateInteractionResponseMessage };
use serenity::async_trait;
use serenity::model::id::RoleId;
use serenity::model::prelude::*;
use serenity::prelude::*;

struct Bot {
    database: sqlx::SqlitePool,
}

mod commands;

#[async_trait]
impl EventHandler for Bot {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let response = match command.data.name.as_str() {
                "setrole" => {
                    if
                        let Some((role_id, user_id)) = commands::setrole::run(
                            &command.data.options()
                        )
                    {
                        let mut message = String::new();
                        let userid = user_id.get() as i64;
                        let roleid = role_id.get() as i64;
                        sqlx::query!(
                            "INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)",
                            userid,
                            roleid
                        )
                            .execute(&self.database).await
                            .unwrap();

                        // add the role to the user
                        if let Some(guild_id) = command.guild_id {
                            if
                                let Err(why) = ctx.http.add_member_role(
                                    guild_id,
                                    user_id,
                                    role_id,
                                    None
                                ).await
                            {
                                message = format!("Error assigning role: {:?}", why).to_owned();
                            }
                        }
                        if message == "" {
                            Some("Added Role to user".to_string())
                        } else {
                            Some(message)
                        }
                    } else {
                        Some("Failed to setrole".to_string())
                    }
                }
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = response {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Error sending response: {:?}", why);
                }
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let guild_id = GuildId::new(
            env
                ::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer")
        );

        guild_id.set_commands(&ctx.http, vec![commands::setrole::register()]).await.unwrap();
    }

    async fn guild_member_addition(&self, ctx: Context, new_member: Member) {
        // check if the user has a default role in db; if so, add the role to the user
        let user_id_i64 = new_member.user.id.get() as i64;
        if
            let Ok(record) = sqlx
                ::query!("SELECT role_id FROM user_roles WHERE user_id = ?", user_id_i64)
                .fetch_optional(&self.database).await
        {
            if let Some(row) = record {
                let role_id = RoleId::new(row.role_id as u64);
                if
                    let Err(why) = ctx.http.add_member_role(
                        new_member.guild_id,
                        new_member.user.id,
                        role_id,
                        None
                    ).await
                {
                    println!("Error assigning default role: {:?}", why);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Initiate a connection to the database file, creating the file if required.
    let database = sqlx::sqlite::SqlitePoolOptions
        ::new()
        .max_connections(5)
        .connect_with(
            sqlx::sqlite::SqliteConnectOptions
                ::new()
                .filename("database.sqlite")
                .create_if_missing(true)
        ).await
        .expect("Couldn't connect to database");

    // Run migrations, which updates the database's schema to the latest version.
    sqlx::migrate!("./migrations").run(&database).await.expect("Couldn't run database migrations");

    let bot = Bot {
        database,
    };

    let intents =
        GatewayIntents::GUILD_MEMBERS |
        GatewayIntents::GUILD_MESSAGES |
        GatewayIntents::DIRECT_MESSAGES |
        GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&token, intents)
        .event_handler(bot).await
        .expect("Err creating client");
    client.start().await.unwrap();
}
