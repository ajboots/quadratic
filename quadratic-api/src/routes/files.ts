import express from 'express';
import { validateAccessToken } from '../middleware/auth';
import { Request as JWTRequest } from 'express-jwt';
import { z } from 'zod';
import rateLimit from 'express-rate-limit';
import { PrismaClient } from '@prisma/client';
import { get_user } from '../helpers/get_user';
import { get_file } from '../helpers/get_file';

const files_router = express.Router();
const prisma = new PrismaClient();

const ai_rate_limiter = rateLimit({
  windowMs: Number(process.env.RATE_LIMIT_AI_WINDOW_MS) || 3 * 60 * 60 * 1000, // 3 hours
  max: Number(process.env.RATE_LIMIT_AI_REQUESTS_MAX) || 25, // Limit number of requests per windowMs
  standardHeaders: true, // Return rate limit info in the `RateLimit-*` headers
  legacyHeaders: false, // Disable the `X-RateLimit-*` headers
  keyGenerator: (request: JWTRequest, response) => {
    return request.auth?.sub || 'anonymous';
  },
});

const FilesBackupRequestBody = z.object({
  uuid: z.string(),
  fileContents: z.any(),
});

// type FilesBackupRequestBodyType = z.infer<typeof FilesBackupRequestBody>;

files_router.post('/backup', validateAccessToken, ai_rate_limiter, async (request: JWTRequest, response) => {
  const r_json = FilesBackupRequestBody.parse(request.body);

  const user = await get_user(request);
  const file = await get_file(user, r_json.uuid);

  const file_contents = JSON.parse(r_json.fileContents);

  if (file) {
    await prisma.qFile.update({
      where: {
        id: file.id,
      },
      data: {
        contents: file_contents,
        updated_date: new Date(),
        times_updated: {
          increment: 1,
        },
      },
    });
  } else {
    await prisma.qFile.create({
      data: {
        name: r_json.uuid, // TODO: Get File Name from Contents
        version: undefined, // TODO: Get Version from Contents
        contents: file_contents,
        qUserId: user.id,
        uuid: r_json.uuid,
        created_date: new Date(),
      },
    });
  }

  response.status(200);
});

export default files_router;