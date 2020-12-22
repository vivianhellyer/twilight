use crate::{
    id::UserId,
    user::{PremiumType, UserFlags},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct UserProfile {
    pub avatar: Option<String>,
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
    pub email: Option<String>,
    pub flags: Option<UserFlags>,
    pub id: UserId,
    pub locale: Option<String>,
    pub mfa_enabled: Option<bool>,
    #[serde(rename = "username")]
    pub name: String,
    pub premium_type: Option<PremiumType>,
    pub verified: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::{PremiumType, UserFlags, UserId, UserProfile};
    use serde_test::Token;

    fn user_tokens(discriminator_token: Token) -> Vec<Token> {
        vec![
            Token::Struct {
                name: "UserProfile",
                len: 11,
            },
            Token::Str("avatar"),
            Token::Some,
            Token::Str("hash"),
            Token::Str("bot"),
            Token::Bool(false),
            Token::Str("discriminator"),
            discriminator_token,
            Token::Str("email"),
            Token::Some,
            Token::Str("email@example.com"),
            Token::Str("flags"),
            Token::Some,
            Token::U64(131_072),
            Token::Str("id"),
            Token::NewtypeStruct { name: "UserId" },
            Token::Str("1"),
            Token::Str("locale"),
            Token::Some,
            Token::Str("en-us"),
            Token::Str("mfa_enabled"),
            Token::Some,
            Token::Bool(true),
            Token::Str("username"),
            Token::Str("user name"),
            Token::Str("premium_type"),
            Token::Some,
            Token::U8(2),
            Token::Str("verified"),
            Token::Some,
            Token::Bool(true),
            Token::StructEnd,
        ]
    }

    #[test]
    fn test_user_profile() {
        let value = UserProfile {
            avatar: Some("hash".to_owned()),
            bot: false,
            discriminator: 4,
            email: Some("email@example.com".to_owned()),
            flags: Some(UserFlags::VERIFIED_BOT_DEVELOPER),
            id: UserId(1),
            locale: Some("en-us".to_owned()),
            mfa_enabled: Some(true),
            name: "user name".to_owned(),
            premium_type: Some(PremiumType::Nitro),
            verified: Some(true),
        };

        // Deserializing a user profile with a string discriminator (which
        // Discord provides)
        serde_test::assert_tokens(&value, &user_tokens(Token::Str("0004")));

        // Deserializing a user profile with an integer discriminator. Userland
        // code may have this due to being a more compact memory representation
        // of a discriminator.
        serde_test::assert_de_tokens(&value, &user_tokens(Token::U64(4)));
    }
}
