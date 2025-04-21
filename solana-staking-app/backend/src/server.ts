// src/server.ts
// Express server setup
// - Import express and middleware
// - Configure middleware (e.g., body-parser, CORS)
// - Set up API routes
// - Error handling middleware

import express, { Request, Response, NextFunction } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import morgan from 'morgan';
import dotenv from 'dotenv';

// Import routes
import authRoutes from './routes/auth';
import poolRoutes from './routes/pools';
import stakingRoutes from './routes/staking';
import userRoutes from './routes/user';
import analyticsRoutes from './routes/analytics';
import priceOracleRoutes from './routes/priceOracle';

// Import logger
import { logInfo, logError } from './utils/logger';

// Load environment variables
dotenv.config();

// Create Express app
const app = express();

// Set up middleware
app.use(helmet()); // Security headers
app.use(cors()); // Enable CORS
app.use(express.json()); // Parse JSON bodies
app.use(express.urlencoded({ extended: true })); // Parse URL-encoded bodies
app.use(morgan('dev')); // HTTP request logging

// Basic route
app.get('/', (req: Request, res: Response) => {
  res.json({ message: 'Welcome to Solana Staking API' });
});

// API routes
app.use('/api/auth', authRoutes);
app.use('/api/pools', poolRoutes);
app.use('/api/staking', stakingRoutes);
app.use('/api/user', userRoutes);
app.use('/api/analytics', analyticsRoutes);
app.use('/api/price-oracle', priceOracleRoutes);

// 404 route
app.use((req: Request, res: Response) => {
  res.status(404).json({ message: 'Route not found' });
});

// Error handling middleware
app.use((err: Error, req: Request, res: Response, next: NextFunction) => {
  logError(`Unhandled error: ${err.message}`);
  res.status(500).json({
    message: 'An unexpected error occurred',
    error: process.env.NODE_ENV === 'development' ? err.message : undefined
  });
});

export default app;