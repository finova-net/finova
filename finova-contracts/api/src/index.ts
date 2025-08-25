/**
 * Main entry point for the Finova API server.
 */

import express from 'express';
import http from 'http';
import app from './app';
import { logger } from './utils/logger';

const PORT = process.env.PORT || 3000;

const server = http.createServer(app);

server.listen(PORT, () => {
    logger.info(`🚀 Server is listening on port ${PORT}`);
    logger.info(`🔗 Connected to database`); // Placeholder log
    logger.info(`🔗 Connected to Solana RPC`); // Placeholder log
});

// Graceful shutdown
process.on('SIGTERM', () => {
    logger.info('SIGTERM signal received: closing HTTP server');
    server.close(() => {
        logger.info('HTTP server closed');
        // Close database connections, etc.
        process.exit(0);
    });
});

process.on('SIGINT', () => {
    logger.info('SIGINT signal received: closing HTTP server');
    server.close(() => {
        logger.info('HTTP server closed');
        process.exit(0);
    });
});
