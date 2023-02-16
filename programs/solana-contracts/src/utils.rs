use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_spl::token::{ Transfer, self, MintTo, TokenAccount, Mint };

use crate::error::MyError;
use crate::account::BlocksState;
use crate::MINT_SEED;
use crate::token_math::calculate_max_bp;
use crate::DUSTS_PER_BLOCK;

const MIN_BLOCKS_SOLUTION_INTERVAL_SECONDS: i64 = 180;
const MIN_FINAL_STAKING_SOLUTION_INTERVAL_SECONDS: i64 = 72_000;

pub fn transfer_tokens<'a>(authority: &Box<Account<'a,TokenAccount>>, to: AccountInfo<'a>, seed: &'a str, program: AccountInfo<'a>, nonce: u8, amount: u64) -> Result<()> {
    let seeds = &[seed.as_bytes(), &[nonce]];
    let signer_seeds = &[&seeds[..]];

    let from = authority.to_account_info();
    let authority = authority.to_account_info();
    
    let cpi_accounts = Transfer {
        from,
        to,
        authority
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    token::transfer(cpi_ctx, amount)
}

pub fn mint_tokens<'a>(mint: AccountInfo<'a>, to: AccountInfo<'a>, authority: AccountInfo<'a>, program: AccountInfo<'a>, nonce: u8, amount: u64) -> Result<()> {
    let seeds = &[MINT_SEED.as_bytes(), &[nonce]];
    let signer_seeds = &[&seeds[..]];
    
    let cpi_accounts = MintTo {
        mint,
        to,
        authority,
    };

    let cpi_ctx =
        CpiContext::new_with_signer(program, cpi_accounts, signer_seeds); 

    token::mint_to(cpi_ctx, amount)
}

pub fn valid_owner(state: &BlocksState, signer: &AccountInfo) -> Result<()> {
    require!(signer.key.eq(&state.authority), MyError::Unauthorized);

    Ok(())
}

pub fn valid_signer(signer: &AccountInfo) -> Result<()> {
    require!(signer.is_signer, MyError::Unauthorized);

    Ok(())
}

pub fn blocks_solution_required_interval_elapsed(timestamp: &i64) -> Result<()> {
    require!(Clock::get()?.unix_timestamp - timestamp >= MIN_BLOCKS_SOLUTION_INTERVAL_SECONDS, MyError::BlockSolutionAheadOfTime);
    
    Ok(())
}

pub fn final_staking_required_interval_elapsed(timestamp: &i64) -> Result<()> {
    require!(Clock::get()?.unix_timestamp - timestamp >= MIN_FINAL_STAKING_SOLUTION_INTERVAL_SECONDS, MyError::FinalStakingAheadOfTime);
    
    Ok(())
}

pub fn blocks_collided(state: &BlocksState) -> Result<()> {
    require!(state.blocks_collided == true, MyError::BlocksNotCollidedYet);

    Ok(())
}

pub fn top_block_not_solved(state: &BlocksState) -> Result<()> {
    require!(state.top_block_available_bp > 0, MyError::BlockAlreadySolved);

    Ok(())
}

pub fn bottom_block_not_solved(state: &BlocksState) -> Result<()> {
    require!(state.bottom_block_available_bp > 0, MyError::BlockAlreadySolved);

    Ok(())
}

pub fn blocks_solved(state: &BlocksState) -> Result<()> {
    require!(state.top_block_available_bp <= 0, MyError::TopBlockNotSolvedYet);
    require!(state.bottom_block_available_bp <= 0, MyError::BottomBlockNotSolvedYet);

    Ok(())
}

pub fn update_blocks_collided(state: &mut BlocksState) -> Result<()> {
    if !can_block_be_switched(state) {
        state.blocks_collided = true;
    }

    Ok(())
}

pub fn initial_token_distribution_not_performed_yet(state: &BlocksState)-> Result<()> {
    require!(
        state.initial_token_distribution_already_performed == false,
        MyError::InitialTokenDistributionAlreadyPerformed
    );

    Ok(())
}

pub fn valid_sysvar_address<T>(rent: &Sysvar<T>) -> Result<()> where T: anchor_lang::prelude::SolanaSysvar {
    require_eq!(rent.key(), sysvar::rent::ID);

    Ok(())
}

pub fn can_block_be_switched(state: &BlocksState) -> bool {
    state.bottom_block_number - 1 > state.top_block_number
}

pub fn switch_top_block_to_next_one_if_applicable<'a>(state: &mut BlocksState, mint_nonce: u8, mint: &Box<Account<'a, Mint>>, distribution_top_block_address: AccountInfo<'a>, token_program: AccountInfo<'a>) -> Result<()> {
    require!(
        (state.top_block_balance == 0 && state.top_block_available_bp == 0) || (state.top_block_balance > 0 && state.top_block_available_bp > 0),
        MyError::MismatchBetweenAvailableBlockBPAndBalance
    );

    if state.top_block_available_bp == 0 && can_block_be_switched(state) {
        state.top_block_solution_timestamp = Clock::get()?.unix_timestamp;
        state.top_block_number = state.top_block_number + 1;

        let authority = mint.to_account_info();
        let mint_token_account = mint.to_account_info();

        mint_tokens(authority, distribution_top_block_address, mint_token_account, token_program, mint_nonce, DUSTS_PER_BLOCK)?;

        state.top_block_available_bp = calculate_max_bp(state.top_block_number) as u64;
        state.top_block_balance = DUSTS_PER_BLOCK;
    }

    Ok(())
}

pub fn switch_bottom_block_to_next_one_if_applicable<'a>(state: &mut BlocksState, mint_nonce: u8, mint: &Box<Account<'a, Mint>>, distribution_bottom_block_address: AccountInfo<'a>, token_program: AccountInfo<'a>) -> Result<()> {
    require!(
        (state.bottom_block_balance == 0 && state.bottom_block_available_bp == 0) || (state.bottom_block_balance > 0 && state.bottom_block_available_bp > 0),
        MyError::MismatchBetweenAvailableBlockBPAndBalance
    );

    if state.bottom_block_available_bp == 0 && can_block_be_switched(state) {
        state.bottom_block_solution_timestamp = Clock::get()?.unix_timestamp;
        state.bottom_block_number = state.bottom_block_number - 1;

        let authority = mint.to_account_info();
        let mint_token_account = mint.to_account_info();

        mint_tokens(authority, distribution_bottom_block_address, mint_token_account, token_program, mint_nonce, DUSTS_PER_BLOCK)?;

        state.bottom_block_available_bp = calculate_max_bp(state.bottom_block_number) as u64;
        state.bottom_block_balance = DUSTS_PER_BLOCK;
    }

    Ok(())
}