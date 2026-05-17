export type {
    CommandAction,
    CommandKind,
    CommandMatch,
} from './domain/command-action';
export { fuzzyScore, rankActions } from './domain/command-action';

export { commandRegistry } from './application/command-registry';

export { CommandPalette } from './ui/CommandPalette';
export { useCommandPalette } from './ui/useCommandPalette';
export type { UseCommandPaletteResult } from './ui/useCommandPalette';
