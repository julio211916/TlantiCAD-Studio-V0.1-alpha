/**
 * Per-tooth STL persistence — domain (V202).
 *
 * The merge step's CSG bridge requires real STLs at
 * `{caseFolderPath}/tooth-{fdi}.stl` to produce a real boolean union. Each
 * wizard step writes its current tooth-level mesh by calling the port; this
 * module owns the value types.
 */

export interface ToothMeshBuffer {
    /** Triangulated positions, row-major xyz (length % 3 === 0). */
    positions: number[];
    /** Triangle indices (length % 3 === 0). */
    indices: number[];
}

export interface ToothStlEntry {
    fdi: number;
    path: string;
    sizeBytes: number;
}

/**
 * Validates a tooth mesh buffer before sending to the backend.
 * Returns null on success, error message on failure.
 */
export function validateToothBuffer(buf: ToothMeshBuffer): string | null {
    if (buf.positions.length === 0) return 'positions buffer is empty';
    if (buf.positions.length % 3 !== 0) return 'positions length must be a multiple of 3';
    if (buf.indices.length === 0) return 'indices buffer is empty';
    if (buf.indices.length % 3 !== 0) return 'indices length must be a multiple of 3';
    const maxIdx = Math.max(...buf.indices);
    const vertexCount = buf.positions.length / 3;
    if (maxIdx >= vertexCount) {
        return `indices reference vertex ${maxIdx} but only ${vertexCount} vertices supplied`;
    }
    return null;
}
