import mongoose from 'mongoose';
import dotenv from 'dotenv';

dotenv.config();

async function testConnection() {
  console.log('Testing MongoDB connection...');
  console.log('MONGO_URI:', process.env.MONGO_URI ? 'Exists' : 'Not defined');
  
  try {
    // Set strict query to false to avoid deprecation warnings
    mongoose.set('strictQuery', false);
    
    const mongoURI = process.env.MONGO_URI || 'mongodb://localhost:27017/solana-staking';
    console.log('Attempting to connect to:', mongoURI.replace(/\/\/([^:]+):([^@]+)@/, '//$1:****@'));
    
    // Updated connection options for Mongoose 7+
    const options = {
      serverSelectionTimeoutMS: 10000, // Timeout after 10s
      socketTimeoutMS: 45000, // Close sockets after 45s of inactivity
    };
    
    await mongoose.connect(mongoURI, options);
    console.log('✅ Connected to MongoDB successfully!');
    
    // Test creating a simple document
    const testSchema = new mongoose.Schema({ name: String, timestamp: Date });
    const TestModel = mongoose.models.Test || mongoose.model('Test', testSchema);
    
    await TestModel.create({
      name: 'Connection Test',
      timestamp: new Date()
    });
    console.log('✅ Successfully inserted test document');
    
    // List all collections
    const collections = await mongoose.connection.db.listCollections().toArray();
    console.log('📋 Available collections:');
    collections.forEach(collection => {
      console.log(` - ${collection.name}`);
    });
    
    await mongoose.connection.close();
    console.log('Connection closed.');
  } catch (error) {
    console.error('❌ Error connecting to MongoDB:');
    console.error(error);
  }
}

testConnection();