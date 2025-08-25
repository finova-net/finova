/**
 * Router for user-related API endpoints.
 */

import express from 'express';
import { getUserProfile, updateUserProfile } from '../controllers/user.controller';

const router = express.Router();

// Define routes
router.route('/:id').get(getUserProfile);
router.route('/:id').put(updateUserProfile);

export { router as userRouter };
