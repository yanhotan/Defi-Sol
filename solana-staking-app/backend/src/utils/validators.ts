// src/utils/validators.ts
// Validators utility
// - Define validation functions for inputs
// - Validate request payloads and query parameters

import { body, param, query, validationResult } from 'express-validator';
import { PublicKey } from '@solana/web3.js';
import { logError } from './logger';
import { Request, Response, NextFunction } from 'express';

// Validation rules for user registration
export const validateUserRegistration = [
  body('email').isEmail().withMessage('Invalid email address'),
  body('password').isLength({ min: 6 }).withMessage('Password must be at least 6 characters long'),
  body('name').notEmpty().withMessage('Name is required'),
];

// Validation rules for pool creation
export const validatePoolCreation = [
  body('name').notEmpty().withMessage('Pool name is required'),
  body('type').isIn(['basic', 'lending', 'lock']).withMessage('Invalid pool type'),
  body('liquidity').isNumeric().withMessage('Liquidity must be a number'),
];

// Validation rules for staking operations
export const validateStaking = [
  body('amount').isNumeric().withMessage('Staking amount must be a number'),
  body('poolId').notEmpty().withMessage('Pool ID is required'),
];

// Middleware to check validation results
export const checkValidationResults = (req: Request, res: Response, next: NextFunction) => {
  const errors = validationResult(req);
  if (!errors.isEmpty()) {
    return res.status(400).json({ errors: errors.array() });
  }
  next();
};

// Validate Solana public key
export const isValidPublicKey = (address: string): boolean => {
  try {
    new PublicKey(address);
    return true;
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Invalid public key: ${error.message}`);
    } else {
      logError('Invalid public key: Unknown error');
    }
    return false;
  }
};

// Validate email format
export const isValidEmail = (email: string): boolean => {
  try {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Email validation error: ${error.message}`);
    } else {
      logError('Email validation error: Unknown error');
    }
    return false;
  }
};

// Validate password strength
export const isStrongPassword = (password: string): boolean => {
  try {
    // At least 8 characters
    // At least one uppercase letter
    // At least one lowercase letter
    // At least one number
    const passwordRegex = /^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)[A-Za-z\d@$!%*?&]{8,}$/;
    return passwordRegex.test(password);
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Password validation error: ${error.message}`);
    } else {
      logError('Password validation error: Unknown error');
    }
    return false;
  }
};

// Validate stake/unstake amount
export const isValidAmount = (amount: number): boolean => {
  try {
    return amount > 0 && isFinite(amount);
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`Amount validation error: ${error.message}`);
    } else {
      logError('Amount validation error: Unknown error');
    }
    return false;
  }
};