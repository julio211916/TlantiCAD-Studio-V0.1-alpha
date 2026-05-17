/**
 * ZIP archive adapter. Uses `jszip` (already installed) to expand a user-provided
 * .zip into an array of File objects in memory, preserving filenames.
 *
 * Only files that look remotely like DICOM are kept; the parser decides
 * conclusively downstream. This keeps memory bounded for large archives that
 * mix reports/images/dicom.
 */

import JSZip from 'jszip';
import type { ZipArchivePort } from '../application/ports';

const KEEP_EXTENSION_RE = /\.(dcm|dicom|ima|zip)$/i;
const DICOM_MAGIC = new Uint8Array([0x44, 0x49, 0x43, 0x4d]); // "DICM"

function hasDicomMagic(bytes: Uint8Array): boolean {
    if (bytes.byteLength < 132) return false;
    // DICOM files have a 128-byte preamble followed by "DICM".
    for (let i = 0; i < DICOM_MAGIC.length; i += 1) {
        if (bytes[128 + i] !== DICOM_MAGIC[i]) return false;
    }
    return true;
}

export function createJszipAdapter(): ZipArchivePort {
    return {
        async expand(archive) {
            const zip = await JSZip.loadAsync(archive);
            const files: File[] = [];

            const entries = Object.values(zip.files).filter((entry) => !entry.dir);

            for (const entry of entries) {
                const name = entry.name.split('/').pop() ?? entry.name;
                if (name.startsWith('.')) continue; // skip hidden (e.g. __MACOSX)
                const ext = KEEP_EXTENSION_RE.test(name);
                const buffer = await entry.async('uint8array');

                // Recurse one level into nested ZIPs (common for Slicer exports).
                if (/\.zip$/i.test(name)) {
                    try {
                        const innerZip = new File([buffer], name, { type: 'application/zip' });
                        const nested = await (createJszipAdapter()).expand(innerZip);
                        files.push(...nested);
                        continue;
                    } catch {
                        // Inner zip corrupt — skip
                        continue;
                    }
                }

                // Accept if extension matches or if DICOM magic bytes present
                if (ext || hasDicomMagic(buffer)) {
                    files.push(new File([buffer], name));
                }
            }

            return files;
        },
    };
}
