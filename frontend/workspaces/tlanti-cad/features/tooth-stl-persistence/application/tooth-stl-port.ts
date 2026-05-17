import type { ToothMeshBuffer, ToothStlEntry } from '../domain/tooth-stl';

export interface ToothStlWriteInput {
    caseFolderPath: string;
    toothFdi: number;
    buffer?: ToothMeshBuffer;
    /** Used when no buffer supplied — sizes the placeholder cube in mm. */
    placeholderSizeMm?: number;
    /**
     * V217 / V222 — file prefix selector. Default 'tooth' writes
     * tooth-{fdi}.stl; 'abutment' (V217) writes abutment-{fdi}.stl;
     * 'screwchannel' (V222) writes screwchannel-{fdi}.stl so the merge bridge
     * can subtract the channel from the union.
     */
    prefix?: 'tooth' | 'abutment' | 'screwchannel';
}

export interface ToothStlWriteOutput {
    outputPath: string;
    triangleCount: number;
    placeholder: boolean;
}

export interface ToothStlPort {
    write(input: ToothStlWriteInput): Promise<ToothStlWriteOutput>;
    list(caseFolderPath: string): Promise<ToothStlEntry[]>;
    clear(caseFolderPath: string): Promise<number>;
}
