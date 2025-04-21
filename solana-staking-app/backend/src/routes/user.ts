// src/routes/user.ts
// User-related routes
// - GET /user/profile
// - PUT /user/update
// - GET /user/positions

import express from 'express';
import {
  getUserProfile,
  updateUserProfile,
  getUserPositions,
  getUserTransactions,
  updateWalletAddress
} from '../controllers/userController';
import { authenticateToken } from '../middleware/auth';

const router = express.Router();

// Apply authentication middleware to all user routes
router.use(authenticateToken);

// Get user profile
router.get('/profile', getUserProfile);

// Update user profile
router.put('/update', updateUserProfile);

// Get user positions
router.get('/positions', getUserPositions);

// Get user transaction history
router.get('/transactions', getUserTransactions);

// Update wallet address
router.put('/wallet', updateWalletAddress);

export default router;