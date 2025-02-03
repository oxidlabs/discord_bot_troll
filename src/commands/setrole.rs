use serenity::all::{RoleId, UserId};
use serenity::builder::{ CreateCommand, CreateCommandOption };
use serenity::model::application::{ CommandOptionType, ResolvedOption, ResolvedValue };

pub fn run(options: &[ResolvedOption]) -> Option<(RoleId, UserId)> {
    if let Some(ResolvedOption { value: ResolvedValue::User(user, _), .. }) = options.first() {
        if let Some(ResolvedOption { value: ResolvedValue::Role(role), .. }) = options.get(1) {
            return Some((role.id, user.id))
        }
    }

    None
}

pub fn register() -> CreateCommand {
    CreateCommand::new("setrole")
        .default_member_permissions(serenity::all::Permissions::ADMINISTRATOR)
        .description("Set a role for a user")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::User,
                "user",
                "The user to set the role for"
            ).required(true)
        )
        .add_option(
            CreateCommandOption::new(CommandOptionType::Role, "role", "The role to set").required(
                true
            )
        )
}
