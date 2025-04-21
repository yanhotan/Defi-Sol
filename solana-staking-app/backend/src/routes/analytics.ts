// src/routes/analytics.ts
// Analytics routes
// - GET /analytics/overview
// - GET /analytics/pool/:id
// - GET /analytics/user/:id

import express from 'express';
import {
  getOverview,
  getPoolAnalytics,
  getUserAnalytics,
  getPlatformStats
} from '../controllers/analyticsController';
import { authenticateToken, isAdmin } from '../middleware/auth';

const router = express.Router();

// Apply authentication middleware to all analytics routes
router.use(authenticateToken);

// Get analytics overview
router.get('/overview', getOverview);

// Get platform statistics (admin only)
router.get('/platform-stats', isAdmin, getPlatformStats);

// Get pool-specific analytics
router.get('/pool/:poolId', getPoolAnalytics);

// Get user-specific analytics
router.get('/user', getUserAnalytics); // Current user's analytics
router.get('/user/:userId', isAdmin, getUserAnalytics); // Admin can view any user's analytics

export default router;