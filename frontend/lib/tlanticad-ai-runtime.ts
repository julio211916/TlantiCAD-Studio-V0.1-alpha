export type TlantiAiLanguageCode = 'en' | 'es' | 'fr' | 'it' | 'pt' | 'ru' | 'de' | 'ja' | 'ko' | 'zh';

export type TlantiAiEngine =
  | 'qwen'
  | 'qwen_custom_voice'
  | 'luxtts'
  | 'chatterbox'
  | 'chatterbox_turbo'
  | 'tada'
  | 'kokoro';

export interface TlantiAiHealth {
  status: string;
  modelLoaded: boolean;
  modelDownloaded?: boolean;
  modelSize?: string;
  gpuAvailable: boolean;
  gpuType?: string;
  backendType?: string;
  backendVariant?: string;
}

export interface TlantiAiProfile {
  id: string;
  name: string;
  description?: string;
  language: string;
  voiceType: 'cloned' | 'preset' | 'designed';
  presetEngine?: string;
  presetVoiceId?: string;
  defaultEngine?: string;
  generationCount: number;
  sampleCount: number;
}

export interface TlantiAiGeneration {
  id: string;
  profileId: string;
  text: string;
  language: string;
  audioPath?: string;
  duration?: number;
  status: 'loading_model' | 'generating' | 'completed' | 'failed';
  error?: string;
  engine?: string;
  modelSize?: string;
  createdAt: string;
}

export interface TlantiAiModelStatus {
  modelName: string;
  displayName: string;
  downloaded: boolean;
  downloading: boolean;
  loaded: boolean;
  sizeMb?: number;
}

export interface TlantiAiTranscription {
  text: string;
  duration: number;
}

const DEFAULT_TLANTICAD_AI_URL = 'http://127.0.0.1:17493';
const TLANTICAD_AI_URL_STORAGE_KEY = 'tlanticad.ai.baseUrl';
const LEGACY_VOICEBOX_URL_STORAGE_KEY = 'tlanticad.voicebox.baseUrl';

function toCamelProfile(raw: any): TlantiAiProfile {
  return {
    id: raw.id,
    name: raw.name,
    description: raw.description,
    language: raw.language,
    voiceType: raw.voice_type,
    presetEngine: raw.preset_engine,
    presetVoiceId: raw.preset_voice_id,
    defaultEngine: raw.default_engine,
    generationCount: raw.generation_count ?? 0,
    sampleCount: raw.sample_count ?? 0,
  };
}

function toCamelGeneration(raw: any): TlantiAiGeneration {
  return {
    id: raw.id,
    profileId: raw.profile_id,
    text: raw.text,
    language: raw.language,
    audioPath: raw.audio_path,
    duration: raw.duration,
    status: raw.status,
    error: raw.error,
    engine: raw.engine,
    modelSize: raw.model_size,
    createdAt: raw.created_at,
  };
}

function toCamelHealth(raw: any): TlantiAiHealth {
  return {
    status: raw.status,
    modelLoaded: Boolean(raw.model_loaded),
    modelDownloaded: raw.model_downloaded,
    modelSize: raw.model_size,
    gpuAvailable: Boolean(raw.gpu_available),
    gpuType: raw.gpu_type,
    backendType: raw.backend_type,
    backendVariant: raw.backend_variant,
  };
}

function toCamelModel(raw: any): TlantiAiModelStatus {
  return {
    modelName: raw.model_name,
    displayName: raw.display_name,
    downloaded: Boolean(raw.downloaded),
    downloading: Boolean(raw.downloading),
    loaded: Boolean(raw.loaded),
    sizeMb: raw.size_mb,
  };
}

function formatErrorDetail(detail: unknown, fallback: string): string {
  if (typeof detail === 'string') return detail;
  if (Array.isArray(detail)) {
    return detail.map((item) => item?.msg || item?.message || JSON.stringify(item)).join('; ');
  }
  if (detail && typeof detail === 'object') {
    const message = (detail as Record<string, unknown>).message;
    return typeof message === 'string' ? message : JSON.stringify(detail);
  }
  return fallback;
}

export function getTlantiAiBaseUrl() {
  if (typeof window === 'undefined') {
    return DEFAULT_TLANTICAD_AI_URL;
  }

  return window.localStorage.getItem(TLANTICAD_AI_URL_STORAGE_KEY)
    || window.localStorage.getItem(LEGACY_VOICEBOX_URL_STORAGE_KEY)
    || DEFAULT_TLANTICAD_AI_URL;
}

export function setTlantiAiBaseUrl(value: string) {
  if (typeof window === 'undefined') {
    return;
  }

  window.localStorage.setItem(TLANTICAD_AI_URL_STORAGE_KEY, value.replace(/\/$/, ''));
}

async function requestJson<T>(endpoint: string, options?: RequestInit): Promise<T> {
  const response = await fetch(`${getTlantiAiBaseUrl()}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ detail: response.statusText }));
    throw new Error(formatErrorDetail(error.detail, `TlantiCAD AI HTTP ${response.status}`));
  }

  return response.json();
}

export const TlantiAiRuntime = {
  async health(): Promise<TlantiAiHealth> {
    return toCamelHealth(await requestJson('/health'));
  },

  async listProfiles(): Promise<TlantiAiProfile[]> {
    const profiles = await requestJson<any[]>('/profiles');
    return profiles.map(toCamelProfile);
  },

  async createDentalNarrator(language: TlantiAiLanguageCode = 'es'): Promise<TlantiAiProfile> {
    const profile = await requestJson('/profiles', {
      method: 'POST',
      body: JSON.stringify({
        name: `TlantiCAD Narrador ${language.toUpperCase()}`,
        description: 'Voz preset para instrucciones clínicas, notas CAD y guías de workflow dental.',
        language,
        voice_type: 'preset',
        preset_engine: 'kokoro',
        preset_voice_id: language === 'es' ? 'ef_dora' : 'af_heart',
        default_engine: 'kokoro',
      }),
    });
    return toCamelProfile(profile);
  },

  async listModels(): Promise<TlantiAiModelStatus[]> {
    const result = await requestJson<{ models: any[] }>('/models/status');
    return result.models.map(toCamelModel);
  },

  async generateSpeech(input: {
    profileId: string;
    text: string;
    language?: TlantiAiLanguageCode;
    engine?: TlantiAiEngine;
    seed?: number;
    instruct?: string;
  }): Promise<TlantiAiGeneration> {
    const generation = await requestJson('/generate', {
      method: 'POST',
      body: JSON.stringify({
        profile_id: input.profileId,
        text: input.text,
        language: input.language ?? 'es',
        engine: input.engine ?? 'kokoro',
        seed: input.seed,
        instruct: input.instruct,
        normalize: true,
        max_chunk_chars: 1200,
        crossfade_ms: 80,
      }),
    });
    return toCamelGeneration(generation);
  },

  async getGeneration(generationId: string): Promise<TlantiAiGeneration> {
    return toCamelGeneration(await requestJson(`/history/${generationId}`));
  },

  getAudioUrl(generationId: string) {
    return `${getTlantiAiBaseUrl()}/audio/${generationId}`;
  },

  async transcribeAudio(file: File, language: TlantiAiLanguageCode = 'es'): Promise<TlantiAiTranscription> {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('language', language);
    formData.append('model', 'base');

    const response = await fetch(`${getTlantiAiBaseUrl()}/transcribe`, {
      method: 'POST',
      body: formData,
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ detail: response.statusText }));
      throw new Error(formatErrorDetail(error.detail, `TlantiCAD AI HTTP ${response.status}`));
    }

    return response.json();
  },
};
