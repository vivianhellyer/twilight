use crate::{
    id::{ChannelId, GuildId, UserId},
    invite::TargetUserType,
    user::{self, User},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct InviteCreate {
    pub channel_id: ChannelId,
    pub code: String,
    pub created_at: String,
    pub guild_id: GuildId,
    pub inviter: Option<User>,
    pub max_age: u64,
    pub max_uses: u64,
    pub target_user_type: Option<TargetUserType>,
    pub target_user: Option<PartialUser>,
    pub temporary: bool,
    pub uses: u8, // will always be zero
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct PartialUser {
    avatar: Option<String>,
    /// Discriminator used to differentiate people with the same username.
    ///
    /// # serde
    ///
    /// The discriminator field can be deserialized from either a string or an
    /// integer. The field will always serialize into a string due to that being
    /// the type Discord's API uses.
    #[serde(with = "user::discriminator")]
    discriminator: u16,
    id: UserId,
    username: String,
}
