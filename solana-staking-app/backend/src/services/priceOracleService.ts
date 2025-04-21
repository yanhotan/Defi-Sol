// src/services/priceOracleService.ts
// Price oracle service
// - Fetch price data from external APIs
// - Cache and update price feeds

import axios from 'axios';
import { logInfo, logError } from '../utils/logger';

// Price cache to minimize external API calls
const priceCache: Record<string, { price: number; lastUpdated: Date }> = {};

// Default tokens to track
const DEFAULT_TOKENS = ['SOL', 'BTC', 'ETH', 'USDC', 'USDT'];

// API endpoints for price data
const PRICE_API_ENDPOINT = 'https://api.coingecko.com/api/v3/simple/price';

/**
 * Initialize price feed service
 */
export const initializePriceFeed = async (): Promise<void> => {
  try {
    // Fetch initial prices for default tokens
    for (const token of DEFAULT_TOKENS) {
      await refreshPrice(token);
    }
    
    logInfo('Price feed initialized successfully');
    
    // Set up periodic price updates (every 5 minutes)
    setInterval(async () => {
      for (const token of DEFAULT_TOKENS) {
        await refreshPrice(token);
      }
      logInfo('Price feed updated');
    }, 5 * 60 * 1000);
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to initialize price feed: ${error.message}`);
    } else {
      logError('Failed to initialize price feed: Unknown error');
    }
  }
};

/**
 * Fetch current price for a token
 */
export const getPrice = async (symbol: string): Promise<number> => {
  try {
    // Check cache first
    const cachedData = priceCache[symbol];
    const cacheExpiry = 5 * 60 * 1000; // 5 minutes
    
    // If we have a recent cache entry, return it
    if (cachedData && (new Date().getTime() - cachedData.lastUpdated.getTime()) < cacheExpiry) {
      return cachedData.price;
    }
    
    // Otherwise fetch fresh data
    return await refreshPrice(symbol);
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to get price for ${symbol}: ${error.message}`);
    } else {
      logError(`Failed to get price for ${symbol}: Unknown error`);
    }
    throw error;
  }
};

/**
 * Refresh price data for a token
 */
export const refreshPrice = async (symbol: string): Promise<number> => {
  try {
    // Normalize symbol
    const normalizedSymbol = symbol.toLowerCase();
    
    // Fetch price from API
    const response = await axios.get(PRICE_API_ENDPOINT, {
      params: {
        ids: normalizedSymbol,
        vs_currencies: 'usd'
      }
    });
    
    // Extract price from response
    const price = response.data[normalizedSymbol]?.usd;
    
    if (!price) {
      throw new Error(`Price data not available for ${symbol}`);
    }
    
    // Update cache
    priceCache[symbol] = {
      price,
      lastUpdated: new Date()
    };
    
    return price;
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to refresh price for ${symbol}: ${error.message}`);
    } else {
      logError(`Failed to refresh price for ${symbol}: Unknown error`);
    }
    
    // If we have a cached price, return that instead of failing
    if (priceCache[symbol]) {
      return priceCache[symbol].price;
    }
    
    throw error;
  }
};

/**
 * Get all cached prices
 */
export const getAllCachedPrices = (): Record<string, { price: number; lastUpdated: string }> => {
  try {
    const result: Record<string, { price: number; lastUpdated: string }> = {};
    
    for (const [symbol, data] of Object.entries(priceCache)) {
      result[symbol] = {
        price: data.price,
        lastUpdated: data.lastUpdated.toISOString()
      };
    }
    
    return result;
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to get cached prices: ${error.message}`);
    } else {
      logError('Failed to get cached prices: Unknown error');
    }
    return {};
  }
};