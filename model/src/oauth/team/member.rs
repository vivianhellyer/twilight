use crate::{
    oauth::{id::TeamId, team::TeamMembershipState},
    user::User,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct TeamMember {
    pub membership_state: TeamMembershipState,
    pub permissions: Vec<String>,
    pub team_id: TeamId,
    pub user: User,
}

#[cfg(test)]
mod tests {
    use super::{TeamId, TeamMember, TeamMembershipState, User};
    use crate::id::UserId;
    use serde_test::Token;

    #[test]
    fn test_team_member() {
        let value = TeamMember {
            membership_state: TeamMembershipState::Accepted,
            permissions: vec!["*".to_owned()],
            team_id: TeamId(1),
            user: User {
                avatar: None,
                bot: false,
                discriminator: 1,
                email: None,
                flags: None,
                id: UserId(2),
                locale: None,
                mfa_enabled: None,
                name: "test".to_owned(),
                premium_type: None,
                public_flags: None,
                system: None,
                verified: None,
            },
        };

        serde_test::assert_tokens(
            &value,
            &[
                Token::Struct {
                    name: "TeamMember",
                    len: 4,
                },
                Token::Str("membership_state"),
                Token::U8(2),
                Token::Str("permissions"),
                Token::Seq { len: Some(1) },
                Token::Str("*"),
                Token::SeqEnd,
                Token::Str("team_id"),
                Token::NewtypeStruct { name: "TeamId" },
                Token::Str("1"),
                Token::Str("user"),
                Token::Struct {
                    name: "User",
                    len: 13,
                },
                Token::Str("avatar"),
                Token::None,
                Token::Str("bot"),
                Token::Bool(false),
                Token::Str("discriminator"),
                Token::Str("0001"),
                Token::Str("email"),
                Token::None,
                Token::Str("flags"),
                Token::None,
                Token::Str("id"),
                Token::NewtypeStruct { name: "UserId" },
                Token::Str("2"),
                Token::Str("locale"),
                Token::None,
                Token::Str("mfa_enabled"),
                Token::None,
                Token::Str("username"),
                Token::Str("test"),
                Token::Str("premium_type"),
                Token::None,
                Token::Str("public_flags"),
                Token::None,
                Token::Str("system"),
                Token::None,
                Token::Str("verified"),
                Token::None,
                Token::StructEnd,
                Token::StructEnd,
            ],
        );
    }
}
