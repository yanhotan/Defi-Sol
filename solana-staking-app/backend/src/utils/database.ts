import mongoose from 'mongoose';
import dotenv from 'dotenv';
import { logInfo, logError } from './logger';

dotenv.config(); // Load environment variables from .env file

const MONGODB_URI = process.env.MONGODB_URI || 'mongodb://localhost:27017/solana-staking';

// Connect to MongoDB
export const connectToDatabase = async (): Promise<void> => {
  try {
    await mongoose.connect(MONGODB_URI);
    logInfo('Successfully connected to MongoDB');
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`MongoDB connection error: ${error.message}`);
    } else {
      logError('MongoDB connection error: Unknown error');
    }
    process.exit(1);
  }
};

// Disconnect from MongoDB
export const disconnectFromDatabase = async (): Promise<void> => {
  try {
    await mongoose.disconnect();
    logInfo('Successfully disconnected from MongoDB');
  } catch (error: unknown) {
    if (error instanceof Error) {
      logError(`MongoDB disconnect error: ${error.message}`);
    } else {
      logError('MongoDB disconnect error: Unknown error');
    }
  }
};

export default connectToDatabase;