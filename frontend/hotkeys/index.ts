export type { HotkeyContext, HotkeyDefinition } from './domain/hotkey';
export { chordFromEvent, normalizeChord } from './domain/hotkey';
export { hotkeyRegistry } from './application/hotkey-registry';
export { HotkeyHelpOverlay } from './ui/HotkeyHelpOverlay';
export { useHotkey } from './ui/useHotkey';
