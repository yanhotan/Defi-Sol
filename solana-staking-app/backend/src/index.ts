// src/index.ts
// Entry point for the backend application
// - Import necessary modules
// - Initialize the server
// - Start listening on a specified port

import app from './server';
import { logInfo, logError } from './utils/logger';
import connectDB from './utils/database';
import { initializePriceFeed } from './services/priceOracleService';

// Get port from environment variables or use default
const PORT = process.env.PORT || 5000;

// Connect to MongoDB and start server
const startServer = async () => {
  try {
    // Connect to MongoDB
    await connectDB();
    
    // Initialize price feed for commonly used tokens
    try {
      await initializePriceFeed();
      logInfo('Price feed initialized successfully');
    } catch (error: unknown) {
      // Proper error handling with type checking
      if (error instanceof Error) {
        logError(`Price feed initialization error: ${error.message}`);
      } else {
        logError('Unknown price feed initialization error');
      }
      // Continue starting the server even if price feed init fails
    }
    
    // Start the server
    app.listen(PORT, () => {
      logInfo(`Server running in ${process.env.NODE_ENV || 'development'} mode on port ${PORT}`);
      console.log(`🚀 Server started on http://localhost:${PORT}`);
    });
  } catch (error: unknown) {
    // Proper error handling with type checking
    if (error instanceof Error) {
      logError(`Server startup error: ${error.message}`);
    } else {
      logError('Unknown server startup error');
    }
    process.exit(1);
  }
};

// Start the server
startServer();

// Handle unhandled promise rejections
process.on('unhandledRejection', (err: unknown) => {
  if (err instanceof Error) {
    logError(`Unhandled Promise Rejection: ${err.message}`);
    console.error('Unhandled Promise Rejection:', err);
  } else {
    logError('Unhandled Promise Rejection with unknown error');
    console.error('Unhandled Promise Rejection with unknown error');
  }
  // Close server & exit process
  process.exit(1);
});