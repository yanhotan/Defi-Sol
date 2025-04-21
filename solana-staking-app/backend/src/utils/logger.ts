// src/utils/logger.ts
// Logger utility
// - Configure logging for the application
// - Provide methods for logging errors, warnings, and info

import winston from 'winston';

// Configure the logger
const logger = winston.createLogger({
  level: 'info',
  format: winston.format.combine(
    winston.format.timestamp(),
    winston.format.printf(({ timestamp, level, message }) => {
      return `${timestamp} [${level.toUpperCase()}]: ${message}`;
    })
  ),
  transports: [
    new winston.transports.Console(),
    new winston.transports.File({ filename: 'logs/error.log', level: 'error' }),
    new winston.transports.File({ filename: 'logs/combined.log' }),
  ],
});

// Export logging methods
export const logInfo = (message: string) => logger.info(message);
export const logError = (message: string) => logger.error(message);
export const logWarning = (message: string) => logger.warn(message);

export default logger;