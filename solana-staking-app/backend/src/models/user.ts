// src/models/user.ts
// User data model
// - Define schema for user data
// - Handle database interactions for users

import mongoose, { Schema, Document } from 'mongoose';

// Define the interface for the User document
export interface IUser extends Document {
  userId: string;
  name: string;
  email: string;
  password: string;
  walletAddress: string;
  stakingDetails: Record<string, any>;
  isAdmin?: boolean;
  resetToken?: string;
  resetExpires?: number;
  notificationPreferences?: {
    email: boolean;
    push: boolean;
    sms: boolean;
  };
  activityLog?: {
    lastLogin: Date;
    loginHistory: Array<{
      timestamp: Date;
      ipAddress: string;
      device: string;
    }>;
    actions: Array<{
      type: string;
      timestamp: Date;
      details: Record<string, any>;
    }>;
  };
  createdAt: Date;
  updatedAt: Date;
}

// Define the schema for the User model
const UserSchema: Schema = new Schema({
  userId: { type: String, required: true, unique: true },
  name: { type: String, required: true },
  email: { type: String, required: true, unique: true },
  password: { type: String, required: true },
  walletAddress: { type: String, required: true },
  stakingDetails: { type: Object, required: false, default: {} },
  isAdmin: { type: Boolean, default: false },
  resetToken: { type: String },
  resetExpires: { type: Number },
  notificationPreferences: { 
    email: { type: Boolean, default: true },
    push: { type: Boolean, default: true },
    sms: { type: Boolean, default: false }
  },
  activityLog: {
    lastLogin: { type: Date },
    loginHistory: [{
      timestamp: { type: Date },
      ipAddress: { type: String },
      device: { type: String }
    }],
    actions: [{
      type: { type: String },
      timestamp: { type: Date, default: Date.now },
      details: { type: Object }
    }]
  }
}, {
  timestamps: true, // Automatically manage createdAt and updatedAt fields
});

// Create and export the User model
const User = mongoose.model<IUser>('User', UserSchema);
export default User;