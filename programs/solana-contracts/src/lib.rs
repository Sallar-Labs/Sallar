use anchor_lang::prelude::*;
use anchor_spl::token;

use token_math::*;
use utils::*;
use error::*;
use context::*;

mod token_math;
mod utils;
mod error;
mod context;
mod account;

const FINAL_STAKING_ACCOUNT_BALANCE_PART_FOR_STAKING: f64 = 0.001;

const DUSTS_PER_BLOCK: u64 = 20_000_000_000;

const MINT_SEED: &str = "mint";
const BLOCKS_STATE_SEED: &str = "blocks_state";
const DISTRIBUTION_TOP_BLOCK_SEED: &str = "distribution_top_block";
const DISTRIBUTION_BOTTOM_BLOCK_SEED: &str = "distribution_bottom_block";
const FINAL_STAKING_ACCOUNT_SEED: &str = "final_staking_account";
const FINAL_MINING_ACCOUNT_SEED: &str = "final_mining_account";

declare_id!("Fx175EjmJi6fRa79dLdvcHZx29CPFHBLvvHTmXmDnfGm");

#[program]
pub mod sallar {
    use super::*;

    #[access_control(valid_signer(&ctx.accounts.signer) valid_sysvar_address(&ctx.accounts.rent))]
    pub fn initialize(ctx: Context<InitializeContext>, mint_nonce: u8, top_block_nonce: u8, bottom_block_nonce: u8, final_staking_account_nonce: u8, final_mining_account_nonce: u8) -> Result<()> {
        let blocks_state = &mut ctx.accounts.blocks_state;
        blocks_state.authority = ctx.accounts.signer.key();
        blocks_state.mint_nonce = mint_nonce;

        blocks_state.top_block_distribution_address = ctx.accounts.distribution_top_block_address.key();
        blocks_state.top_block_distribution_nonce = top_block_nonce;
        blocks_state.top_block_solution_timestamp = 0;
        blocks_state.top_block_number = 1_u64;

        blocks_state.top_block_available_bp = calculate_max_bp(blocks_state.top_block_number) as u64;
        blocks_state.top_block_balance = DUSTS_PER_BLOCK;

        mint_tokens(ctx.accounts.mint.to_account_info(), ctx.accounts.distribution_top_block_address.to_account_info(), ctx.accounts.mint.to_account_info(), ctx.accounts.token_program.to_account_info(), mint_nonce, DUSTS_PER_BLOCK)?;

        blocks_state.bottom_block_distribution_address = ctx.accounts.distribution_bottom_block_address.key();
        blocks_state.bottom_block_distribution_nonce = bottom_block_nonce;
        blocks_state.bottom_block_solution_timestamp = 0;
        blocks_state.bottom_block_number = 2_600_000_u64;

        blocks_state.bottom_block_available_bp = calculate_max_bp(blocks_state.bottom_block_number) as u64;
        blocks_state.bottom_block_balance = DUSTS_PER_BLOCK;

        mint_tokens(ctx.accounts.mint.to_account_info(), ctx.accounts.distribution_bottom_block_address.to_account_info(), ctx.accounts.mint.to_account_info(), ctx.accounts.token_program.to_account_info(), mint_nonce, DUSTS_PER_BLOCK)?;
        
        blocks_state.initial_token_distribution_already_performed = false;
        blocks_state.blocks_collided = false;

        blocks_state.final_staking_account_nonce = final_staking_account_nonce;
        blocks_state.final_staking_pool_in_round = 0;
        blocks_state.final_staking_last_staking_timestamp = 0;
        blocks_state.final_staking_left_reward_parts_in_round = 1.0;
        blocks_state.final_staking_left_balance_in_round = 0;

        blocks_state.final_mining_account_nonce = final_mining_account_nonce;

        Ok(())
    }

