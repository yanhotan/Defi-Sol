// src/models/transaction.ts
// Transaction data model
// - Define schema for transactions
// - Handle database interactions for transactions

import mongoose, { Schema, Document } from 'mongoose';

// Define the interface for the Transaction document
export interface ITransaction extends Document {
  transactionId: string;
  originalTransactionId?: string;  // Added field for unstaking reference
  type: string;
  amount: number;
  timestamp: Date;
  userId: string;
  poolId: string;
  unlockTime?: Date;  // Added field for lock pools
}

// Define the schema for the Transaction model
const TransactionSchema: Schema = new Schema({
  transactionId: { type: String, required: true, unique: true },
  originalTransactionId: { type: String },  // Reference to original stake transaction for unstaking
  type: { type: String, required: true },
  amount: { type: Number, required: true },
  timestamp: { type: Date, required: true },
  userId: { type: String, required: true },
  poolId: { type: String, required: true },
  unlockTime: { type: Date },  // Unlock time for locked staking
});

// Create and export the Transaction model
const Transaction = mongoose.model<ITransaction>('Transaction', TransactionSchema);
export default Transaction;