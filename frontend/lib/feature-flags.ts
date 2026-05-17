export function isDentistSotaBetaEnabled(): boolean {
  const env = (import.meta as unknown as { env?: Record<string, string | boolean | undefined> }).env
  const envValue = env?.VITE_TLANTICAD_DENTIST_SOTA_BETA
  if (envValue === '1' || envValue === 'true') return true
  if (envValue === '0' || envValue === 'false') return false

  if (typeof window !== 'undefined') {
    const stored = window.localStorage.getItem('tlanticad.features.dentistSotaBeta')
    if (stored === '1' || stored === 'true') return true
    if (stored === '0' || stored === 'false') return false
  }

  return env?.DEV === true || env?.DEV === 'true'
}
