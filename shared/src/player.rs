use std::fmt;

use anyhow::Context;
use orm::players::PlayerKindDb;

use crate::id::Id;
use crate::transaction::RawMemo;

#[derive(Debug, Clone)]
pub enum PlayerKind {
    Pilot,
    Crew,
}

impl fmt::Display for PlayerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pilot => write!(f, "pilot"),
            Self::Crew => write!(f, "crew"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub moniker: String,
    pub namada_public_key: Id,
    pub email: String,
    pub kind: PlayerKind,
}

impl From<&PlayerKind> for PlayerKindDb {
    fn from(value: &PlayerKind) -> Self {
        match value {
            PlayerKind::Pilot => Self::Pilot,
            PlayerKind::Crew => Self::Crew,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PlayerId(pub String);

impl TryFrom<RawMemo> for PlayerId {
    type Error = anyhow::Error;

    fn try_from(RawMemo(raw_memo): RawMemo) -> Result<Self, Self::Error> {
        use std::str::FromStr;
        let originating_player = String::from_utf8(raw_memo).context("Memo is not UTF-8 text")?;
        namada_core::types::key::common::PublicKey::from_str(&originating_player)
            .context("Invalid Namada public key")?;
        Ok(Self(originating_player))
    }
}

impl TryFrom<&RawMemo> for PlayerId {
    type Error = anyhow::Error;

    fn try_from(raw: &RawMemo) -> Result<Self, Self::Error> {
        use std::str::FromStr;
        let originating_player = std::str::from_utf8(&raw.0).context("Memo is not UTF-8 text")?;
        namada_core::types::key::common::PublicKey::from_str(originating_player)
            .context("Invalid Namada public key")?;
        Ok(Self(originating_player.to_owned()))
    }
}
