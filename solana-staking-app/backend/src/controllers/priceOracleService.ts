// src/controllers/priceOracleService.ts
// Price oracle controller
// - Expose price feed service through API endpoints
// - Handle price data requests

import { Request, Response } from 'express';
import { logInfo, logError } from '../utils/logger';
import { getPrice, getAllCachedPrices, refreshPrice, initializePriceFeed as initializePriceFeedService } from '../services/priceOracleService';

// Get price for a token
export const getTokenPrice = async (req: Request, res: Response) => {
  try {
    const { symbol } = req.params;
    
    if (!symbol) {
      return res.status(400).json({ message: 'Token symbol is required' });
    }
    
    const price = await getPrice(symbol);
    
    res.status(200).json({
      message: 'Price fetched successfully',
      data: { symbol, price }
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error fetching token price: ${error.message}`);
    } else {
      logError('Error fetching token price: Unknown error');
    }
    res.status(500).json({ message: 'Server error fetching token price' });
  }
};

// Get all cached prices
export const getAllPrices = async (req: Request, res: Response) => {
  try {
    const prices = getAllCachedPrices();
    
    res.status(200).json({
      message: 'Prices fetched successfully',
      data: prices
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error fetching all prices: ${error.message}`);
    } else {
      logError('Error fetching all prices: Unknown error');
    }
    res.status(500).json({ message: 'Server error fetching all prices' });
  }
};

// Refresh price for a token
export const refreshTokenPrice = async (req: Request, res: Response) => {
  try {
    const { symbol } = req.params;
    
    if (!symbol) {
      return res.status(400).json({ message: 'Token symbol is required' });
    }
    
    const price = await refreshPrice(symbol);
    
    res.status(200).json({
      message: 'Price refreshed successfully',
      data: { symbol, price }
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error refreshing token price: ${error.message}`);
    } else {
      logError('Error refreshing token price: Unknown error');
    }
    res.status(500).json({ message: 'Server error refreshing token price' });
  }
};

// Initialize price feed
export const initializePriceFeed = async (req: Request, res: Response) => {
  try {
    await initializePriceFeedService();
    
    res.status(200).json({
      message: 'Price feed initialized successfully'
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error initializing price feed: ${error.message}`);
    } else {
      logError('Error initializing price feed: Unknown error');
    }
    res.status(500).json({ message: 'Failed to initialize price feed' });
  }
};
