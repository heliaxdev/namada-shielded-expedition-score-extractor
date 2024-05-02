use std::fmt::Display;

use diesel::{query_builder::AsChangeset, Queryable, Selectable};
use serde::{Deserialize, Serialize};

use crate::schema::players;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::PlayerKind"]
pub enum PlayerKindDb {
    Crew,
    Pilot,
}

impl Display for PlayerKindDb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlayerKindDb::Crew => write!(f, "crew"),
            PlayerKindDb::Pilot => write!(f, "pilot"),
        }
    }
}

#[derive(Serialize, Queryable, Selectable, Clone)]
#[diesel(table_name = players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerDb {
    pub id: String,
    pub moniker: String,
    pub namada_player_address: String,
    pub namada_validator_address: Option<String>,
    pub email: String,
    pub kind: PlayerKindDb,
    pub score: i64,
    pub block_height: Option<i32>,
    pub avatar_url: Option<String>,
    pub is_banned: Option<bool>,
}

#[derive(Serialize, AsChangeset, Clone)]
#[diesel(table_name = players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PlayerUpdateValidatorAddressDb {
    pub namada_validator_address: Option<String>,
}
