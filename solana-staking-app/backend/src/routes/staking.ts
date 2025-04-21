// src/routes/staking.ts
// Staking routes
// - POST /staking/stake/:poolId: Stake tokens
// - POST /staking/unstake/:transactionId: Unstake tokens
// - GET /staking/user-stakes: Fetch user's staking transactions
// - GET /staking/calculate-rewards/:transactionId: Calculate rewards for a stake

import express from 'express';
import {
  stakeTokens,
  unstakeTokens,
  getUserStakes,
  calculateRewards
} from '../controllers/stakingController';
import { validateStaking, checkValidationResults } from '../utils/validators';
import { authenticateToken } from '../middleware/auth';

const router = express.Router();

// Apply authentication middleware to all staking routes
router.use(authenticateToken);

// Stake tokens
router.post('/stake/:poolId', validateStaking, checkValidationResults, stakeTokens);

// Unstake tokens
router.post('/unstake/:transactionId', unstakeTokens);

// Get user stakes
router.get('/user-stakes', getUserStakes);

// Calculate rewards
router.get('/calculate-rewards/:transactionId', calculateRewards);

export default router;