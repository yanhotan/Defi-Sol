import request from 'supertest';
import mongoose from 'mongoose';
import app from '../src/server';
import User from '../src/models/user';
import Pool from '../src/models/pool';
import dotenv from 'dotenv';
import { describe, test, expect, beforeAll, afterAll } from '@jest/globals';

dotenv.config();

let authToken: string;
let testUserId: string;
let testPoolId: string;

beforeAll(async () => {
  // Connect to test database
  const mongoURI = process.env.MONGO_URI_TEST || process.env.MONGO_URI || 'mongodb://localhost:27017/solana-staking-test';
  await mongoose.connect(mongoURI);
  
  // Clear test data
  await User.deleteMany({});
  await Pool.deleteMany({});
});

afterAll(async () => {
  // Disconnect from database
  await mongoose.connection.close();
});

describe('Authentication API', () => {
  test('Should register a new user', async () => {
    const res = await request(app)
      .post('/api/auth/register')
      .send({
        name: 'Test User',
        email: 'test@example.com',
        password: 'password123',
        walletAddress: '9XyMGZfFcfLi3ugiV3FnBLM9M7EUBk7s3oHcxJ1Fh9f9'
      });
    
    expect(res.statusCode).toEqual(201);
    expect(res.body).toHaveProperty('token');
    expect(res.body).toHaveProperty('user');
    expect(res.body.user.email).toEqual('test@example.com');
    
    testUserId = res.body.user.userId;
  });
  
  test('Should login with correct credentials', async () => {
    const res = await request(app)
      .post('/api/auth/login')
      .send({
        email: 'test@example.com',
        password: 'password123'
      });
    
    expect(res.statusCode).toEqual(200);
    expect(res.body).toHaveProperty('token');
    authToken = res.body.token;
  });
  
  test('Should reject login with incorrect credentials', async () => {
    const res = await request(app)
      .post('/api/auth/login')
      .send({
        email: 'test@example.com',
        password: 'wrongpassword'
      });
    
    expect(res.statusCode).toEqual(400);
  });
});

describe('Pool API', () => {
  test('Should create a new pool (admin only)', async () => {
    // First make the test user an admin
    const user = await User.findOne({ userId: testUserId });
    if (user) {
      user.isAdmin = true;
      await user.save();
    }
    
    const res = await request(app)
      .post('/api/pools/create')
      .set('Authorization', `Bearer ${authToken}`)
      .send({
        name: 'Test Pool',
        type: 'basic',
        liquidity: 1000
      });
    
    expect(res.statusCode).toEqual(201);
    expect(res.body).toHaveProperty('pool');
    expect(res.body.pool.name).toEqual('Test Pool');
    
    testPoolId = res.body.pool._id;
  });
  
  test('Should get all pools', async () => {
    const res = await request(app)
      .get('/api/pools');
    
    expect(res.statusCode).toEqual(200);
    expect(res.body).toHaveProperty('pools');
    expect(Array.isArray(res.body.pools)).toBeTruthy();
  });
  
  test('Should get a specific pool', async () => {
    const res = await request(app)
      .get(`/api/pools/${testPoolId}`);
    
    expect(res.statusCode).toEqual(200);
    expect(res.body).toHaveProperty('pool');
    expect(res.body.pool._id).toEqual(testPoolId);
  });
});

describe('Price Oracle API', () => {
  test('Should get price for a token', async () => {
    const res = await request(app)
      .get('/api/price-oracle/price/solana');
    
    expect(res.statusCode).toEqual(200);
    expect(res.body).toHaveProperty('data');
    expect(res.body.data).toHaveProperty('price');
  });
});

// More test suites can be added for other APIs