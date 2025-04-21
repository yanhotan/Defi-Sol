// src/middleware/auth.ts
// Authentication middleware
// - Verify JWT tokens
// - Protect routes that require authentication
// - Handle admin-only access

import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { logError } from '../utils/logger';
import User from '../models/user';

// Get JWT secret from environment variables or use default for development
const JWT_SECRET = process.env.JWT_SECRET || 'your-secret-key';

// Interface for JWT payload
interface JwtPayload {
  userId: string;
  email: string;
}

// Add user field to Express Request type
declare global {
  namespace Express {
    interface Request {
      user: JwtPayload;
    }
  }
}

// Middleware to authenticate token
export const authenticateToken = async (req: Request, res: Response, next: NextFunction) => {
  try {
    const token = req.header('Authorization')?.replace('Bearer ', '');
    
    if (!token) {
      return res.status(401).json({ message: 'Authentication required' });
    }
    
    try {
      const decoded = jwt.verify(token, JWT_SECRET) as JwtPayload;
      req.user = decoded;
      next();
    } catch (error: unknown) {
      if (error instanceof Error) {
        logError(`Token verification error: ${error.message}`);
      } else {
        logError('Token verification error: Unknown error');
      }
      return res.status(401).json({ message: 'Invalid or expired token' });
    }
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Authentication middleware error: ${error.message}`);
    } else {
      logError('Authentication middleware error: Unknown error');
    }
    return res.status(500).json({ message: 'Server error during authentication' });
  }
};

// Middleware to check if user is admin
export const isAdmin = async (req: Request, res: Response, next: NextFunction) => {
  try {
    const { userId } = req.user;
    
    // Find user in database
    const user = await User.findOne({ userId });
    
    if (!user) {
      return res.status(404).json({ message: 'User not found' });
    }
    
    // Check if user has admin role
    if (user.isAdmin) {
      return next();
    }
    
    return res.status(403).json({ message: 'Admin access required' });
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Admin authorization error: ${error.message}`);
    } else {
      logError('Admin authorization error: Unknown error');
    }
    return res.status(500).json({ message: 'Server error during authorization' });
  }
};