    #[access_control(valid_owner(&ctx.accounts.blocks_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) initial_token_distribution_not_performed_yet(&ctx.accounts.blocks_state))]
    pub fn initial_token_distribution(ctx: Context<InitialTokenDistributionContext>) -> Result<()> {
        let blocks_state = &mut ctx.accounts.blocks_state;
        let mint_nonce = blocks_state.mint_nonce;

        mint_tokens(ctx.accounts.mint.to_account_info(), ctx.accounts.organization_account.to_account_info(), ctx.accounts.mint.to_account_info(), ctx.accounts.token_program.to_account_info(), mint_nonce, 2_600_000_000_000_000_u64)?;

        blocks_state.initial_token_distribution_already_performed = true;

        Ok(())
    }

    #[access_control(valid_owner(&ctx.accounts.blocks_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) top_block_not_solved(&ctx.accounts.blocks_state) blocks_solution_required_interval_elapsed(&ctx.accounts.blocks_state.top_block_solution_timestamp))]
    pub fn solve_top_block<'info>(
        ctx: Context<'_, '_, '_, 'info, SolveTopBlockContext<'info>>,
        users_info: Vec<UserInfoTopBlock>
    ) -> Result<u64> {
        let blocks_state = &mut ctx.accounts.blocks_state;

        let block_number = blocks_state.top_block_number;
        let mint_nonce = blocks_state.mint_nonce;

        let top_bp_with_boost = calculate_top_bp_with_boost(block_number);
        let dust_per_bp = calculate_dust_per_bp(block_number);

        for account in ctx.remaining_accounts.iter() {
            let matching_users = users_info.iter().filter(|user_info| user_info.user_public_key == account.key()).collect::<Vec<&UserInfoTopBlock>>();

            let user_info = match matching_users.first() {
                Some(x) => x,
                None => return err!(MyError::MismatchBetweenRemainingAccountsAndUserInfo),
            };

            require!(matching_users.len() == 1,
                MyError::UserDuplicatedInUserInfoForTopBlock
            );

            let (current_user_total_bp, mut current_user_transfer_amount) = calculate_user_reward_top_block(user_info.user_request_without_boost, user_info.user_request_with_boost, top_bp_with_boost, dust_per_bp);

            if current_user_total_bp <= blocks_state.top_block_available_bp {
                blocks_state.top_block_available_bp = blocks_state.top_block_available_bp - current_user_total_bp;
            } else {
                return err!(MyError::UserRequestExceedsAvailableBPs);
            }

            if blocks_state.top_block_available_bp == 0 {
                current_user_transfer_amount = blocks_state.top_block_balance;
            }

            transfer_tokens(
                &ctx.accounts.distribution_top_block_address,
                account.to_account_info(),
                DISTRIBUTION_TOP_BLOCK_SEED,
                ctx.accounts.token_program.to_account_info(),
                blocks_state.top_block_distribution_nonce,
                current_user_transfer_amount,
            )?;

            blocks_state.top_block_balance = blocks_state.top_block_balance - current_user_transfer_amount;
        }

        switch_top_block_to_next_one_if_applicable(blocks_state, mint_nonce, &ctx.accounts.mint, ctx.accounts.distribution_top_block_address.to_account_info(), ctx.accounts.token_program.to_account_info())?;
        update_blocks_collided(blocks_state)?;

        Ok(blocks_state.top_block_number)
    }

    #[access_control(valid_owner(&ctx.accounts.blocks_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) bottom_block_not_solved(&ctx.accounts.blocks_state) blocks_solution_required_interval_elapsed(&ctx.accounts.blocks_state.bottom_block_solution_timestamp))]
    pub fn solve_bottom_block<'info>(
        ctx: Context<'_, '_, '_, 'info, SolveBottomBlockContext<'info>>,
        users_info: Vec<UserInfoBottomBlock>,
    ) -> Result<u64> {
        let blocks_state = &mut ctx.accounts.blocks_state;
        let block_number = blocks_state.bottom_block_number;
        let mint_nonce = blocks_state.mint_nonce;

        let mut current_user_total_bp;
        let mut current_user_transfer_amount;

        let dust_per_bp = calculate_dust_per_bp(block_number);

        for account in ctx.remaining_accounts.iter() {
            let user_find_result = users_info.iter().filter(|user_info| user_info.user_public_key == account.key()).collect::<Vec<&UserInfoBottomBlock>>();

            require!(user_find_result.len() > 0,
                MyError::MismatchBetweenRemainingAccountsAndUserInfo
            );

            for user_sub_info in &user_find_result {
                let bottom_bp_with_boost = calculate_bottom_bp_with_boost(block_number, user_sub_info.user_balance);
                let bottom_bp_without_boost = calculate_bottom_bp_without_boost(user_sub_info.user_balance);

                (current_user_total_bp, current_user_transfer_amount) = calculate_user_reward_bottom_block(user_sub_info.user_request_without_boost, user_sub_info.user_request_with_boost, bottom_bp_without_boost, bottom_bp_with_boost, dust_per_bp, user_sub_info.user_balance);

                if current_user_total_bp <= blocks_state.bottom_block_available_bp {
                    blocks_state.bottom_block_available_bp = blocks_state.bottom_block_available_bp - current_user_total_bp;
                } else {
                    return err!(MyError::UserRequestExceedsAvailableBPs);
                }

                if blocks_state.bottom_block_available_bp == 0 {
                    current_user_transfer_amount = blocks_state.bottom_block_balance;
                }

                transfer_tokens(
                    &ctx.accounts.distribution_bottom_block_address,
                    account.to_account_info(),
                    DISTRIBUTION_BOTTOM_BLOCK_SEED,
                    ctx.accounts.token_program.to_account_info(),
                    blocks_state.bottom_block_distribution_nonce,
                    current_user_transfer_amount,
                )?;

                blocks_state.bottom_block_balance = blocks_state.bottom_block_balance - current_user_transfer_amount;
            }
        }

        switch_bottom_block_to_next_one_if_applicable(blocks_state, mint_nonce, &ctx.accounts.mint, ctx.accounts.distribution_bottom_block_address.to_account_info(), ctx.accounts.token_program.to_account_info())?;
        update_blocks_collided(blocks_state)?;

        Ok(blocks_state.bottom_block_number)
    }

    #[access_control(valid_owner(&ctx.accounts.blocks_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) blocks_collided(&ctx.accounts.blocks_state) blocks_solved(&ctx.accounts.blocks_state))]
    pub fn final_mining<'info>(ctx: Context<'_, '_, '_, 'info, FinalMiningContext<'info>>, users_info: Vec<UserInfoFinalMining>) -> Result<()> {
        let blocks_state = &mut ctx.accounts.blocks_state;

        for account in ctx.remaining_accounts.iter() {
            let user_find_result = users_info.iter().filter(|user_info| user_info.user_public_key == account.key()).collect::<Vec<&UserInfoFinalMining>>();
            
            require!(user_find_result.len() > 0,
                MyError::MismatchBetweenRemainingAccountsAndUserInfo
            );

            let mut total_amount = 0;
            for user_sub_info in &user_find_result {
                let transfer_amount = match user_sub_info.final_mining_balance {
                    0...124_999_999_999_999 => 25_000_000,
                    125_000_000_000_000...249_999_999_999_999 => 50_000_000,
                    250_000_000_000_000...499_999_999_999_999 => 100_000_000,
                    500_000_000_000_000...1_000_000_000_000_000 => 250_000_000,
                    _ => 500_000_000
                };
                total_amount += transfer_amount;
            }
            transfer_tokens(
                &ctx.accounts.final_mining_account,
                account.to_account_info(),
                FINAL_MINING_ACCOUNT_SEED,
                ctx.accounts.token_program.to_account_info(),
                blocks_state.final_mining_account_nonce,
                total_amount,
            )?;
        }

        Ok(())
    }

    #[access_control(valid_owner(&ctx.accounts.blocks_state, &ctx.accounts.signer) valid_signer(&ctx.accounts.signer) blocks_collided(&ctx.accounts.blocks_state) blocks_solved(&ctx.accounts.blocks_state) final_staking_required_interval_elapsed(&ctx.accounts.blocks_state.final_staking_last_staking_timestamp))]
    pub fn final_staking<'info>(ctx: Context<'_, '_, '_, 'info, FinalStakingContext<'info>>, users_info: Vec<UserInfoFinalStaking>) -> Result<()> {
        let blocks_state = &mut ctx.accounts.blocks_state;
        let mut total_users_reward_part = 0.0;

        if blocks_state.final_staking_left_balance_in_round == 0 {
            blocks_state.final_staking_pool_in_round = ((token::accessor::amount(&ctx.accounts.final_staking_account.to_account_info())? as f64) * (FINAL_STAKING_ACCOUNT_BALANCE_PART_FOR_STAKING)) as u64;
            
            require!(blocks_state.final_staking_pool_in_round > 0,
                MyError::FinalStakingPoolInRoundIsEmpty
            );

            blocks_state.final_staking_left_balance_in_round = blocks_state.final_staking_pool_in_round;
            blocks_state.final_staking_left_reward_parts_in_round = 1.0; 
        }

        users_info.iter().for_each(|user_info| total_users_reward_part += user_info.reward_part);

        require!(total_users_reward_part <= 1.0,
            MyError::UserRewardPartsSumTooHigh
        );

        let mut current_user_transfer_amount;

        for account in ctx.remaining_accounts.iter() {
            let user_find_result = users_info.iter().filter(|user_info| user_info.user_public_key == account.key()).collect::<Vec<&UserInfoFinalStaking>>();

            require!(user_find_result.len() > 0,
                MyError::MismatchBetweenRemainingAccountsAndUserInfo
            );

            for user_sub_info in &user_find_result {
                require!(user_sub_info.reward_part <= 1.0 && user_sub_info.reward_part > 0.0,
                    MyError::UserRequestExceedsAvailableRewardParts
                );

                let reward_parts_pool_after_user = blocks_state.final_staking_left_reward_parts_in_round - user_sub_info.reward_part;
                require!(reward_parts_pool_after_user >= 0.0, MyError::UserRequestExceedsAvailableRewardParts);

                if reward_parts_pool_after_user == 0.0 {
                    current_user_transfer_amount = blocks_state.final_staking_left_balance_in_round;
                } else {
                    current_user_transfer_amount = (user_sub_info.reward_part * blocks_state.final_staking_pool_in_round as f64) as u64;
                }

                require!(current_user_transfer_amount <= blocks_state.final_staking_left_balance_in_round,
                    MyError::LackOfFundsToPayTheReward
                );

                transfer_tokens(
                    &ctx.accounts.final_staking_account,
                    account.to_account_info(),
                    FINAL_STAKING_ACCOUNT_SEED,
                    ctx.accounts.token_program.to_account_info(),
                    blocks_state.final_staking_account_nonce,
                    current_user_transfer_amount,
                )?;

                blocks_state.final_staking_left_reward_parts_in_round = reward_parts_pool_after_user;
                blocks_state.final_staking_left_balance_in_round = blocks_state.final_staking_left_balance_in_round - current_user_transfer_amount;
            }
        }

        if blocks_state.final_staking_left_balance_in_round == 0 {
            blocks_state.final_staking_last_staking_timestamp = Clock::get()?.unix_timestamp;
        }

        Ok(())
    }
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserInfoTopBlock {
    pub user_public_key: Pubkey,
    pub user_request_without_boost: u8,
    pub user_request_with_boost: u8
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserInfoBottomBlock {
    pub user_public_key: Pubkey,
    pub user_balance: u64,
    pub user_request_without_boost: u8,
    pub user_request_with_boost: u8
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserInfoFinalMining {
    pub user_public_key: Pubkey,
    pub final_mining_balance: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct UserInfoFinalStaking {
    pub user_public_key: Pubkey,
    pub reward_part: f64,
}