-- Your SQL goes here

CREATE TYPE TASK_TYPE AS ENUM (
    'delegate_stake_on_v0',
    'delegate_stake_on_v1',
    'claim_pos_rewards',
    'shield_naan',
    'unshield_naan',
    'shield_to_shielded',
    'shield_asset_over_ibc',
    'submit_pre_genesis_bond_tx',
    'start_node5_min_from_genesis',
    'init_post_genesis_validator',
    'in_validator_set_for1_epoch',
    'vote_pgf_steward_proposal',
    'vote_upgrade_v0_to_v1',
    'vote_upgrade_v1_to_v2',
    'sign_first_block_of_upgrade_to_v2',
    'keep99_per_cent_uptime',
    'keep95_per_cent_uptime',
    'keep99_per_cent_gov_participation_rate',
    'keep90_per_cent_gov_participation_rate',
    'provide_public_rpc_endpoint',
    'operate_namada_indexer',
    'operate_namada_interface',
    'operate_cosmos_testnet_relayer',
    'operate_osmosis_testnet_relayer',
    'operate_noble_testnet_relayer',
    'operate_relayer_on_net_with_nfts',
    'operate_relayer_on_another_net',
    'integrate_se_in_block_explorer',
    'integrate_se_in_browser_wallet',
    'integrate_se_in_android_wallet',
    'integrate_se_in_ios_wallet',
    'integrate_se_in_another_wallet',
    'support_shielded_txs_in_block_explorer',
    'support_shielded_txs_in_browser_wallet',
    'support_shielded_txs_in_android_wallet',
    'support_shielded_txs_in_ios_wallet',
    'build_additional_foss_tooling',
    'build_web_app_with_shielded_action_on_ibc_chain',
    'osmosis_frontend_shielded_swaps',
    'another_app_with_shielded_action_on_ibc_chain',
    'reduce_masp_proof_gen_time',
    'increase_note_scan_speed',
    'find_and_prove_nam_specs_flaw',
    'optimize_nam_sm_exec_speed',
    'find_protocol_sec_vulnerability'
);

CREATE TABLE tasks (
    id SERIAL PRIMARY KEY,
    task TASK_TYPE NOT NULL,
    --completed_at INT NOT NULL,
    player_id VARCHAR NOT NULL,
    CONSTRAINT fk_player FOREIGN KEY(player_id) REFERENCES players(id) ON DELETE CASCADE
);

-- allows us to keep a single task completed per player
ALTER TABLE tasks
ADD UNIQUE (player_id, task);

--------------------------------------------

-- maintain a separate table for manual tasks, such
-- that we can drop all values from the `tasks` table
-- if necessary, without losing progress on manual tasks
CREATE TABLE manual_tasks (
    id SERIAL PRIMARY KEY,
    task TASK_TYPE NOT NULL,
    player_id VARCHAR NOT NULL,
    CONSTRAINT fk_player FOREIGN KEY(player_id) REFERENCES players(id) ON DELETE CASCADE
);

ALTER TABLE manual_tasks
ADD UNIQUE (player_id, task);

INSERT INTO manual_tasks (player_id, task)
SELECT id,
       'submit_pre_genesis_bond_tx'
FROM players
WHERE namada_validator_address IS NOT NULL;
