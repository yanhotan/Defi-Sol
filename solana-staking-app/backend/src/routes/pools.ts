// src/routes/pools.ts
// Pool-related routes
// - GET /pools
// - POST /pools/create
// - POST /pools/deposit
// - POST /pools/withdraw

import express from 'express';
import {
  createPool,
  getPools,
  getPool,
  depositToPool,
  withdrawFromPool,
  getPoolTransactions,
  updatePool
} from '../controllers/poolController';
import { validatePoolCreation, checkValidationResults } from '../utils/validators';
import { authenticateToken, isAdmin } from '../middleware/auth';

const router = express.Router();

// Public route - Get all pools
router.get('/', getPools);

// Public route - Get pool by ID
router.get('/:poolId', getPool);

// Apply authentication middleware to all protected routes
router.use(authenticateToken);

// Admin routes - Require admin privileges
router.post('/create', isAdmin, validatePoolCreation, checkValidationResults, createPool);
router.post('/:poolId/deposit', isAdmin, depositToPool);
router.post('/:poolId/withdraw', isAdmin, withdrawFromPool);
router.put('/:poolId', isAdmin, updatePool);

// Get pool transactions - Admin only
router.get('/:poolId/transactions', isAdmin, getPoolTransactions);

export default router;