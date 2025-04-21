// src/services/notificationService.ts
// Notification service
// - Send notifications to users
// - Manage notification preferences

import { logInfo, logError } from '../utils/logger';
import User from '../models/user';

// Notification types
export enum NotificationType {
  SYSTEM = 'system',
  STAKING = 'staking',
  SECURITY = 'security',
  REWARD = 'reward',
  TRANSACTION = 'transaction'
}

// Interface for notification
interface Notification {
  userId: string;
  type: NotificationType;
  title: string;
  message: string;
  timestamp: Date;
  isRead: boolean;
}

// In-memory notification storage (would be DB in production)
const notifications: Notification[] = [];

// Define notification preferences interface
interface NotificationPreferences {
  email: boolean;
  push: boolean;
  sms: boolean;
}

// Send email notification
export const sendEmailNotification = async (
  userId: string,
  subject: string,
  message: string
) => {
  try {
    // In a real implementation, this would connect to an email service
    logInfo(`Email sent to user ${userId}: ${subject}`);
    return true;
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to send email to user ${userId}: ${error.message}`);
    } else {
      logError('Failed to send email to user: Unknown error');
    }
    return false;
  }
};

// Send push notification
export const sendPushNotification = async (
  userId: string,
  title: string,
  body: string
) => {
  try {
    // In a real implementation, this would connect to a push notification service
    logInfo(`Push notification sent to user ${userId}: ${title}`);
    return true;
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to send push notification to user ${userId}: ${error.message}`);
    } else {
      logError('Failed to send push notification to user: Unknown error');
    }
    return false;
  }
};

// Update user notification preferences
export const updateNotificationPreferences = async (
  userId: string,
  preferences: NotificationPreferences
) => {
  try {
    const user = await User.findOne({ userId });
    if (!user) {
      throw new Error('User not found');
    }

    // Update user preferences in database
    user.notificationPreferences = preferences;
    await user.save();
    
    logInfo(`Updated notification preferences for user ${userId}`);
    return true;
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to update notification preferences for user ${userId}: ${error.message}`);
    } else {
      logError('Failed to update notification preferences for user: Unknown error');
    }
    return false;
  }
};

/**
 * Send a notification to a user
 */
export const sendNotification = async (
  userId: string,
  type: NotificationType,
  title: string,
  message: string
): Promise<boolean> => {
  try {
    // Create notification
    const notification: Notification = {
      userId,
      type,
      title,
      message,
      timestamp: new Date(),
      isRead: false
    };
    
    // Store notification
    notifications.push(notification);
    
    // In a real implementation, we would:
    // 1. Save to database
    // 2. Maybe send via WebSocket to connected clients
    // 3. Potentially send push notification, email, etc. based on user preferences
    
    logInfo(`Notification sent to user ${userId}: ${type} - ${title}`);
    return true;
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to send notification: ${error.message}`);
    } else {
      logError('Failed to send notification: Unknown error');
    }
    return false;
  }
};

/**
 * Get notifications for a user
 */
export const getUserNotifications = async (
  userId: string,
  limit: number = 50,
  unreadOnly: boolean = false
): Promise<Notification[]> => {
  try {
    let result = notifications.filter(n => n.userId === userId);
    
    if (unreadOnly) {
      result = result.filter(n => !n.isRead);
    }
    
    // Sort by timestamp descending (newest first)
    result.sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime());
    
    // Limit results
    return result.slice(0, limit);
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to get user notifications: ${error.message}`);
    } else {
      logError('Failed to get user notifications: Unknown error');
    }
    return [];
  }
};

/**
 * Mark notification as read
 */
export const markNotificationAsRead = async (
  userId: string,
  notificationIndex: number
): Promise<boolean> => {
  try {
    const userNotifications = notifications.filter(n => n.userId === userId);
    
    if (notificationIndex < 0 || notificationIndex >= userNotifications.length) {
      return false;
    }
    
    const notificationToUpdate = userNotifications[notificationIndex];
    notificationToUpdate.isRead = true;
    
    return true;
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Failed to mark notification as read: ${error.message}`);
    } else {
      logError('Failed to mark notification as read: Unknown error');
    }
    return false;
  }
};
