export type {
    MaterialLibraryEntry,
    MaterialLibrarySource,
    MaterialLibraryState,
} from './domain/material-library';
export { defaultMaterialLibraryState, findLibrary } from './domain/material-library';

export type { MaterialLibraryPort } from './application/material-library-port';
export { createBackendMaterialLibraryAdapter } from './infrastructure/backend-material-library-adapter';

export { MaterialLibraryDialog } from './ui/MaterialLibraryDialog';
export type { MaterialLibraryDialogProps } from './ui/MaterialLibraryDialog';
