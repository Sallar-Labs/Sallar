use anchor_lang::prelude::*;

#[error_code]
pub enum MyError {
    #[msg("You are not an owner")]
    Unauthorized = 0,
    #[msg("Required time interval between solved blocks not passed (3 minutes).")]
    BlockSolutionAheadOfTime = 1,
    #[msg("Required time interval between final staking not passed (3 minutes).")]
    FinalStakingAheadOfTime = 2,
    #[msg("Required time interval between final mining not passed (3 minutes).")]
    FinalMiningAheadOfTime = 3,
    #[msg("Block already solved")]
    BlockAlreadySolved = 4,
    #[msg("Top block not solved yet")]
    TopBlockNotSolvedYet = 5,
    #[msg("Bottom block not solved yet")]
    BottomBlockNotSolvedYet = 6,
    #[msg("Initial token distribution already performed")]
    InitialTokenDistributionAlreadyPerformed = 7,
    #[msg("Blocks not collided yet")]
    BlocksNotCollidedYet = 8,
    #[msg("Final staking pool in round is empty")]
    FinalStakingPoolInRoundIsEmpty = 9,
    #[msg("User request exceeds available bps")]
    UserRequestExceedsAvailableBPs = 10,
    #[msg("User request exceeds available reward parts")]
    UserRequestExceedsAvailableRewardParts = 11,
    #[msg("Account from remaining accounts not found in user info")]
    MismatchBetweenRemainingAccountsAndUserInfo = 12,
    #[msg("Sum of user reward parts exceeds 1")]
    UserRewardPartsSumTooHigh = 13,
    #[msg("User duplicated in user info for top block")]
    UserDuplicatedInUserInfoForTopBlock = 14,
    #[msg("Lack of funds to pay the reward")]
    LackOfFundsToPayTheReward = 15,
    #[msg("Mismatch between available block BP and balance")]
    MismatchBetweenAvailableBlockBPAndBalance = 16,
}