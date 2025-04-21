// src/models/pool.ts
// Pool data model
// - Define schema for pool data
// - Handle database interactions for pools

import mongoose, { Schema, Document } from 'mongoose';

// Define the interface for the Pool document
export interface IPool extends Document {
  name: string;
  type: string;
  apy: number;
  minStake: number;
  lockPeriod: number;
  liquidity: number;
  totalStaked?: number;
  createdAt: Date;
  updatedAt: Date;
}

// Define the schema for the Pool model
const PoolSchema: Schema = new Schema({
  name: { type: String, required: true },
  type: { type: String, required: true },
  apy: { type: Number, required: true },
  minStake: { type: Number, default: 0.1 },
  lockPeriod: { type: Number, default: 0 },
  liquidity: { type: Number, required: true, default: 0 },
  totalStaked: { type: Number, default: 0 },
}, {
  timestamps: true, // Automatically manage createdAt and updatedAt fields
});

// Create and export the Pool model
const Pool = mongoose.model<IPool>('Pool', PoolSchema);
export default Pool;