// src/services/analyticsService.ts
// Analytics service
// - Aggregate data for analytics
// - Generate reports for pools and users

import { logInfo, logError } from '../utils/logger';
import Pool from '../models/pool';
import Transaction from '../models/transaction';
import User from '../models/user';

// Get analytics overview
export const getAnalyticsOverview = async () => {
  try {
    // Get total users
    const totalUsers = await User.countDocuments();
    
    // Get total pools
    const totalPools = await Pool.countDocuments();
    
    // Get total liquidity across all pools
    const pools = await Pool.find();
    const totalLiquidity = pools.reduce((sum, pool) => sum + pool.liquidity, 0);
    
    // Get recent transactions
    const recentTransactions = await Transaction.find()
      .sort({ timestamp: -1 })
      .limit(10);
    
    return {
      totalUsers,
      totalPools,
      totalLiquidity,
      recentTransactions
    };
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error generating analytics overview: ${error.message}`);
    } else {
      logError('Error generating analytics overview: Unknown error');
    }
    throw error;
  }
};

// Get pool analytics
export const getPoolAnalytics = async (poolId: string) => {
  try {
    // Get pool details
    const pool = await Pool.findById(poolId);
    if (!pool) {
      throw new Error(`Pool not found: ${poolId}`);
    }
    
    // Get transactions for this pool
    const transactions = await Transaction.find({ poolId })
      .sort({ timestamp: -1 });
    
    // Calculate pool metrics
    const depositTransactions = transactions.filter(tx => tx.type === 'deposit' || tx.type === 'stake');
    const withdrawalTransactions = transactions.filter(tx => tx.type === 'withdraw' || tx.type === 'unstake');
    
    const totalDeposited = depositTransactions.reduce((sum, tx) => sum + tx.amount, 0);
    const totalWithdrawn = withdrawalTransactions.reduce((sum, tx) => sum + tx.amount, 0);
    
    // Count unique users who have interacted with this pool
    const uniqueUsers = new Set();
    transactions.forEach(tx => uniqueUsers.add(tx.userId));
    
    return {
      poolId,
      name: pool.name,
      type: pool.type,
      liquidity: pool.liquidity,
      totalDeposited,
      totalWithdrawn,
      uniqueUsers: uniqueUsers.size,
      recentTransactions: transactions.slice(0, 10)
    };
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error generating pool analytics: ${error.message}`);
    } else {
      logError('Error generating pool analytics: Unknown error');
    }
    throw error;
  }
};

// Get user analytics
export const getUserAnalytics = async (userId: string) => {
  try {
    // Get user details
    const user = await User.findOne({ userId });
    if (!user) {
      throw new Error(`User not found: ${userId}`);
    }
    
    // Get all transactions for this user
    const transactions = await Transaction.find({ userId })
      .sort({ timestamp: -1 });
    
    // Calculate user metrics
    const stakingTransactions = transactions.filter(tx => tx.type === 'stake');
    const unstakingTransactions = transactions.filter(tx => tx.type === 'unstake');
    
    const totalStaked = stakingTransactions.reduce((sum, tx) => sum + tx.amount, 0);
    const totalUnstaked = unstakingTransactions.reduce((sum, tx) => sum + tx.amount, 0);
    
    // Get current staking positions
    const currentStaked = totalStaked - totalUnstaked;
    
    // Count unique pools this user has interacted with
    const uniquePools = new Set();
    transactions.forEach(tx => uniquePools.add(tx.poolId));
    
    return {
      userId,
      totalStaked,
      totalUnstaked,
      currentStaked,
      uniquePools: uniquePools.size,
      recentTransactions: transactions.slice(0, 10)
    };
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error generating user analytics: ${error.message}`);
    } else {
      logError('Error generating user analytics: Unknown error');
    }
    throw error;
  }
};