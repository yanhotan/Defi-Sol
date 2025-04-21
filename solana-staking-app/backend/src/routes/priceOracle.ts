// src/routes/priceOracle.ts
// Price oracle routes
// - GET /price-oracle/price/:symbol
// - GET /price-oracle/prices
// - POST /price-oracle/refresh/:symbol
// - POST /price-oracle/initialize

import express from 'express';
import {
  getTokenPrice,
  getAllPrices,
  refreshTokenPrice,
  initializePriceFeed
} from '../controllers/priceOracleService';
import { authenticateToken, isAdmin } from '../middleware/auth';

const router = express.Router();

// Public routes
router.get('/price/:symbol', getTokenPrice);
router.get('/prices', getAllPrices);

// Protected routes (require authentication)
router.use(authenticateToken);

// Admin only routes
router.post('/refresh/:symbol', isAdmin, refreshTokenPrice);
router.post('/initialize', isAdmin, initializePriceFeed);

export default router;