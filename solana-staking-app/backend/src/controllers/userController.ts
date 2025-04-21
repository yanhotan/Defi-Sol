// src/controllers/userController.ts
// User controller
// - Handle user profile updates
// - Fetch user positions
// - Manage user-related operations

import { Request, Response } from 'express';
import User from '../models/user';
import Transaction from '../models/transaction';
import { logInfo, logError } from '../utils/logger';
import { isValidPublicKey, isValidEmail } from '../utils/validators';

// Get user profile
export const getUserProfile = async (req: Request, res: Response) => {
  try {
    const userId = req.user?.userId;

    if (!userId) {
      return res.status(401).json({ message: 'Unauthorized' });
    }

    const user = await User.findOne({ userId }).select('-password -resetToken -resetExpires');
    if (!user) {
      return res.status(404).json({ message: 'User not found' });
    }

    logInfo(`Profile fetched for user: ${userId}`);

    res.status(200).json({
      message: 'User profile fetched successfully',
      user
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error fetching user profile: ${error.message}`);
      res.status(500).json({ message: error.message });
    } else {
      logError('Error fetching user profile: Unknown error');
      res.status(500).json({ message: 'An unknown error occurred' });
    }
  }
};

// Update user profile
export const updateUserProfile = async (req: Request, res: Response) => {
  try {
    const userId = req.user?.userId;
    const { name, email, walletAddress } = req.body;

    if (!userId) {
      return res.status(401).json({ message: 'Unauthorized' });
    }

    // Validate inputs
    if (email && !isValidEmail(email)) {
      return res.status(400).json({ message: 'Invalid email format' });
    }

    if (walletAddress && !isValidPublicKey(walletAddress)) {
      return res.status(400).json({ message: 'Invalid wallet address' });
    }

    const updateData: Record<string, string> = {};
    if (name) updateData.name = name;
    if (email) updateData.email = email;
    if (walletAddress) updateData.walletAddress = walletAddress;

    const updatedUser = await User.findOneAndUpdate(
      { userId },
      { $set: updateData },
      { new: true }
    ).select('-password -resetToken -resetExpires');

    if (!updatedUser) {
      return res.status(404).json({ message: 'User not found' });
    }

    logInfo(`Profile updated for user: ${userId}`);

    res.status(200).json({
      message: 'User profile updated successfully',
      user: {
        userId: updatedUser.userId,
        name: updatedUser.name,
        email: updatedUser.email,
        walletAddress: updatedUser.walletAddress
      }
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error updating user profile: ${error.message}`);
      res.status(500).json({ message: error.message });
    } else {
      logError('Error updating user profile: Unknown error');
      res.status(500).json({ message: 'An unknown error occurred' });
    }
  }
};

// Get user positions (staking positions across all pools)
export const getUserPositions = async (req: Request, res: Response) => {
  try {
    const userId = req.user?.userId;

    if (!userId) {
      return res.status(401).json({ message: 'Unauthorized' });
    }

    const user = await User.findOne({ userId }).populate('stakingPositions');
    if (!user) {
      return res.status(404).json({ message: 'User not found' });
    }

    // Check if user has any staking positions
    if (!user.stakingDetails || Object.keys(user.stakingDetails).length === 0) {
      return res.status(200).json({
        message: 'No staking positions found',
        positions: []
      });
    }

    logInfo(`Positions fetched for user: ${userId}`);

    res.status(200).json({
      message: 'User positions fetched successfully',
      positions: user.stakingDetails
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error fetching user positions: ${error.message}`);
      res.status(500).json({ message: error.message });
    } else {
      logError('Error fetching user positions: Unknown error');
      res.status(500).json({ message: 'An unknown error occurred' });
    }
  }
};

// Get user transaction history
export const getUserTransactions = async (req: Request, res: Response) => {
  try {
    const userId = req.user?.userId;
    const { limit = 10, skip = 0 } = req.query;

    if (!userId) {
      return res.status(401).json({ message: 'Unauthorized' });
    }

    // Validate pagination parameters
    const limitNum = Math.min(parseInt(limit as string) || 10, 100);
    const skipNum = parseInt(skip as string) || 0;

    // Find transactions for the user
    const transactions = await Transaction.find({ userId })
      .sort({ timestamp: -1 })
      .skip(skipNum)
      .limit(limitNum);

    const total = await Transaction.countDocuments({ userId });

    logInfo(`Transactions fetched for user: ${userId}`);

    res.status(200).json({
      message: 'User transactions fetched successfully',
      transactions,
      pagination: {
        total,
        limit: limitNum,
        skip: skipNum
      }
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error fetching user transactions: ${error.message}`);
      res.status(500).json({ message: error.message });
    } else {
      logError('Error fetching user transactions: Unknown error');
      res.status(500).json({ message: 'An unknown error occurred' });
    }
  }
};

// Update wallet address
export const updateWalletAddress = async (req: Request, res: Response) => {
  try {
    const userId = req.user?.userId;
    const { walletAddress } = req.body;

    if (!userId) {
      return res.status(401).json({ message: 'Unauthorized' });
    }

    if (!walletAddress) {
      return res.status(400).json({ message: 'Wallet address is required' });
    }

    if (!isValidPublicKey(walletAddress)) {
      return res.status(400).json({ message: 'Invalid wallet address' });
    }

    const user = await User.findOne({ userId });
    if (!user) {
      return res.status(404).json({ message: 'User not found' });
    }

    user.walletAddress = walletAddress;
    await user.save();

    logInfo(`Wallet address updated for user: ${userId}`);

    res.status(200).json({
      message: 'Wallet address updated successfully',
      walletAddress
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error updating wallet address: ${error.message}`);
      res.status(500).json({ message: error.message });
    } else {
      logError('Error updating wallet address: Unknown error');
      res.status(500).json({ message: 'An unknown error occurred' });
    }
  }
};

// Update notification preferences
export const updateNotificationPreferences = async (req: Request, res: Response) => {
  try {
    const userId = req.user?.userId;
    const { enableEmailNotifications, enablePushNotifications } = req.body;

    if (!userId) {
      return res.status(401).json({ message: 'Unauthorized' });
    }

    // Ensure the values are boolean
    const emailPref = typeof enableEmailNotifications === 'boolean' ? enableEmailNotifications : undefined;
    const pushPref = typeof enablePushNotifications === 'boolean' ? enablePushNotifications : undefined;

    const user = await User.findOne({ userId });
    if (!user) {
      return res.status(404).json({ message: 'User not found' });
    }

    // Update notification preferences
    if (emailPref !== undefined) {
      if (!user.notificationPreferences) {
        user.notificationPreferences = { email: emailPref, push: true, sms: false };
      } else {
        user.notificationPreferences.email = emailPref;
      }
    }

    if (pushPref !== undefined) {
      if (!user.notificationPreferences) {
        user.notificationPreferences = { email: true, push: pushPref, sms: false };
      } else {
        user.notificationPreferences.push = pushPref;
      }
    }

    await user.save();

    logInfo(`Notification preferences updated for user: ${userId}`);

    res.status(200).json({
      message: 'Notification preferences updated successfully',
      notificationPreferences: user.notificationPreferences
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error updating notification preferences: ${error.message}`);
      res.status(500).json({ message: error.message });
    } else {
      logError('Error updating notification preferences: Unknown error');
      res.status(500).json({ message: 'An unknown error occurred' });
    }
  }
};