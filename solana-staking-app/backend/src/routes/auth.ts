// src/routes/auth.ts
// Authentication routes
// - POST /login
// - POST /register
// - POST /logout

import express from 'express';
import { 
  login, 
  register, 
  logout, 
  requestPasswordReset,
  resetPassword,
  verifyAuth
} from '../controllers/authController';
import { validateUserRegistration, checkValidationResults } from '../utils/validators';

const router = express.Router();

// Register new user
router.post('/register', validateUserRegistration, checkValidationResults, register);

// Login user
router.post('/login', login);

// Logout user
router.post('/logout', logout);

// Request password reset
router.post('/reset-password-request', requestPasswordReset);

// Reset password with token
router.post('/reset-password', resetPassword);

// Verify authentication status
router.get('/verify', verifyAuth);

export default router;