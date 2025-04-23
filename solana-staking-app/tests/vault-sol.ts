import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { expect } from 'chai';

describe('vault-sol', () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.VaultSol as Program;
  const connection = provider.connection;
  
  // Generate a new keypair for our test user
  const user = Keypair.generate();
  const admin = provider.wallet;
  let userBalance: number;
  
  // PDA for the vault and rewards accounts
  let vaultPDA: PublicKey;
  let vaultBump: number;
  let rewardsPDA: PublicKey;
  let rewardsBump: number;
  
  before(async () => {
    // Airdrop SOL to the user for transactions
    const airdropSignature = await connection.requestAirdrop(
      user.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSignature);
    userBalance = await connection.getBalance(user.publicKey);
    
    // Find the vault PDA
    const [vaultPDAAddress, bump] = await PublicKey.findProgramAddress(
      [Buffer.from("vault_sol_config")],
      program.programId
    );
    vaultPDA = vaultPDAAddress;
    vaultBump = bump;

    // Find the rewards pool PDA
    const [rewardsPDAAddress, rewardsBump] = await PublicKey.findProgramAddress(
      [Buffer.from("rewards_pool")],
      program.programId
    );
    rewardsPDA = rewardsPDAAddress;
    rewardsBump = rewardsBump;
  });

  it('Initializes the vault', async () => {
    const platformFeeBps = 500; // 5%
    const minStake = 0.1 * LAMPORTS_PER_SOL;

    await program.methods
      .initializeVault(platformFeeBps, new anchor.BN(minStake))
      .accounts({
        config: vaultPDA,
        rewardsPool: rewardsPDA,
        authority: admin.publicKey,
        treasury: admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
      
    // Verify the vault account was created correctly
    const vaultAccount = await program.account.vaultConfig.fetch(vaultPDA);
    expect(vaultAccount.authority.equals(admin.publicKey)).to.be.true;
    expect(vaultAccount.platformFeeBps).to.equal(platformFeeBps);
    expect(vaultAccount.minStakeAmount.toNumber()).to.equal(minStake);
    expect(vaultAccount.totalStaked.toNumber()).to.equal(0);
    expect(vaultAccount.stakersCount.toNumber()).to.equal(0);
    expect(vaultAccount.paused).to.be.false;

    // Verify rewards pool was created
    const rewardsAccount = await program.account.rewardsPool.fetch(rewardsPDA);
    expect(rewardsAccount.totalRewards.toNumber()).to.equal(0);
    expect(rewardsAccount.apyPoints).to.equal(500); // 5% default APY
    expect(rewardsAccount.distributedRewards.toNumber()).to.equal(0);
  });

  it('Creates a stake position', async () => {
    const stakeAmount = 0.5 * LAMPORTS_PER_SOL;
    const [stakePDA] = await PublicKey.findProgramAddress(
      [Buffer.from("stake_position"), user.publicKey.toBuffer()],
      program.programId
    );
    
    // Get initial balances
    const initialUserBalance = await connection.getBalance(user.publicKey);
    const initialTreasuryBalance = await connection.getBalance(admin.publicKey);
    
    // Create stake position
    await program.methods
      .createStake(new anchor.BN(stakeAmount))
      .accounts({
        config: vaultPDA,
        stakePosition: stakePDA,
        user: user.publicKey,
        treasury: admin.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();
      
    // Verify stake was recorded
    const stakePosition = await program.account.stakePosition.fetch(stakePDA);
    expect(stakePosition.owner.equals(user.publicKey)).to.be.true;
    expect(stakePosition.amount.toNumber()).to.equal(stakeAmount);
    
    // Verify balances changed correctly
    const newUserBalance = await connection.getBalance(user.publicKey);
    const newTreasuryBalance = await connection.getBalance(admin.publicKey);
    expect(newUserBalance).to.be.at.most(initialUserBalance - stakeAmount);
    expect(newTreasuryBalance).to.equal(initialTreasuryBalance + stakeAmount);

    // Verify vault state updated
    const vaultAccount = await program.account.vaultConfig.fetch(vaultPDA);
    expect(vaultAccount.totalStaked.toNumber()).to.equal(stakeAmount);
    expect(vaultAccount.stakersCount.toNumber()).to.equal(1);
  });

  it('Claims rewards', async () => {
    // Add rewards first
    const rewardAmount = 0.1 * LAMPORTS_PER_SOL;
    await program.methods
      .addRewards(new anchor.BN(rewardAmount))
      .accounts({
        config: vaultPDA,
        rewardsPool: rewardsPDA,
        authority: admin.publicKey,
      })
      .rpc();

    // Wait a bit to accrue rewards
    await new Promise(resolve => setTimeout(resolve, 2000));

    const [stakePDA] = await PublicKey.findProgramAddress(
      [Buffer.from("stake_position"), user.publicKey.toBuffer()],
      program.programId
    );

    const initialUserBalance = await connection.getBalance(user.publicKey);
    
    // Claim rewards
    await program.methods
      .claimRewards()
      .accounts({
        config: vaultPDA,
        stakePosition: stakePDA,
        rewardsPool: rewardsPDA,
        user: user.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    const newUserBalance = await connection.getBalance(user.publicKey);
    expect(newUserBalance).to.be.above(initialUserBalance);
  });

  it('Withdraws stake', async () => {
    const [stakePDA] = await PublicKey.findProgramAddress(
      [Buffer.from("stake_position"), user.publicKey.toBuffer()],
      program.programId
    );

    const stakePosition = await program.account.stakePosition.fetch(stakePDA);
    const withdrawAmount = stakePosition.amount.toNumber();
    
    const initialUserBalance = await connection.getBalance(user.publicKey);
    const initialTreasuryBalance = await connection.getBalance(admin.publicKey);

    // Withdraw stake
    await program.methods
      .withdrawStake(new anchor.BN(withdrawAmount))
      .accounts({
        config: vaultPDA,
        stakePosition: stakePDA,
        user: user.publicKey,
        treasury: admin.publicKey,
      })
      .signers([user])
      .rpc();

    // Verify balances changed correctly
    const newUserBalance = await connection.getBalance(user.publicKey);
    const newTreasuryBalance = await connection.getBalance(admin.publicKey);
    
    // Account for transaction fees in the check
    expect(newUserBalance).to.be.above(initialUserBalance + withdrawAmount - 0.01 * LAMPORTS_PER_SOL);
    expect(newTreasuryBalance).to.equal(initialTreasuryBalance - withdrawAmount);

    // Verify vault state updated
    const vaultAccount = await program.account.vaultConfig.fetch(vaultPDA);
    expect(vaultAccount.totalStaked.toNumber()).to.equal(0);
    expect(vaultAccount.stakersCount.toNumber()).to.equal(0);
  });
});