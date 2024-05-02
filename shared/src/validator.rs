use orm::validators::{TmAddressInsertDb, ValidatorInsertDb};

use crate::{block::Epoch, id::Id};
use std::str::FromStr;

pub type VotingPower = String;

#[derive(Debug, Clone)]
pub struct ValidatorSet {
    pub validators: Vec<Validator>,
    pub epoch: Epoch,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub address: Id,
    pub voting_power: VotingPower,
    pub tm_address: Id,
    pub max_commission: String,
    pub commission: String,
    pub email: String,
    pub description: Option<String>,
    pub website: Option<String>,
    pub discord_handler: Option<String>,
    pub avatar: Option<String>,
}

impl ValidatorSet {
    pub fn to_validators_db(&self) -> Vec<ValidatorInsertDb> {
        self.validators
            .iter()
            .map(|validator| ValidatorInsertDb {
                namada_address: validator.address.to_string(),
                voting_power: f32::from_str(&validator.voting_power).unwrap() as i32,
                max_commission: validator.max_commission.clone(),
                commission: validator.commission.clone(),
                email: validator.email.clone(),
                website: validator.website.clone(),
                description: validator.description.clone(),
                discord_handle: validator.discord_handler.clone(),
                avatar: validator.avatar.clone(),
                epoch: self.epoch as i32,
            })
            .collect()
    }

    pub fn to_tm_addresses_db(&self) -> Vec<TmAddressInsertDb> {
        self.validators
            .iter()
            .map(|validator| TmAddressInsertDb {
                tm_address: validator.tm_address.to_string(),
                epoch: self.epoch as i32,
                validator_namada_address: validator.address.to_string(),
            })
            .collect()
    }
}
