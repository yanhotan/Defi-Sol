// authController.ts
// Authentication controller
// - Handle user login
// - Handle user registration
// - Handle user logout
// - Manage password reset functionality
// - Verify user authentication status

import { Request, Response } from 'express';
import * as bcrypt from 'bcryptjs';
import * as jwt from 'jsonwebtoken';
import { v4 as uuidv4 } from 'uuid';
import User from '../models/user';
import { logInfo, logError } from '../utils/logger';
import { NotificationType, sendNotification } from '../services/notificationService';
import { isValidEmail, isStrongPassword } from '../utils/validators';

// Environment variables
const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key';
const JWT_EXPIRES_IN = process.env.JWT_EXPIRES_IN || '24h';

// Register a new user
export const register = async (req: Request, res: Response) => {
  try {
    const { name, email, password, walletAddress } = req.body;

    // Validate input
    if (!email || !password || !name) {
      return res.status(400).json({ message: 'All fields are required' });
    }

    // Validate email format
    if (!isValidEmail(email)) {
      return res.status(400).json({ message: 'Invalid email format' });
    }

    // Validate password strength
    if (!isStrongPassword(password)) {
      return res.status(400).json({
        message: 'Password must be at least 8 characters with uppercase, lowercase, and numbers'
      });
    }

    // Check if user already exists
    const existingUser = await User.findOne({ email });
    if (existingUser) {
      return res.status(400).json({ message: 'User already exists with this email' });
    }

    // Hash the password
    const salt = await bcrypt.genSalt(10);
    const hashedPassword = await bcrypt.hash(password, salt);

    // Create new user
    const userId = uuidv4();
    const newUser = new User({
      userId,
      name,
      email,
      password: hashedPassword,
      walletAddress
    });

    await newUser.save();

    // Generate JWT token with fixed type handling
    const token = jwt.sign(
      { userId: newUser.userId, email: newUser.email },
      JWT_SECRET,
      { expiresIn: JWT_EXPIRES_IN } as jwt.SignOptions
    );

    // Send welcome notification
    await sendNotification(
      userId,
      NotificationType.SYSTEM,
      'Welcome to Solana Staking App',
      'Thank you for registering! Start staking to earn rewards.'
    );

    logInfo(`User registered: ${email}`);

    res.status(201).json({
      message: 'User registered successfully',
      token,
      user: {
        userId: newUser.userId,
        name: newUser.name,
        email: newUser.email,
        walletAddress: newUser.walletAddress
      }
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error during registration: ${error.message}`);
    } else {
      logError('Error during registration: Unknown error');
    }
    res.status(500).json({ message: 'Server error during registration' });
  }
};

// Login user
export const login = async (req: Request, res: Response) => {
  try {
    const { email, password } = req.body;

    // Validate input
    if (!email || !password) {
      return res.status(400).json({ message: 'Email and password are required' });
    }

    // Check if user exists
    const user = await User.findOne({ email });
    if (!user) {
      return res.status(400).json({ message: 'Invalid email or password' });
    }

    // Validate password
    const isPasswordValid = await bcrypt.compare(password, user.password);
    if (!isPasswordValid) {
      return res.status(400).json({ message: 'Invalid email or password' });
    }

    // Generate JWT token with fixed type handling
    const token = jwt.sign(
      { userId: user.userId, email: user.email },
      JWT_SECRET,
      { expiresIn: JWT_EXPIRES_IN } as jwt.SignOptions
    );

    logInfo(`User logged in: ${email}`);

    res.status(200).json({
      message: 'Login successful',
      token,
      user: {
        userId: user.userId,
        name: user.name,
        email: user.email,
        walletAddress: user.walletAddress
      }
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error during login: ${error.message}`);
    } else {
      logError('Error during login: Unknown error');
    }
    res.status(500).json({ message: 'Server error during login' });
  }
};

// Logout user
export const logout = async (req: Request, res: Response) => {
  // JWT tokens are stateless, so we just return success
  // In a real implementation, you might want to add the token to a blacklist
  res.status(200).json({ message: 'Logout successful' });
};

// Request password reset
export const requestPasswordReset = async (req: Request, res: Response) => {
  try {
    const { email } = req.body;

    // Check if user exists
    const user = await User.findOne({ email });
    if (!user) {
      return res.status(404).json({ message: 'User not found' });
    }

    // Generate reset token
    const resetToken = uuidv4();
    const resetExpires = Date.now() + 3600000; // 1 hour

    // Save reset token to user
    user.resetToken = resetToken;
    user.resetExpires = resetExpires;
    await user.save();

    // Send password reset notification
    await sendNotification(
      user.userId,
      NotificationType.SECURITY,
      'Password Reset Request',
      'A password reset has been requested for your account. If this was not you, please secure your account.'
    );

    logInfo(`Password reset requested for: ${email}`);

    res.status(200).json({
      message: 'Password reset link has been sent to your email'
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error during password reset request: ${error.message}`);
    } else {
      logError('Error during password reset request: Unknown error');
    }
    res.status(500).json({ message: 'Server error during password reset request' });
  }
};

// Reset password with token
export const resetPassword = async (req: Request, res: Response) => {
  try {
    const { resetToken, newPassword } = req.body;

    // Find user with the reset token
    const user = await User.findOne({
      resetToken,
      resetExpires: { $gt: Date.now() }
    });

    if (!user) {
      return res.status(400).json({ message: 'Invalid or expired reset token' });
    }

    // Hash the new password
    const salt = await bcrypt.genSalt(10);
    user.password = await bcrypt.hash(newPassword, salt);
    user.resetToken = undefined;
    user.resetExpires = undefined;
    await user.save();

    // Send password updated notification
    await sendNotification(
      user.userId,
      NotificationType.SECURITY,
      'Password Updated',
      'Your password has been updated successfully.'
    );

    logInfo(`Password reset completed for: ${user.email}`);

    res.status(200).json({ message: 'Password reset successful' });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error during password reset: ${error.message}`);
    } else {
      logError('Error during password reset: Unknown error');
    }
    res.status(500).json({ message: 'Server error during password reset' });
  }
};

// Verify authentication status
export const verifyAuth = async (req: Request, res: Response) => {
  try {
    // The authentication middleware will have already verified the token
    // and added the user to the request object
    const userId = req.user.userId;

    const user = await User.findOne({ userId });
    if (!user) {
      return res.status(404).json({ message: 'User not found' });
    }

    res.status(200).json({
      authenticated: true,
      user: {
        userId: user.userId,
        name: user.name,
        email: user.email,
        walletAddress: user.walletAddress
      }
    });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Error during auth verification: ${error.message}`);
    } else {
      logError('Error during auth verification: Unknown error');
    }
    res.status(401).json({ authenticated: false });
  }
};