// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "evidence_kind"))]
    pub struct EvidenceKind;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "governance_kind"))]
    pub struct GovernanceKind;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "governance_result"))]
    pub struct GovernanceResult;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "player_kind"))]
    pub struct PlayerKind;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "task_type"))]
    pub struct TaskType;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tx_exit_status"))]
    pub struct TxExitStatus;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tx_kind"))]
    pub struct TxKind;

    #[derive(diesel::query_builder::QueryId, std::fmt::Debug, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "vote_kind"))]
    pub struct VoteKind;
}

diesel::table! {
    blocks (id) {
        #[max_length = 64]
        id -> Varchar,
        height -> Int4,
        proposer_address -> Varchar,
        included_at -> Timestamp,
        epoch -> Int4,
    }
}

diesel::table! {
    chain_parameters (id) {
        id -> Int4,
        total_native_token_supply -> Varchar,
        total_staked_native_token -> Varchar,
        max_validators -> Int4,
        pos_inflation -> Varchar,
        pgf_steward_inflation -> Varchar,
        pgf_treasury_inflation -> Varchar,
        pgf_treasury -> Varchar,
    }
}

diesel::table! {
    commits (id) {
        id -> Int4,
        signature -> Nullable<Varchar>,
        address -> Varchar,
        #[max_length = 64]
        block_id -> Varchar,
    }
}

diesel::table! {
    crawler_state (id) {
        id -> Int4,
        height -> Int4,
        epoch -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::EvidenceKind;

    evidences (id) {
        id -> Int4,
        kind -> EvidenceKind,
        validator_address -> Varchar,
        #[max_length = 64]
        block_id -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::GovernanceKind;
    use super::sql_types::GovernanceResult;

    governance_proposals (id) {
        id -> Int4,
        content -> Nullable<Varchar>,
        kind -> GovernanceKind,
        author -> Varchar,
        start_epoch -> Int4,
        end_epoch -> Int4,
        grace_epoch -> Int4,
        result -> GovernanceResult,
        yay_votes -> Varchar,
        nay_votes -> Varchar,
        abstain_votes -> Varchar,
        #[max_length = 64]
        transaction_id -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::VoteKind;

    governance_votes (id) {
        id -> Int4,
        kind -> VoteKind,
        voter_address -> Varchar,
        proposal_id -> Int4,
        transaction_id -> Varchar,
        player_id -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TaskType;

    manual_tasks (id) {
        id -> Int4,
        task -> TaskType,
        player_id -> Varchar,
    }
}

diesel::table! {
    player_ranks (id) {
        id -> Int4,
        ranking -> Int4,
        player_id -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::PlayerKind;

    players (id) {
        id -> Varchar,
        moniker -> Varchar,
        namada_player_address -> Varchar,
        namada_validator_address -> Nullable<Varchar>,
        email -> Varchar,
        kind -> PlayerKind,
        score -> Int8,
        avatar_url -> Nullable<Varchar>,
        is_banned -> Nullable<Bool>,
        block_height -> Nullable<Int4>,
        internal_id -> Int4,
    }
}

diesel::table! {
    stewards (id) {
        id -> Int4,
        namada_address -> Varchar,
        block_height -> Int4,
    }
}

diesel::table! {
    task_completion_state (id) {
        id -> Int4,
        last_processed_time -> Timestamp,
        last_processed_height -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TaskType;

    tasks (id) {
        id -> Int4,
        task -> TaskType,
        player_id -> Varchar,
    }
}

diesel::table! {
    tm_addresses (id) {
        id -> Int4,
        tm_address -> Varchar,
        epoch -> Int4,
        validator_namada_address -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TxKind;
    use super::sql_types::TxExitStatus;

    transactions (id) {
        #[max_length = 64]
        id -> Varchar,
        #[max_length = 64]
        inner_hash -> Nullable<Varchar>,
        index -> Int4,
        kind -> TxKind,
        associated_data -> Nullable<Bytea>,
        exit_status -> TxExitStatus,
        gas_used -> Int4,
        memo -> Nullable<Bytea>,
        #[max_length = 64]
        block_id -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::TxKind;

    unidentified_tasks (id) {
        id -> Int4,
        tx_kind -> TxKind,
        player_id -> Varchar,
    }
}

diesel::table! {
    validators (id) {
        id -> Int4,
        namada_address -> Varchar,
        voting_power -> Int4,
        max_commission -> Varchar,
        commission -> Varchar,
        email -> Varchar,
        website -> Nullable<Varchar>,
        description -> Nullable<Varchar>,
        discord_handle -> Nullable<Varchar>,
        avatar -> Nullable<Varchar>,
        epoch -> Int4,
    }
}

diesel::joinable!(commits -> blocks (block_id));
diesel::joinable!(evidences -> blocks (block_id));
diesel::joinable!(governance_proposals -> transactions (transaction_id));
diesel::joinable!(governance_votes -> governance_proposals (proposal_id));
diesel::joinable!(governance_votes -> transactions (transaction_id));
diesel::joinable!(manual_tasks -> players (player_id));
diesel::joinable!(player_ranks -> players (player_id));
diesel::joinable!(tasks -> players (player_id));
diesel::joinable!(transactions -> blocks (block_id));
diesel::joinable!(unidentified_tasks -> players (player_id));

diesel::allow_tables_to_appear_in_same_query!(
    blocks,
    chain_parameters,
    commits,
    crawler_state,
    evidences,
    governance_proposals,
    governance_votes,
    manual_tasks,
    player_ranks,
    players,
    stewards,
    task_completion_state,
    tasks,
    tm_addresses,
    transactions,
    unidentified_tasks,
    validators,
);
