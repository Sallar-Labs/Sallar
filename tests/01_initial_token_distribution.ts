import { Sallar } from "../target/types/sallar";
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert } from "chai";
import { findProgramAddress } from "./utils/pda";
import { ComputeBudgetProgram, Connection, Transaction } from "@solana/web3.js";
import { getTestAccounts } from "./utils/accounts";

describe("Sallar", async () => {
    const provider: anchor.AnchorProvider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);

    const program: Program<Sallar> = anchor.workspace.Sallar;
    const connection = new Connection("http://localhost:8899", "finalized");

    let blocks_state_address: anchor.web3.PublicKey = null;
    let blocks_state_bump: number = null;

    let mint_address: anchor.web3.PublicKey = null;
    let mint_bump: number = null;

    let distribution_bottom_block_address: anchor.web3.PublicKey = null;
    let distribution_bottom_block_bump: number = null;

    let distribution_top_block_address: anchor.web3.PublicKey = null;
    let distribution_top_block_bump: number = null;

    let final_staking_address: anchor.web3.PublicKey = null;
    let final_staking_account_bump: number = null;

    let final_mining_address: anchor.web3.PublicKey = null;
    let final_mining_account_bump: number = null;

    let testAccounts: Array<anchor.web3.PublicKey> = [];

    let rem_accounts: any = [];
    let user_info_top_block = [];
    let user_info_bottom_block = [];
    let user_info_final_staking = [];

    let organization_account: anchor.web3.PublicKey = null;

    describe("Initializes state", async () => {
        it("Initialize test state", async () => {
            [mint_address, mint_bump] = findProgramAddress("sallar");
            [blocks_state_address, blocks_state_bump] =
                findProgramAddress("blocks_state");
            [final_staking_address, final_staking_account_bump] =
                findProgramAddress("final_staking");
            [final_mining_address, final_mining_account_bump] =
                findProgramAddress("final_mining");

            [distribution_top_block_address, distribution_top_block_bump] =
                findProgramAddress("distribution_top_block");
            [
                distribution_bottom_block_address,
                distribution_bottom_block_bump,
            ] = findProgramAddress("distribution_bottom_block");
        });

        it("Is initialized!", async () => {
            const tx = await program.methods
                .initialize(
                    mint_bump,
                    blocks_state_bump,
                    distribution_top_block_bump,
                    distribution_bottom_block_bump,
                    final_staking_account_bump,
                    final_mining_account_bump,
                )
                .accounts({
                    blocksStateAccount: blocks_state_address,
                    mint: mint_address,
                    distributionTopBlockAccount: distribution_top_block_address,
                    distributionBottomBlockAccount:
                        distribution_bottom_block_address,
                    finalStakingAccount: final_staking_address,
                    finalMiningAccount: final_mining_address,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    signer: provider.wallet.publicKey,
                    systemProgram: anchor.web3.SystemProgram.programId,
                })
                .transaction();

            const additionalComputeBudgetInstruction =
                ComputeBudgetProgram.setComputeUnitLimit({
                    units: 500_000,
                });
            const transaction = new Transaction()
                .add(additionalComputeBudgetInstruction)
                .add(tx);

            await provider.sendAndConfirm(transaction, [], {
                commitment: "confirmed",
                maxRetries: 3
            });

            const blocksState = await program.account.blocksState.fetch(
                blocks_state_address,
            );

            	assert.isTrue(
            		blocksState.authority.toBase58() ===
            		provider.wallet.publicKey.toBase58()
            	);

            	assert.isTrue(
            		blocksState.mintNonce === mint_bump
            	);

            	assert.isTrue(
            		blocksState.topBlockDistributionAddress.toBase58() ===
            		distribution_top_block_address.toBase58()
            	);

            	assert.isTrue(
            		blocksState.topBlockDistributionNonce === distribution_top_block_bump
            	);

            	assert.isTrue(
            		blocksState.topBlockSolutionTimestamp.toString() === new anchor.BN(0).toString()
            	);

            	assert.isTrue(
            		blocksState.topBlockBalance.toString() === new anchor.BN(2_000_000_000_000).toString()
            	);

            	assert.isTrue(
            		blocksState.bottomBlockDistributionAddress.toBase58() ===
            		distribution_bottom_block_address.toBase58()
            	);

            	assert.isTrue(
            		blocksState.bottomBlockDistributionNonce === distribution_bottom_block_bump
            	);

            	assert.isTrue(
            		blocksState.bottomBlockBalance.toString() === new anchor.BN(2_000_000_000_000).toString()
            	);

            	assert.isTrue(
            		blocksState.bottomBlockSolutionTimestamp.toString() === new anchor.BN(0).toString()
            	);

            	assert.isTrue(
            		blocksState.topBlockNumber.toString() ===
            		new anchor.BN(1).toString()
            	);

            	assert.isTrue(
            		blocksState.bottomBlockNumber.toString() ===
            		new anchor.BN(2600000).toString()
            	);
        });

        it("PASS - Create 2 new remainingTokenAccounts", async () => {
            testAccounts = await getTestAccounts(0, 2, connection);
            organization_account = testAccounts[0];

            for (let i = 0; i < testAccounts.length; i++) {
                rem_accounts.push({
                    pubkey: testAccounts[i],
                    isWritable: true,
                    isSigner: false,
                });

                user_info_top_block.push({
                    userPublicKey: testAccounts[i],
                    userRequestWithoutBoost: new anchor.BN(1),
                    userRequestWithBoost: new anchor.BN(0),
                });
                user_info_bottom_block.push({
                    userPublicKey: testAccounts[i],
                    userBalance: new anchor.BN(2_000_000_000),
                    userRequestWithoutBoost: new anchor.BN(1),
                    userRequestWithBoost: new anchor.BN(0),
                });
                user_info_final_staking.push({
                    userPublicKey: testAccounts[i],
                    rewardPart: 0.2,
                });
            }
        });

        describe("Initial token distribution", () => {
            it("Pass - (Initial token distribution)", async () => {
                await program.methods
                    .initialTokenDistribution()
                    .accounts({
                        blocksStateAccount: blocks_state_address,
                        mint: mint_address,
                        organizationAccount: organization_account,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        signer: provider.wallet.publicKey,
                    })
                    .rpc({maxRetries: 3});
            });

            it("FAIL - (Second run Initial Sale)", async () => {
                try {
                    await program.methods
                        .initialTokenDistribution()
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            mint: mint_address,
                            organizationAccount: organization_account,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: provider.wallet.publicKey,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.error.errorMessage,
                        "Initial token distribution already performed",
                    );

                    return;
                }
            });
        });

        describe("Block collison", () => {
            it("FAIL - (Second run Initial Sale)", async () => {
                try {
                    await program.methods
                        .initialTokenDistribution()
                        .accounts({
                            blocksStateAccount: blocks_state_address,
                            mint: mint_address,
                            organizationAccount: organization_account,
                            tokenProgram: TOKEN_PROGRAM_ID,
                            signer: provider.wallet.publicKey,
                        })
                        .rpc();
                        assert.fail("Transaction succeeded but was expected to fail");
                } catch (error) {
                    assert.equal(
                        error.error.errorMessage,
                        "Initial token distribution already performed",
                    );

                    return;
                }
            });
        });
    });
});
