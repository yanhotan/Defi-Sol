// stakingController.ts
// Staking controller
// - Handle staking operations
// - Handle unstaking operations
// - Fetch staking details for a user
// - Calculate staking rewards
// - Manage staking pool interactions

import { Request, Response } from 'express';
import { v4 as uuidv4 } from 'uuid';
import User from '../models/user';
import Pool from '../models/pool';
import Transaction from '../models/transaction';
import { logInfo, logError } from '../utils/logger';
import { NotificationType, sendNotification } from '../services/notificationService';

// Stake tokens to a pool
export const stakeTokens = async (req: Request, res: Response) => {
  try {
    const { poolId } = req.params;
    const { amount } = req.body;
    const userId = req.user.userId;
    
    // Validate amount
    if (!amount || isNaN(amount) || amount <= 0) {
      return res.status(400).json({ message: 'Invalid stake amount' });
    }
    
    // Find pool by ID
    const pool = await Pool.findById(poolId);
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    // Check minimum stake amount
    if (amount < pool.minStake) {
      return res.status(400).json({ message: `Minimum stake amount is ${pool.minStake}` });
    }
    
    // Find user
    const user = await User.findById(userId);
    if (!user) {
      return res.status(404).json({ message: 'User not found' });
    }
    
    // Create transaction record
    const transaction = new Transaction({
      transactionId: uuidv4(),
      type: 'stake',
      amount,
      timestamp: new Date(),
      userId,
      poolId,
      // If it's a lock pool, calculate unlock time
      unlockTime: pool.lockPeriod ? new Date(Date.now() + pool.lockPeriod * 86400000) : null
    });
    
    await transaction.save();
    
    // Update pool's total staked amount
    pool.totalStaked = (pool.totalStaked || 0) + amount;
    await pool.save();
    
    logInfo(`User ${userId} staked ${amount} tokens to pool ${poolId}`);
    
    res.status(200).json({
      message: 'Stake successful',
      transaction,
      pool
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Staking error: ${error.message}`);
    } else {
      logError('Staking error: Unknown error');
    }
    res.status(500).json({ message: 'Server error during staking' });
  }
};

// Unstake tokens from a pool
export const unstakeTokens = async (req: Request, res: Response) => {
  try {
    const { transactionId } = req.params;
    const userId = req.user.userId;
    
    // Find the original stake transaction
    const stakeTransaction = await Transaction.findOne({ transactionId, type: 'stake', userId });
    
    if (!stakeTransaction) {
      return res.status(404).json({ message: 'Stake transaction not found' });
    }
    
    // Check if already unstaked
    const unstakeCheck = await Transaction.findOne({ 
      originalTransactionId: transactionId, 
      type: 'unstake',
      userId
    });
    
    if (unstakeCheck) {
      return res.status(400).json({ message: 'Tokens already unstaked' });
    }
    
    // Check if lockPeriod is over for lock pools
    if (stakeTransaction.unlockTime && new Date() < stakeTransaction.unlockTime) {
      return res.status(400).json({ 
        message: 'Tokens are still locked',
        unlockTime: stakeTransaction.unlockTime
      });
    }
    
    // Find pool by ID
    const pool = await Pool.findById(stakeTransaction.poolId);
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    // Create unstake transaction
    const unstakeTransaction = new Transaction({
      transactionId: uuidv4(),
      originalTransactionId: transactionId,
      type: 'unstake',
      amount: stakeTransaction.amount,
      timestamp: new Date(),
      userId,
      poolId: stakeTransaction.poolId
    });
    
    await unstakeTransaction.save();
    
    // Update pool's total staked amount
    pool.totalStaked = Math.max(0, (pool.totalStaked || 0) - stakeTransaction.amount);
    await pool.save();
    
    logInfo(`User ${userId} unstaked ${stakeTransaction.amount} tokens from pool ${stakeTransaction.poolId}`);
    
    res.status(200).json({
      message: 'Unstake successful',
      transaction: unstakeTransaction,
      pool
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Unstaking error: ${error.message}`);
    } else {
      logError('Unstaking error: Unknown error');
    }
    res.status(500).json({ message: 'Server error during unstaking' });
  }
};

// Get user's staking transactions
export const getUserStakes = async (req: Request, res: Response) => {
  try {
    const userId = req.user.userId;
    
    // Find all stake transactions for the user
    const stakeTransactions = await Transaction.find({ 
      userId, 
      type: 'stake'
    }).sort({ timestamp: -1 });
    
    // Find all unstake transactions for the user
    const unstakeTransactions = await Transaction.find({
      userId,
      type: 'unstake'
    });
    
    // Create a map of unstaked transactions for easy lookup
    const unstakeMap: Record<string, any> = {};
    unstakeTransactions.forEach(transaction => {
      if (transaction.originalTransactionId) {
        unstakeMap[transaction.originalTransactionId] = transaction;
      }
    });
    
    // Enrich stake transactions with unstake status
    const enrichedTransactions = stakeTransactions.map(transaction => {
      const unstakeInfo = unstakeMap[transaction.transactionId];
      return {
        ...transaction.toObject(),
        isUnstaked: !!unstakeInfo,
        unstakeTime: unstakeInfo ? unstakeInfo.timestamp : null
      };
    });
    
    res.status(200).json({
      message: 'User staking transactions retrieved successfully',
      stakes: enrichedTransactions
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error retrieving user stakes: ${error.message}`);
    } else {
      logError('Error retrieving user stakes: Unknown error');
    }
    res.status(500).json({ message: 'Server error retrieving user stakes' });
  }
};

// Calculate rewards for user's stake
export const calculateRewards = async (req: Request, res: Response) => {
  try {
    const { transactionId } = req.params;
    const userId = req.user.userId;
    
    // Find the stake transaction
    const stakeTransaction = await Transaction.findOne({ 
      transactionId, 
      type: 'stake',
      userId
    });
    
    if (!stakeTransaction) {
      return res.status(404).json({ message: 'Stake transaction not found' });
    }
    
    // Find pool to get APY
    const pool = await Pool.findById(stakeTransaction.poolId);
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    // Check if already unstaked
    const unstakeCheck = await Transaction.findOne({ 
      originalTransactionId: transactionId, 
      type: 'unstake' 
    });
    
    // Calculate staking duration in days
    const startTime = new Date(stakeTransaction.timestamp).getTime();
    const endTime = unstakeCheck 
      ? new Date(unstakeCheck.timestamp).getTime() 
      : new Date().getTime();
    
    const stakingDurationMs = endTime - startTime;
    const stakingDurationDays = stakingDurationMs / (1000 * 60 * 60 * 24);
    
    // Calculate rewards based on APY for the staking period
    const rewards = (stakeTransaction.amount * pool.apy / 100) * (stakingDurationDays / 365);
    
    res.status(200).json({
      message: 'Rewards calculated successfully',
      stakeAmount: stakeTransaction.amount,
      stakingDurationDays: parseFloat(stakingDurationDays.toFixed(2)),
      apy: pool.apy,
      rewards: parseFloat(rewards.toFixed(6)),
      isUnstaked: !!unstakeCheck
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error calculating rewards: ${error.message}`);
    } else {
      logError('Error calculating rewards: Unknown error');
    }
    res.status(500).json({ message: 'Server error calculating rewards' });
  }
};