// src/utils/activityLogger.ts
// Activity logger utility
// - Track user activities
// - Update user activity logs in the database

import { logInfo, logError } from './logger';
import User from '../models/user';

/**
 * Log a user activity
 * @param userId The ID of the user
 * @param actionType The type of action performed
 * @param details Additional details about the action
 */
export const logUserActivity = async (
  userId: string, 
  actionType: string, 
  details: Record<string, any> = {}
): Promise<void> => {
  try {
    // Create activity log entry
    const activity = {
      type: actionType,
      timestamp: new Date(),
      details
    };

    // Update the user's activity log
    await User.findOneAndUpdate(
      { userId },
      { 
        $push: { 
          'activityLog.actions': activity 
        }
      }
    );

    logInfo(`User activity logged: ${userId} - ${actionType}`);
  } catch (error) {
    if (error instanceof Error) {
      logError(`Error logging user activity: ${error.message}`);
    } else {
      logError('Error logging user activity: Unknown error');
    }
  }
};

/**
 * Log a user login
 * @param userId The ID of the user
 * @param ipAddress The IP address of the user
 * @param device The device used for login
 */
export const logUserLogin = async (
  userId: string, 
  ipAddress: string, 
  device: string
): Promise<void> => {
  try {
    const loginInfo = {
      timestamp: new Date(),
      ipAddress,
      device
    };

    // Update user's login history and last login date
    await User.findOneAndUpdate(
      { userId },
      { 
        $push: { 
          'activityLog.loginHistory': loginInfo 
        },
        'activityLog.lastLogin': new Date()
      }
    );

    logInfo(`User login logged: ${userId} from ${ipAddress} on ${device}`);
  } catch (error) {
    if (error instanceof Error) {
      logError(`Error logging user login: ${error.message}`);
    } else {
      logError('Error logging user login: Unknown error');
    }
  }
};