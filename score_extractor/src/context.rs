use std::collections::hash_map::{self, HashMap};
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context as AnyhowContext;
use namada_core::types::address::Address as NamadaAddress;
use namada_core::types::storage::Epoch as NamadaEpoch;
use namada_sdk::rpc::query_native_token;
use shared::orm::players::PlayerKindDb;
use shared::orm::schema;
use tendermint_rpc::HttpClient;
use tokio::time;

use crate::db;

#[derive(Clone)]
pub struct Context {
    db_connection_pool: db::Pool,
    address_book: AddressBook,
    genesis_time: Option<chrono::NaiveDateTime>,
    epochs: Epochs,
    player_kinds: PlayerKinds,
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Context")
            .field("genesis_time", &self.genesis_time)
            .field("epochs", &self.epochs)
            .field("address_book", &self.address_book)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct PlayerKinds {
    inner: Arc<Mutex<HashMap<String, PlayerKindDb>>>,
}

impl Clone for PlayerKinds {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl PlayerKinds {
    fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_or_update(
        &self,
        player_id: &str,
        conn: &mut db::Connection,
    ) -> anyhow::Result<PlayerKindDb> {
        let mut player_kinds = self.inner.lock().unwrap();

        match player_kinds.entry(player_id.to_owned()) {
            hash_map::Entry::Occupied(occupied) => Ok(occupied.get().clone()),
            hash_map::Entry::Vacant(vacant) => {
                use diesel::prelude::*;
                use schema::players;

                let player_kind: PlayerKindDb = players::table
                    .filter(players::dsl::id.eq(&player_id))
                    .select(players::dsl::kind)
                    .first(conn)
                    .context("Failed to query player kind from players table")?;

                vacant.insert(player_kind.clone());
                Ok(player_kind)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct Epochs {
    pub v0_to_v1: NamadaEpoch,
    pub v1_to_v2: NamadaEpoch,
}

#[derive(Debug)]
pub struct AddressBook {
    inner: Arc<Addresses>,
}

impl Clone for AddressBook {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Deref for AddressBook {
    type Target = Addresses;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub struct Addresses {
    pub naan: NamadaAddress,
    pub upgrade_proposer: NamadaAddress,
}

pub struct GenesisTime(pub Option<chrono::NaiveDateTime>);

pub struct UpgradeProposer(pub NamadaAddress);

pub struct DatabaseUrl(pub String);

pub struct CometBftUrl(pub String);

impl Context {
    pub async fn new(
        epochs: Epochs,
        UpgradeProposer(upgrade_proposer): UpgradeProposer,
        GenesisTime(genesis_time): GenesisTime,
        DatabaseUrl(database_url): DatabaseUrl,
        CometBftUrl(cometbft_url): CometBftUrl,
    ) -> anyhow::Result<Self> {
        tracing::debug!(cometbft_url, "Connecting to CometBFT");
        let client =
            HttpClient::new(&*cometbft_url).context("Failed to instantiate CometBFT RPC client")?;
        let naan = loop {
            if let Ok(token) = query_native_token(&client).await {
                break token;
            }
            const RETRY_SLEEP: time::Duration = time::Duration::from_secs(30);
            tracing::warn!(
                retry_sleep = ?RETRY_SLEEP,
                cometbft_url,
                "Failed to query NAAN token address, retrying"
            );
            time::sleep(RETRY_SLEEP).await;
        };
        let address_book = AddressBook {
            inner: Arc::new(Addresses {
                naan,
                upgrade_proposer,
            }),
        };
        tracing::debug!(?address_book, "Fetched token address book from CometBFT");
        tracing::debug!(database_url, "Connecting to Postgres");
        let db_connection_pool = db::Pool::new(database_url).await?;
        Ok(Self {
            db_connection_pool,
            address_book,
            genesis_time,
            epochs,
            player_kinds: PlayerKinds::new(),
        })
    }

    pub fn db_connection_pool(&self) -> &db::Pool {
        &self.db_connection_pool
    }

    pub fn address_book(&self) -> &AddressBook {
        &self.address_book
    }

    pub fn genesis_time(&self) -> Option<&chrono::NaiveDateTime> {
        self.genesis_time.as_ref()
    }

    pub fn epochs(&self) -> &Epochs {
        &self.epochs
    }

    pub fn player_kinds(&self) -> &PlayerKinds {
        &self.player_kinds
    }
}
