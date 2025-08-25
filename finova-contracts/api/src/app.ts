/**
 * Express application setup.
 */

import express, { Application, Request, Response } from 'express';
import cors from 'cors';
import helmet from 'helmet';
import { userRouter } from './routes/user.routes'; // We will create this next

const app: Application = express();

// --- Middleware ---
// Enable Cross-Origin Resource Sharing
app.use(cors());
// Secure the app by setting various HTTP headers
app.use(helmet());
// Parse incoming JSON requests
app.use(express.json());
// Parse URL-encoded bodies
app.use(express.urlencoded({ extended: true }));


// --- Health Check Route ---
app.get('/', (req: Request, res: Response) => {
    res.status(200).json({
        status: 'ok',
        message: 'Finova API is running',
        timestamp: new Date().toISOString(),
    });
});

// --- API Routes ---
// Mount the user router
app.use('/api/v1/users', userRouter);
// Other routers will be mounted here
// app.use('/api/v1/mining', miningRouter);
// app.use('/api/v1/nfts', nftRouter);


// --- Error Handling Middleware ---
// (To be implemented later)
// app.use(errorHandler);


export default app;
