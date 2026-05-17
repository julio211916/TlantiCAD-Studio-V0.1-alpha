import { z } from 'zod';

import type { TlantiDbState } from '@/stores/tlantidb-case-store';

const performanceProfileSchema = z.object({
  mode: z.string().optional(),
  source: z.string().optional(),
  renderQuality: z.string().optional(),
  dicomCacheMb: z.number().optional(),
  enableGpuInference: z.boolean().optional(),
  enable3dViewer: z.boolean().optional(),
  enableCbctViewer: z.boolean().optional(),
  maxConcurrentImports: z.number().optional(),
  thumbnailQuality: z.string().optional(),
  overallScore: z.number().optional(),
  machineSignature: z.string().nullable().optional(),
  lastAutoAppliedAt: z.string().nullable().optional(),
}).passthrough();

const preferencesSchema = z.object({
  timeZone: z.string().optional(),
  numberingSystem: z.enum(['FDI', 'UNIVERSAL']).optional(),
  assetProfile: z.enum(['clinical', 'lab', 'demo']).optional(),
  operatorAlias: z.string().optional(),
  navigationSensitivity: z.object({
    zoom: z.number().optional(),
    pan: z.number().optional(),
    rotation: z.number().optional(),
  }).passthrough().optional(),
  performanceProfile: performanceProfileSchema.optional(),
}).passthrough();

const caseSchema = z.object({
  id: z.string(),
  name: z.string().optional(),
  caseNumber: z.string().optional(),
  status: z.string().optional(),
  assets: z.array(z.record(z.unknown())).optional(),
  toothMap: z.record(z.record(z.unknown())).optional(),
  pipeline: z.record(z.unknown()).optional(),
}).passthrough();

const stateSchema = z.object({
  activeCaseId: z.string().optional(),
  cases: z.array(caseSchema).optional(),
  preferences: preferencesSchema.optional(),
}).passthrough();

export function parsePersistedTlantiDbState(value: unknown): Partial<TlantiDbState> | null {
  const parsed = stateSchema.safeParse(value);
  return parsed.success ? parsed.data as Partial<TlantiDbState> : null;
}
