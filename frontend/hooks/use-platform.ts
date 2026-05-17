import { useEffect, useState } from "react"

export type Platform = "windows" | "macos" | "linux" | "ios" | "android" | "unknown"

export interface PlatformInfo {
  platform: Platform
  isWindows: boolean
  isMacOS: boolean
  isLinux: boolean
  isDesktop: boolean
  isMobile: boolean
  isLoading: boolean
}

let cachedPlatform: Platform | null = null

function detectPlatformFromNavigator(): Platform {
  if (typeof navigator === "undefined") return "unknown"

  const userAgent = navigator.userAgent.toLowerCase()
  const platform = navigator.platform?.toLowerCase() ?? ""

  if (/iphone|ipad|ipod/.test(userAgent)) return "ios"
  if (/android/.test(userAgent)) return "android"
  if (platform.includes("win") || userAgent.includes("windows")) return "windows"
  if (platform.includes("mac") || userAgent.includes("macintosh")) return "macos"
  if (platform.includes("linux") || userAgent.includes("linux")) return "linux"

  return "unknown"
}

function normalizeTauriPlatform(value: string): Platform {
  if (value === "windows" || value === "macos" || value === "linux" || value === "ios" || value === "android") {
    return value
  }
  return "unknown"
}

export function getPlatformSync(): Platform {
  if (cachedPlatform) return cachedPlatform
  return detectPlatformFromNavigator()
}

export function usePlatform(): PlatformInfo {
  const initialPlatform = getPlatformSync()
  const [platform, setPlatform] = useState<Platform>(initialPlatform)
  const [isLoading, setIsLoading] = useState(cachedPlatform === null)

  useEffect(() => {
    if (cachedPlatform) {
      setPlatform(cachedPlatform)
      setIsLoading(false)
      return
    }

    let mounted = true

    async function detect() {
      try {
        const os = await import("@tauri-apps/plugin-os")
        const detected = normalizeTauriPlatform(await os.type())
        cachedPlatform = detected
        if (mounted) setPlatform(detected)
      } catch {
        cachedPlatform = detectPlatformFromNavigator()
        if (mounted) setPlatform(cachedPlatform)
      } finally {
        if (mounted) setIsLoading(false)
      }
    }

    void detect()

    return () => {
      mounted = false
    }
  }, [])

  return {
    platform,
    isWindows: platform === "windows",
    isMacOS: platform === "macos",
    isLinux: platform === "linux",
    isDesktop: platform === "windows" || platform === "macos" || platform === "linux",
    isMobile: platform === "ios" || platform === "android",
    isLoading,
  }
}

export function isMacOS(): boolean {
  return getPlatformSync() === "macos"
}

export function isWindows(): boolean {
  return getPlatformSync() === "windows"
}

export function isLinux(): boolean {
  return getPlatformSync() === "linux"
}

export function useIsFullscreen(): boolean {
  const [isFullscreen, setIsFullscreen] = useState(false)

  useEffect(() => {
    let mounted = true
    let unlisten: (() => void) | undefined

    async function checkFullscreen() {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window")
        const fullscreen = await getCurrentWindow().isFullscreen()
        if (mounted) setIsFullscreen(fullscreen)
      } catch {
        if (mounted) setIsFullscreen(document.fullscreenElement !== null)
      }
    }

    function handleBrowserFullscreen() {
      setIsFullscreen(document.fullscreenElement !== null)
    }

    window.addEventListener("resize", checkFullscreen)
    document.addEventListener("fullscreenchange", handleBrowserFullscreen)
    void checkFullscreen()

    async function setupTauriListener() {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window")
        const off = await getCurrentWindow().onResized(() => {
          void checkFullscreen()
        })
        if (mounted) {
          unlisten = off
        } else {
          off()
        }
      } catch {
        // Browser/dev fallback only.
      }
    }

    void setupTauriListener()

    return () => {
      mounted = false
      window.removeEventListener("resize", checkFullscreen)
      document.removeEventListener("fullscreenchange", handleBrowserFullscreen)
      unlisten?.()
    }
  }, [])

  return isFullscreen
}

