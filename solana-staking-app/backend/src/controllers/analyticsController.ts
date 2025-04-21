// src/controllers/analyticsController.ts
// Analytics controller
// - Fetch analytics overview
// - Fetch pool-specific analytics
// - Fetch user-specific analytics

import { Request, Response } from 'express';
import { logInfo, logError } from '../utils/logger';
import * as analyticsService from '../services/analyticsService';

// Get analytics overview
export const getOverview = async (req: Request, res: Response) => {
  try {
    // Get analytics overview from analytics service
    const overview = await analyticsService.getAnalyticsOverview();
    
    logInfo('Analytics overview generated');
    
    res.status(200).json({
      message: 'Analytics overview generated successfully',
      data: overview
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error generating analytics overview: ${error.message}`);
    } else {
      logError('Error generating analytics overview: Unknown error');
    }
    res.status(500).json({ message: 'Server error generating analytics overview' });
  }
};

// Get pool analytics
export const getPoolAnalytics = async (req: Request, res: Response) => {
  try {
    const { poolId } = req.params;
    
    if (!poolId) {
      return res.status(400).json({ message: 'Pool ID is required' });
    }
    
    // Get pool analytics from analytics service
    const analytics = await analyticsService.getPoolAnalytics(poolId);
    
    logInfo(`Pool analytics generated for pool: ${poolId}`);
    
    res.status(200).json({
      message: 'Pool analytics generated successfully',
      data: analytics
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error generating pool analytics: ${error.message}`);
    } else {
      logError('Error generating pool analytics: Unknown error');
    }
    res.status(500).json({ message: 'Server error generating pool analytics' });
  }
};

// Get user analytics
export const getUserAnalytics = async (req: Request, res: Response) => {
  try {
    // Get user ID from authenticated request
    const { userId } = req.user;
    
    // Get user analytics from analytics service
    const analytics = await analyticsService.getUserAnalytics(userId);
    
    logInfo(`User analytics generated for user: ${userId}`);
    
    res.status(200).json({
      message: 'User analytics generated successfully',
      data: analytics
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error generating user analytics: ${error.message}`);
    } else {
      logError('Error generating user analytics: Unknown error');
    }
    res.status(500).json({ message: 'Server error generating user analytics' });
  }
};

// Get platform statistics
export const getPlatformStats = async (req: Request, res: Response) => {
  try {
    // This would be a more advanced analytics endpoint that combines various data
    const overview = await analyticsService.getAnalyticsOverview();
    
    // Additional platform-specific metrics could be calculated here
    const platformStats = {
      ...overview,
      platformUptime: '99.9%',
      averageStakingPeriod: '45 days',
      totalRewardsDistributed: 5000000
    };
    
    logInfo('Platform statistics fetched');
    
    res.status(200).json({
      message: 'Platform statistics fetched successfully',
      data: platformStats
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error fetching platform statistics: ${error.message}`);
    } else {
      logError('Error fetching platform statistics: Unknown error');
    }
    res.status(500).json({ message: 'Server error when fetching platform statistics' });
  }
};