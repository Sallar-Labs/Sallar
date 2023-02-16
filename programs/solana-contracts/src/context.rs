use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_spl::token::{ Mint, Token, TokenAccount };

use crate::account::*;
use crate::*;

const DISCRIMINATOR_LENGTH: usize = 8;
const AUTHORITY_PUBLIC_KEY_LENGTH: usize = 32;
const MINT_NONCE_LENGTH: usize = 1;

const INITIAL_TOKEN_DISTRIBUTION_ALREADY_PERFORMED_LENGTH: usize = 1;
const BLOCKS_COLLIDED_LENGTH: usize = 1;

const TOP_BLOCK_NUMBER_LENGTH: usize = 8;
const TOP_BLOCK_AVAILABLE_BP_LENGTH: usize = 8;
const TOP_BLOCK_SOLUTION_TIMESTAMP_LENGTH: usize = 8;
const TOP_BLOCK_BALANCE_LENGTH: usize = 8;
const TOP_BLOCK_DISTRIBUTION_ADDRESS_LENGTH: usize = 32;
const TOP_BLOCK_DISTRIBUTION_NONCE_LENGTH: usize = 1;

const BOTTOM_BLOCK_NUMBER_LENGTH: usize = 8;
const BOTTOM_BLOCK_AVAILABLE_BP_LENGTH: usize = 8;
const BOTTOM_BLOCK_SOLUTION_TIMESTAMP_LENGTH: usize = 8;
const BOTTOM_BLOCK_BALANCE_LENGTH: usize = 8;
const BOTTOM_BLOCK_DISTRIBUTION_ADDRESS_LENGTH: usize = 32;
const BOTTOM_BLOCK_DISTRIBUTION_NONCE_LENGTH: usize = 1;

const FINAL_STAKING_ACCOUNT_NONCE_LENGTH: usize = 1;
const FINAL_STAKING_POOL_IN_ROUND_LENGTH: usize = 8;
const FINAL_STAKING_LAST_STAKING_TIMESTAMP_LENGTH: usize = 8;
const FINAL_STAKING_ACCOUNT_BALANCE_LENGTH: usize = 8;
const FINAL_STAKING_REWARD_PARTS_POOL_LENGTH: usize = 8;

const FINAL_MINING_ACCOUNT_NONCE_LENGTH: usize = 1;

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct InitializeContext<'info> {
    #[account(init, payer = signer, space = InitializeContext::BLOCKS_STATE_LENGTH, seeds = [BLOCKS_STATE_SEED.as_bytes()], bump)]
    pub blocks_state: Box<Account<'info, BlocksState>>,

    #[account(
        init, 
        payer = signer,
        seeds = [MINT_SEED.as_bytes()],
        bump,
        mint::decimals = 9,
        mint::authority = mint
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = distribution_top_block_address, 
        seeds = [DISTRIBUTION_TOP_BLOCK_SEED.as_bytes()],
        bump,
    )]
    pub distribution_top_block_address: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = distribution_bottom_block_address,
        seeds = [DISTRIBUTION_BOTTOM_BLOCK_SEED.as_bytes()],
        bump,
    )]
    pub distribution_bottom_block_address: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = final_staking_account,
        seeds = [FINAL_STAKING_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub final_staking_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        token::mint = mint,
        token::authority = final_mining_account,
        seeds = [FINAL_MINING_ACCOUNT_SEED.as_bytes()],
        bump,
    )]
    pub final_mining_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    #[account(mut)]
    pub signer: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,
}

impl<'info> InitializeContext<'info> {
    const BLOCKS_STATE_LENGTH: usize =
    DISCRIMINATOR_LENGTH
    + AUTHORITY_PUBLIC_KEY_LENGTH
    + MINT_NONCE_LENGTH
    
    + INITIAL_TOKEN_DISTRIBUTION_ALREADY_PERFORMED_LENGTH
    + BLOCKS_COLLIDED_LENGTH
    
    + TOP_BLOCK_NUMBER_LENGTH
    + TOP_BLOCK_AVAILABLE_BP_LENGTH
    + TOP_BLOCK_SOLUTION_TIMESTAMP_LENGTH
    + TOP_BLOCK_BALANCE_LENGTH
    + TOP_BLOCK_DISTRIBUTION_ADDRESS_LENGTH
    + TOP_BLOCK_DISTRIBUTION_NONCE_LENGTH
    
    + BOTTOM_BLOCK_NUMBER_LENGTH
    + BOTTOM_BLOCK_AVAILABLE_BP_LENGTH
    + BOTTOM_BLOCK_SOLUTION_TIMESTAMP_LENGTH
    + BOTTOM_BLOCK_BALANCE_LENGTH
    + BOTTOM_BLOCK_DISTRIBUTION_ADDRESS_LENGTH
    + BOTTOM_BLOCK_DISTRIBUTION_NONCE_LENGTH
    
    + FINAL_STAKING_ACCOUNT_NONCE_LENGTH
    + FINAL_STAKING_POOL_IN_ROUND_LENGTH
    + FINAL_STAKING_LAST_STAKING_TIMESTAMP_LENGTH
    + FINAL_STAKING_ACCOUNT_BALANCE_LENGTH
    + FINAL_STAKING_REWARD_PARTS_POOL_LENGTH

    + FINAL_MINING_ACCOUNT_NONCE_LENGTH;
}

#[derive(Accounts)]
pub struct InitialTokenDistributionContext<'info> {
    pub blocks_state: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = blocks_state.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub organization_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state.authority)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SolveTopBlockContext<'info> {
    #[account(mut)]
    pub blocks_state: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [DISTRIBUTION_TOP_BLOCK_SEED.as_bytes()],
        bump = blocks_state.top_block_distribution_nonce
    )]
    pub distribution_top_block_address: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = blocks_state.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state.authority)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct SolveBottomBlockContext<'info> {
    #[account(mut)]
    pub blocks_state: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [DISTRIBUTION_BOTTOM_BLOCK_SEED.as_bytes()],
        bump = blocks_state.bottom_block_distribution_nonce,
    )]
    pub distribution_bottom_block_address: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [MINT_SEED.as_bytes()],
        bump = blocks_state.mint_nonce,
    )]
    pub mint: Box<Account<'info, Mint>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state.authority)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(bump: u8)]
pub struct FinalStakingContext<'info> {
    #[account(mut)]
    pub blocks_state: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [FINAL_STAKING_ACCOUNT_SEED.as_bytes()],
        bump = blocks_state.final_staking_account_nonce,
    )]
    pub final_staking_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state.authority)]
    pub signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct FinalMiningContext<'info> {
    #[account(mut)]
    pub blocks_state: Box<Account<'info, BlocksState>>,
    #[account(
        mut,
        seeds = [FINAL_MINING_ACCOUNT_SEED.as_bytes()],
        bump = blocks_state.final_mining_account_nonce,
    )]
    pub final_mining_account: Box<Account<'info, TokenAccount>>,
    pub token_program: Program<'info, Token>,
    #[account(mut, constraint = &signer.key() == &blocks_state.authority)]
    pub signer: Signer<'info>,
}