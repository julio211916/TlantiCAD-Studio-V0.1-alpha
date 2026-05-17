import type { TlantiCaseAsset } from '@/stores/tlantidb-case-store';

export type ClinicalImportFilter = { name: string; extensions: string[] };

export const DEFAULT_CLINICAL_IMPORT_ACCEPT =
    '.stl,.obj,.ply,.glb,.gltf,.dcm,.dicom,.ima,.png,.jpg,.jpeg,.webp,.pdf,.txt,.md,.xml';

/**
 * Resolve the native file dialog filters for a clinical import based on the
 * asset roles the caller is requesting. Filters narrow the picker to the
 * extensions that make sense clinically for that next-step role.
 */
export function resolveClinicalImportFilters(
    roles?: TlantiCaseAsset['role'][] | null,
): ClinicalImportFilter[] {
    const roleSet = new Set(roles ?? []);
    const extensionSet = new Set<string>();

    if (roleSet.size === 0) {
        [
            'stl', 'obj', 'ply', 'glb', 'gltf',
            'dcm', 'dicom', 'ima',
            'png', 'jpg', 'jpeg', 'webp',
            'pdf', 'txt', 'md', 'xml',
        ].forEach((extension) => extensionSet.add(extension));
    }

    if (
        roleSet.has('lab-prescription') ||
        roleSet.has('clinical-note') ||
        roleSet.has('consent-document')
    ) {
        ['pdf', 'jpg', 'jpeg', 'png'].forEach((extension) => extensionSet.add(extension));
    }

    if (
        roleSet.has('prep-scan') ||
        roleSet.has('dicom-study') ||
        roleSet.has('restoration-model') ||
        roleSet.has('gingiva-scan')
    ) {
        ['stl', 'obj', 'dcm', 'dicom', 'ima'].forEach((extension) => extensionSet.add(extension));
    }

    if (roleSet.has('antagonist-scan') || roleSet.has('bite-registration')) {
        ['stl', 'obj'].forEach((extension) => extensionSet.add(extension));
    }

    if (
        roleSet.has('shade-reference') ||
        roleSet.has('smile-photo') ||
        roleSet.has('clinical-photo') ||
        roleSet.has('pre-op-photo')
    ) {
        ['jpg', 'jpeg', 'png'].forEach((extension) => extensionSet.add(extension));
    }

    if (roleSet.has('manufacturing-report')) {
        ['pdf', 'xml', 'txt'].forEach((extension) => extensionSet.add(extension));
    }

    const extensions = Array.from(extensionSet);
    return [
        {
            name: roles?.length ? 'Clinical assets for next step' : 'Clinical assets',
            extensions,
        },
    ];
}

/**
 * Resolve the HTML `<input accept="...">` string for the same role set.
 * Used by the web fallback file input when Tauri dialog is unavailable.
 */
export function resolveClinicalImportAccept(
    roles?: TlantiCaseAsset['role'][] | null,
): string {
    return resolveClinicalImportFilters(roles)
        .flatMap((filter) => filter.extensions)
        .map((extension) => `.${extension}`)
        .join(',');
}
