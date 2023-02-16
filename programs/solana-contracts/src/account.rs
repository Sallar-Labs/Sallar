use anchor_lang::prelude::*;

#[account]
pub struct BlocksState {
    pub authority: Pubkey,
    pub mint_nonce: u8,

    pub initial_token_distribution_already_performed: bool,
    pub blocks_collided: bool,
    
    pub top_block_number: u64,
    pub top_block_available_bp: u64,
    pub top_block_solution_timestamp: i64,
    pub top_block_balance: u64,
    pub top_block_distribution_address: Pubkey,
    pub top_block_distribution_nonce: u8,

    pub bottom_block_number: u64,
    pub bottom_block_available_bp: u64,
    pub bottom_block_solution_timestamp: i64,
    pub bottom_block_balance: u64,
    pub bottom_block_distribution_address: Pubkey,
    pub bottom_block_distribution_nonce: u8,

    pub final_staking_account_nonce: u8,
    pub final_staking_pool_in_round: u64,
    pub final_staking_last_staking_timestamp: i64,
    pub final_staking_left_reward_parts_in_round: f64,
    pub final_staking_left_balance_in_round: u64,
    
    pub final_mining_account_nonce: u8,
}