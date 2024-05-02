use tendermint::evidence::Evidence as TendermintEvidence;

#[derive(Debug, Clone)]
pub enum EvidenceKind {
    DuplicateVote(String),
    LightClientAttack(String),
}

impl From<&TendermintEvidence> for EvidenceKind {
    fn from(value: &TendermintEvidence) -> Self {
        match value {
            TendermintEvidence::DuplicateVote(evidence) => {
                Self::DuplicateVote(evidence.vote_a.validator_address.to_string())
            }
            TendermintEvidence::LightClientAttack(_) => unimplemented!(),
        }
    }
}
