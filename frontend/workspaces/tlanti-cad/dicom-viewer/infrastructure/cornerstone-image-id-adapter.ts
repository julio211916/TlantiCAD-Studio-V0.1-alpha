/**
 * Image ID factory for Cornerstone3D. Keeps direct knowledge of
 * `@cornerstonejs/dicom-image-loader.wadouri.fileManager` inside this
 * adapter so the rest of the codebase never imports Cornerstone directly
 * (respects the hexagonal boundary).
 */

import { wadouri } from '@cornerstonejs/dicom-image-loader';
import type { ImageIdFactoryPort } from '../application/ports';

export function createCornerstoneImageIdFactory(): ImageIdFactoryPort {
    return {
        fromFile(file) {
            // Cornerstone's fileManager.add returns a wadouri:<blob:...> id.
            return wadouri.fileManager.add(file);
        },
        fromPath(absolutePath) {
            // For Tauri we prefix file:// then let dicom-image-loader fetch it.
            // Works because the Tauri runtime exposes local files via fetch proxy.
            const cleaned = absolutePath.replace(/\\/g, '/');
            const url = cleaned.startsWith('file://') ? cleaned : `file://${cleaned}`;
            return `wadouri:${url}`;
        },
    };
}
