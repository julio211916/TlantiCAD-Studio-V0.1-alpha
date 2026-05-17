export interface BrowserCapabilityState {
  runtime: 'desktop' | 'browser-fallback';
  filesystem: boolean;
  gpu: boolean;
  dicom: boolean;
  nativeWindow: boolean;
  export: boolean;
}

function hasTauriRuntime() {
  if (typeof window === 'undefined') {
    return false;
  }

  return '__TAURI_INTERNALS__' in window || '__TAURI__' in window;
}

function hasWebGl() {
  if (typeof document === 'undefined') {
    return false;
  }

  try {
    const canvas = document.createElement('canvas');
    return Boolean(canvas.getContext('webgl') || canvas.getContext('experimental-webgl'));
  } catch {
    return false;
  }
}

export function getBrowserCapabilityState(): BrowserCapabilityState {
  const desktop = hasTauriRuntime();
  const gpu = hasWebGl();

  return {
    runtime: desktop ? 'desktop' : 'browser-fallback',
    filesystem: desktop,
    gpu,
    dicom: true,
    nativeWindow: desktop,
    export: true,
  };
}
