// src/controllers/poolController.ts
// Pool controller
// - Handle pool creation
// - Handle deposits and withdrawals
// - Fetch pool details

import { Request, Response } from 'express';
import { v4 as uuidv4 } from 'uuid';
import Pool from '../models/pool';
import Transaction from '../models/transaction';
import { logInfo, logError } from '../utils/logger';

// Create a new pool
export const createPool = async (req: Request, res: Response) => {
  try {
    const { name, type, apy, minStake, lockPeriod } = req.body;
    
    // Validate required fields
    if (!name || !type || !apy) {
      return res.status(400).json({ message: 'Name, type and APY are required' });
    }
    
    // Create new pool
    const pool = new Pool({
      name,
      type,
      apy,
      minStake: minStake || 0.1, // Default min stake
      lockPeriod: lockPeriod || 0, // Default no lock period
      totalStaked: 0,
      createdAt: new Date()
    });
    
    // Save to database
    await pool.save();
    
    logInfo(`New pool created: ${name}`);
    
    res.status(201).json({
      message: 'Pool created successfully',
      pool
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error creating pool: ${error.message}`);
    } else {
      logError('Error creating pool: Unknown error');
    }
    res.status(500).json({ message: 'Server error creating pool' });
  }
};

// Get all pools
export const getPools = async (req: Request, res: Response) => {
  try {
    // Get all pools
    const pools = await Pool.find().sort({ createdAt: -1 });
    
    res.status(200).json({
      message: 'Pools retrieved successfully',
      pools
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error retrieving pools: ${error.message}`);
    } else {
      logError('Error retrieving pools: Unknown error');
    }
    res.status(500).json({ message: 'Server error retrieving pools' });
  }
};

// Get a single pool
export const getPool = async (req: Request, res: Response) => {
  try {
    const { poolId } = req.params;
    
    // Find pool by ID
    const pool = await Pool.findById(poolId);
    
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    res.status(200).json({
      message: 'Pool retrieved successfully',
      pool
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error retrieving pool: ${error.message}`);
    } else {
      logError('Error retrieving pool: Unknown error');
    }
    res.status(500).json({ message: 'Server error retrieving pool' });
  }
};

// Deposit to a pool (admin function)
export const depositToPool = async (req: Request, res: Response) => {
  try {
    const { poolId } = req.params;
    const { amount } = req.body;
    
    // Validate amount
    if (isNaN(amount) || amount <= 0) {
      return res.status(400).json({ message: 'Invalid deposit amount' });
    }
    
    // Check if pool exists
    const pool = await Pool.findById(poolId);
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    // Create transaction record
    const transaction = new Transaction({
      transactionId: uuidv4(),
      type: 'deposit',
      amount,
      timestamp: new Date(),
      userId: req.user.userId, // Admin user ID
      poolId
    });
    
    await transaction.save();
    
    // Update pool liquidity
    pool.liquidity += amount;
    await pool.save();
    
    logInfo(`Deposit of ${amount} tokens to pool ${poolId}`);
    
    res.status(200).json({
      message: 'Deposit successful',
      transaction,
      updatedPool: pool
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Deposit error: ${error.message}`);
    } else {
      logError('Deposit error: Unknown error');
    }
    res.status(500).json({ message: 'Server error during deposit' });
  }
};

// Withdraw from a pool (admin function)
export const withdrawFromPool = async (req: Request, res: Response) => {
  try {
    const { poolId } = req.params;
    const { amount } = req.body;
    
    // Validate amount
    if (isNaN(amount) || amount <= 0) {
      return res.status(400).json({ message: 'Invalid withdrawal amount' });
    }
    
    // Check if pool exists
    const pool = await Pool.findById(poolId);
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    // Check if pool has enough liquidity
    if (pool.liquidity < amount) {
      return res.status(400).json({ message: 'Insufficient liquidity in pool' });
    }
    
    // Create transaction record
    const transaction = new Transaction({
      transactionId: uuidv4(),
      type: 'withdraw',
      amount,
      timestamp: new Date(),
      userId: req.user.userId, // Admin user ID
      poolId
    });
    
    await transaction.save();
    
    // Update pool liquidity
    pool.liquidity -= amount;
    await pool.save();
    
    logInfo(`Withdrawal of ${amount} tokens from pool ${poolId}`);
    
    res.status(200).json({
      message: 'Withdrawal successful',
      transaction,
      updatedPool: pool
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Withdrawal error: ${error.message}`);
    } else {
      logError('Withdrawal error: Unknown error');
    }
    res.status(500).json({ message: 'Server error during withdrawal' });
  }
};

// Get pool transactions
export const getPoolTransactions = async (req: Request, res: Response) => {
  try {
    const { poolId } = req.params;
    
    // Check if pool exists
    const pool = await Pool.findById(poolId);
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    // Get transactions for the pool
    const transactions = await Transaction.find({ poolId }).sort({ timestamp: -1 });
    
    res.status(200).json({
      message: 'Transactions retrieved successfully',
      transactions
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error retrieving pool transactions: ${error.message}`);
    } else {
      logError('Error retrieving pool transactions: Unknown error');
    }
    res.status(500).json({ message: 'Server error when retrieving pool transactions' });
  }
};

// Update pool details (admin function)
export const updatePool = async (req: Request, res: Response) => {
  try {
    const { poolId } = req.params;
    const { name, apy, minStake, lockPeriod } = req.body;
    
    // Find pool by ID
    const pool = await Pool.findById(poolId);
    
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    // Update fields if provided
    if (name) pool.name = name;
    if (apy !== undefined) pool.apy = apy;
    if (minStake !== undefined) pool.minStake = minStake;
    if (lockPeriod !== undefined) pool.lockPeriod = lockPeriod;
    
    // Save updated pool
    await pool.save();
    
    logInfo(`Pool updated: ${poolId}`);
    
    res.status(200).json({
      message: 'Pool updated successfully',
      pool
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error updating pool: ${error.message}`);
    } else {
      logError('Error updating pool: Unknown error');
    }
    res.status(500).json({ message: 'Server error updating pool' });
  }
};

// Delete a pool
export const deletePool = async (req: Request, res: Response) => {
  try {
    const { poolId } = req.params;
    
    // Find and delete pool
    const pool = await Pool.findByIdAndDelete(poolId);
    
    if (!pool) {
      return res.status(404).json({ message: 'Pool not found' });
    }
    
    logInfo(`Pool deleted: ${poolId}`);
    
    res.status(200).json({
      message: 'Pool deleted successfully'
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error deleting pool: ${error.message}`);
    } else {
      logError('Error deleting pool: Unknown error');
    }
    res.status(500).json({ message: 'Server error deleting pool' });
  }
};
