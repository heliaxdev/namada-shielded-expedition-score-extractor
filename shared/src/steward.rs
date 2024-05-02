use orm::stewards::StewardInsertDb;

use crate::{block::BlockHeight, id::Id};

#[derive(Debug, Clone)]
pub struct StewardSet {
    pub stewards: Vec<Steward>,
    pub block_height: BlockHeight,
}

#[derive(Debug, Clone)]
pub struct Steward {
    pub namada_address: Id,
}

impl StewardSet {
    pub fn to_stewards_db(&self) -> Vec<StewardInsertDb> {
        self.stewards
            .iter()
            .map(|steward| StewardInsertDb {
                namada_address: steward.namada_address.to_string(),
                block_height: self.block_height as i32,
            })
            .collect()
    }
}
