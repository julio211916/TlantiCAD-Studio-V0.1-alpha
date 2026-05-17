import { z } from 'zod';

export const toolkitPayloadSchema = z.object({
  payload: z.string().min(1, 'Add a payload to process.'),
  passphrase: z.string().min(8, 'Use at least 8 characters for the passphrase.'),
});

export type ToolkitPayloadForm = z.infer<typeof toolkitPayloadSchema>;