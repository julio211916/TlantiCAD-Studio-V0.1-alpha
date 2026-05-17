export type { ToothMeshBuffer, ToothStlEntry } from './domain/tooth-stl';
export { validateToothBuffer } from './domain/tooth-stl';

export type {
    ToothStlPort,
    ToothStlWriteInput,
    ToothStlWriteOutput,
} from './application/tooth-stl-port';
export { createBackendToothStlAdapter } from './infrastructure/backend-tooth-stl-adapter';
