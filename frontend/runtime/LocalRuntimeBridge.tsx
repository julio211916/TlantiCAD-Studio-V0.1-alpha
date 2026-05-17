'use client';

import { useEffect, useMemo, useState } from 'react';

import { BACKEND_ORIGIN } from '@/lib/backend-config';
import { ipc } from '@/lib/ipc';

export interface RuntimeInfo {
  appName: string;
  company: string;
  version: string;
  buildUid: string;
  profile: string;
}

export interface LocalBackendEndpoint {
  baseUrl: string;
  healthPath: string;
  websocketBaseUrl: string;
  offlineOnly: boolean;
}

interface LocalBackendHealth {
  status: string;
  offlineOnly?: boolean;
  dicom?: unknown;
  queue?: unknown;
  database?: unknown;
}

interface BridgeState {
  tauri: 'desktop' | 'browser' | 'error';
  fastapi: 'checking' | 'ready' | 'offline';
  runtimeInfo?: RuntimeInfo;
  backendEndpoint: string;
  error?: string;
}

const HEALTH_TIMEOUT_MS = 1800;

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && ('__TAURI_INTERNALS__' in window || '__TAURI__' in window);
}

async function fetchJsonWithTimeout<T>(url: string, timeoutMs: number): Promise<T> {
  const controller = new AbortController();
  const timeoutId = window.setTimeout(() => controller.abort(), timeoutMs);
  try {
    const response = await fetch(url, {
      method: 'GET',
      cache: 'no-store',
      signal: controller.signal,
    });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
    return await response.json() as T;
  } finally {
    window.clearTimeout(timeoutId);
  }
}

function resolveBrowserBackendEndpoint(): string {
  return process.env.NEXT_PUBLIC_TLANTI_FASTAPI_URL ?? BACKEND_ORIGIN;
}

export function LocalRuntimeBridge() {
  const initialEndpoint = useMemo(resolveBrowserBackendEndpoint, []);
  const [state, setState] = useState<BridgeState>({
    tauri: 'browser',
    fastapi: 'checking',
    backendEndpoint: initialEndpoint,
  });

  useEffect(() => {
    let cancelled = false;

    async function runHealthcheck() {
      const tauri = isTauriRuntime();
      let endpoint = initialEndpoint;
      let runtimeInfo: RuntimeInfo | undefined;

      try {
        if (tauri) {
          const [nextEndpoint, nextRuntimeInfo] = await Promise.all([
            ipc<void, LocalBackendEndpoint>('local_backend_endpoint'),
            ipc<void, RuntimeInfo>('get_runtime_info'),
          ]);
          endpoint = nextEndpoint.baseUrl;
          runtimeInfo = nextRuntimeInfo;
        }

        const health = await fetchJsonWithTimeout<LocalBackendHealth>(
          `${endpoint.replace(/\/$/, '')}/api/v1/health/local`,
          HEALTH_TIMEOUT_MS,
        );

        if (!cancelled) {
          setState({
            tauri: tauri ? 'desktop' : 'browser',
            fastapi: health.status === 'ready' ? 'ready' : 'offline',
            runtimeInfo,
            backendEndpoint: endpoint,
          });
        }
      } catch (error) {
        if (!cancelled) {
          setState({
            tauri: tauri ? 'desktop' : 'browser',
            fastapi: 'offline',
            runtimeInfo,
            backendEndpoint: endpoint,
            error: error instanceof Error ? error.message : String(error),
          });
        }
      }
    }

    void runHealthcheck();
    return () => {
      cancelled = true;
    };
  }, [initialEndpoint]);

  return (
    <aside className="tlanti-runtime-bridge" aria-label="Runtime local">
      <span data-state={state.tauri}>{state.tauri === 'desktop' ? 'Tauri' : 'Next'}</span>
      <span data-state={state.fastapi}>{state.fastapi === 'ready' ? 'FastAPI' : 'FastAPI off'}</span>
      <code>{state.runtimeInfo?.profile ?? state.backendEndpoint.replace('http://', '')}</code>
      {state.error && <span className="tlanti-runtime-bridge__error">{state.error}</span>}
    </aside>
  );
}
