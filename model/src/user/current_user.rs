use super::{PremiumType, UserFlags};
use crate::id::UserId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CurrentUser {
    /// User's avatar hash.
    ///
    /// To retrieve the url to the avatar, you can follow [Discord's documentation] on
    /// Image formatting.
    ///
    /// [Discord's documentation]: https://discord.com/developers/docs/reference#image-formatting
    pub avatar: Option<String>,
    /// Whether the user belongs to an OAuth2 application.
    #[serde(default)]
    pub bot: bool,
    /// Discriminator used to differentiate people with the same username.
    ///
    /// # serde
    ///
    /// The discriminator field can be deserialized from either a string or an
    /// integer. The field will always serialize into a string due to that being
    /// the type Discord's API uses.
    #[serde(with = "super::discriminator")]
    pub discriminator: u16,
    /// User's email address associated to the account.
    ///
    /// Requires the `email` oauth scope. See [Discord's documentation] for
    /// more information.
    ///
    /// [Discord's documentation]: https://discord.com/developers/docs/resources/user#user-object-user-structure
    pub email: Option<String>,
    /// All flags on a user's account.
    pub flags: Option<UserFlags>,
    /// User's id.
    pub id: UserId,
    /// User's chosen language option.
    pub locale: Option<String>,
    /// Whether the user has two factor enabled on their account.
    pub mfa_enabled: bool,
    /// User's username, not unique across the platform.
    #[serde(rename = "username")]
    pub name: String,
    /// Type of Nitro subscription on a user's account.
    pub premium_type: Option<PremiumType>,
    /// Public flags on a user's account.
    pub public_flags: Option<UserFlags>,
    /// Whether the email on this account has been verified.
    ///
    /// Requires the `email` oauth scope. See [Discord's documentation] for
    /// more information.
    ///
    /// [Discord's documentation]: https://discord.com/developers/docs/resources/user#user-object-user-structure
    pub verified: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::{CurrentUser, PremiumType, UserFlags, UserId};
    use serde_test::Token;

    fn user_tokens(discriminator_token: Token) -> Vec<Token> {
        vec![
            Token::Struct {
                name: "CurrentUser",
                len: 12,
            },
            Token::Str("avatar"),
            Token::Some,
            Token::Str("avatar hash"),
            Token::Str("bot"),
            Token::Bool(true),
            Token::Str("discriminator"),
            discriminator_token,
            Token::Str("email"),
            Token::None,
            Token::Str("flags"),
            Token::None,
            Token::Str("id"),
            Token::NewtypeStruct { name: "UserId" },
            Token::Str("1"),
            Token::Str("locale"),
            Token::Some,
            Token::Str("test locale"),
            Token::Str("mfa_enabled"),
            Token::Bool(true),
            Token::Str("username"),
            Token::Str("test name"),
            Token::Str("premium_type"),
            Token::Some,
            Token::U8(1),
            Token::Str("public_flags"),
            Token::Some,
            Token::U64(1),
            Token::Str("verified"),
            Token::Some,
            Token::Bool(true),
            Token::StructEnd,
        ]
    }

    #[test]
    fn test_current_user() {
        let value = CurrentUser {
            avatar: Some("avatar hash".to_owned()),
            bot: true,
            discriminator: 9999,
            email: None,
            id: UserId(1),
            mfa_enabled: true,
            name: "test name".to_owned(),
            verified: Some(true),
            premium_type: Some(PremiumType::NitroClassic),
            public_flags: Some(UserFlags::DISCORD_EMPLOYEE),
            flags: None,
            locale: Some("test locale".to_owned()),
        };

        // Deserializing a current user with a string discriminator (which
        // Discord provides)
        serde_test::assert_tokens(&value, &user_tokens(Token::Str("9999")));

        // Deserializing a current user with an integer discriminator. Userland
        // code may have this due to being a more compact memory representation
        // of a discriminator.
        serde_test::assert_de_tokens(&value, &user_tokens(Token::U64(9999)));
    }
}
