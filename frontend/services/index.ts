/**
 * Services Index - CADHY
 *
 * Re-exports all services for convenient imports.
 */

// AI chat service
export * from "./ai-service"
// Typed IPC layer (re-exported from lib/ipc for service-side consumers)
export * from "../lib/ipc"
// CAD operations (OpenCASCADE)
export * from "./cad-operations-init"
export * from "./cad-service"
// Chat persistence
export * from "./chat-persistence"
// Hotkeys
export * from "./default-hotkeys"
// Export utilities
export * from "./export-service"
export * from "./hotkey-registry"
// Hydraulic analysis
export * from "./hydraulics-service"
// Performance and optimization
export * from "./instancing-manager"
export * from "./lod-manager"
export * from "./material-pool"
// Measurement and tools
export * from "./measurement-tools"
// Project management
export * from "./project-service"
export * from "./section-tool"
export * from "./snap-manager"
// System and Tauri utilities
export * from "./tauri-service"
// Textures
export * from "./texture-cache"
export * from "./texture-service"
// Thumbnail generation
export * from "./thumbnail-service"
// Viewport
export * from "./viewport-registry"
