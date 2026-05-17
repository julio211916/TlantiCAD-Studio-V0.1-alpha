import { z } from 'zod';

export const dicomExecutionBriefSchema = z.object({
  owner: z.string().min(2, 'Define a responsible owner for Sprint 01 / Phase 01.'),
  validationMode: z.enum(['clinical', 'dataset', 'ui', 'automation']),
  evidenceGoal: z.string().min(8, 'Describe the evidence needed to close this phase.'),
});

export type DicomExecutionBriefForm = z.infer<typeof dicomExecutionBriefSchema>;