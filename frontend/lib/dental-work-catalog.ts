/**
 * Full exocad-style "Defining jobs for individual teeth" catalog.
 *
 * Reference: exocad docs → Defining jobs for individual teeth (2010-2025).
 * This is a UI-facing catalog — it does NOT replace `dental-workflow.ts`,
 * which keeps the legacy restoration union. Each entry here carries a
 * `legacyType` mapping so the existing store shape keeps working while
 * the modal offers the full exocad vocabulary.
 */

import type { DentalMaterialType, DentalRestorationType } from './dental-workflow';

export type DentalWorkCategory =
    | 'crowns-copings'
    | 'pontics-mockup'
    | 'inlays-onlays-veneers'
    | 'digital-copy-milling'
    | 'removables-appliances'
    | 'bars'
    | 'residual-dentition';

export interface DentalWorkType {
    id: string;
    label: string;
    description: string;
    category: DentalWorkCategory;
    color: string;
    /** Mapping into the legacy DentalRestorationType so the store stays backwards-compatible. */
    legacyType: DentalRestorationType;
    /** Baseline lab work time in minutes — used as the default for the tooth-work dialog and emitted in the interop XML. */
    defaultMinutes: number;
}

export const DENTAL_WORK_CATEGORIES: Array<{ id: DentalWorkCategory; label: string }> = [
    { id: 'crowns-copings', label: 'Crowns and copings' },
    { id: 'pontics-mockup', label: 'Pontics and Mockup' },
    { id: 'inlays-onlays-veneers', label: 'Inlays, onlays and veneers' },
    { id: 'digital-copy-milling', label: 'Digital copy milling' },
    { id: 'removables-appliances', label: 'Removables and appliances' },
    { id: 'bars', label: 'Bars' },
    { id: 'residual-dentition', label: 'Residual dentition' },
];

export const DENTAL_WORK_TYPES: DentalWorkType[] = [
    // Crowns and copings
    {
        id: 'anatomic-crown',
        label: 'Anatomic crown',
        description: 'Full contour crown with anatomic shape.',
        category: 'crowns-copings',
        color: '#7F4BD8',
        legacyType: 'anatomic-crown',
        defaultMinutes: 35,
    },
    {
        id: 'coping',
        label: 'Coping',
        description: 'Anatomic coping derived from the full contour using cutback.',
        category: 'crowns-copings',
        color: '#2BB6A4',
        legacyType: 'anatomic-coping',
        defaultMinutes: 20,
    },
    {
        id: 'pressed-crown',
        label: 'Pressed crown',
        description: 'Two-part restoration: framework + chewing surface to overpress.',
        category: 'crowns-copings',
        color: '#7AAE4F',
        legacyType: 'anatomic-crown',
        defaultMinutes: 45,
    },
    {
        id: 'eggshell-crown',
        label: 'Eggshell crown (Provisional)',
        description: 'Provisional eggshell designed preoperatively over non-prepped teeth.',
        category: 'crowns-copings',
        color: '#A37BD1',
        legacyType: 'anatomic-crown',
        defaultMinutes: 15,
    },
    {
        id: 'overlay',
        label: 'Overlay',
        description: 'Three-quarter / onlay crown for concave preparation margins.',
        category: 'crowns-copings',
        color: '#9EA1A7',
        legacyType: 'inlay-onlay',
        defaultMinutes: 30,
    },
    {
        id: 'offset-coping',
        label: 'Offset coping',
        description: 'Simple coping with fixed thickness over the preparation.',
        category: 'crowns-copings',
        color: '#34A853',
        legacyType: 'anatomic-coping',
        defaultMinutes: 20,
    },
    // Pontics and mockup
    {
        id: 'anatomic-pontic',
        label: 'Anatomic pontic',
        description: 'Full contour pontic; connected to adjacent restorations of same material.',
        category: 'pontics-mockup',
        color: '#A03B4A',
        legacyType: 'pontic',
        defaultMinutes: 40,
    },
    {
        id: 'reduced-pontic',
        label: 'Reduced pontic',
        description: 'Pontic derived from the anatomic shape using cutback.',
        category: 'pontics-mockup',
        color: '#D65365',
        legacyType: 'pontic',
        defaultMinutes: 30,
    },
    {
        id: 'pressed-pontic',
        label: 'Pressed pontic',
        description: 'Two-piece pontic: framework + over-press.',
        category: 'pontics-mockup',
        color: '#6CAEC0',
        legacyType: 'pontic',
        defaultMinutes: 50,
    },
    {
        id: 'eggshell-pontic',
        label: 'Eggshell pontic (Provisional)',
        description: 'Provisional full-contour pontic over non-prepped teeth.',
        category: 'pontics-mockup',
        color: '#D0507A',
        legacyType: 'pontic',
        defaultMinutes: 20,
    },
    {
        id: 'mockup',
        label: 'Mockup',
        description: 'Full-contour or veneer snap-on reconstruction (try-in).',
        category: 'pontics-mockup',
        color: '#C85C63',
        legacyType: 'veneer',
        defaultMinutes: 25,
    },
    // Inlays, onlays, veneers
    {
        id: 'inlay',
        label: 'Inlay',
        description: 'Single-cusp inlay with full anatomy.',
        category: 'inlays-onlays-veneers',
        color: '#1E7F5F',
        legacyType: 'inlay-onlay',
        defaultMinutes: 25,
    },
    {
        id: 'inlay-onlay',
        label: 'Inlay / Onlay',
        description: 'Inlay or onlay with full anatomy (multi-cusp).',
        category: 'inlays-onlays-veneers',
        color: '#1E7F5F',
        legacyType: 'inlay-onlay',
        defaultMinutes: 25,
    },
    {
        id: 'offset-inlay',
        label: 'Offset inlay',
        description: 'Framework for an inlay with fixed thickness.',
        category: 'inlays-onlays-veneers',
        color: '#206AC9',
        legacyType: 'inlay-onlay',
        defaultMinutes: 25,
    },
    {
        id: 'veneer',
        label: 'Veneer',
        description: 'Full-anatomic veneer over a prepped tooth.',
        category: 'inlays-onlays-veneers',
        color: '#3CA8D4',
        legacyType: 'veneer',
        defaultMinutes: 30,
    },
    // Digital copy milling — see Pre-op vs Waxup (https://wiki.exocad.com/...)
    {
        id: 'waxup',
        label: 'Waxup',
        description: '1:1 copy milling of a wax modellation (no model teeth used).',
        category: 'digital-copy-milling',
        color: '#5B9F5B',
        legacyType: 'anatomic-crown',
        defaultMinutes: 30,
    },
    {
        id: 'anatomic-waxup',
        label: 'Anatomic waxup',
        description: 'Full-contour crown copied from anatomic wax model over a prep.',
        category: 'digital-copy-milling',
        color: '#6FBF6F',
        legacyType: 'anatomic-crown',
        defaultMinutes: 40,
    },
    {
        id: 'reduced-waxup',
        label: 'Reduced waxup',
        description: 'Framework derived from anatomic wax with cutback.',
        category: 'digital-copy-milling',
        color: '#A1554E',
        legacyType: 'anatomic-coping',
        defaultMinutes: 35,
    },
    {
        id: 'pontic-waxup',
        label: 'Pontic waxup',
        description: 'Full-contour pontic over a missing tooth from wax.',
        category: 'digital-copy-milling',
        color: '#B53E49',
        legacyType: 'pontic',
        defaultMinutes: 45,
    },
    // Removables and appliances
    {
        id: 'full-denture',
        label: 'Full denture',
        description: 'Full edentulous denture using standard tooth libraries.',
        category: 'removables-appliances',
        color: '#5F6F74',
        legacyType: 'anatomic-crown',
        defaultMinutes: 180,
    },
    {
        id: 'partial-denture',
        label: 'Partial denture',
        description: 'Partial denture framework case.',
        category: 'removables-appliances',
        color: '#8B9093',
        legacyType: 'anatomic-coping',
        defaultMinutes: 120,
    },
    {
        id: 'bite-splint',
        label: 'Bite splint',
        description: 'Bite splint — custom night-guard, sports-guard, occlusal correction.',
        category: 'removables-appliances',
        color: '#4A6574',
        legacyType: 'anatomic-crown',
        defaultMinutes: 60,
    },
    {
        id: 'primary-telescopic-crown',
        label: 'Primary telescopic crown',
        description: 'Primary part for a telescopic removable structure.',
        category: 'removables-appliances',
        color: '#BC6974',
        legacyType: 'anatomic-coping',
        defaultMinutes: 60,
    },
    {
        id: 'secondary-telescopic-crown',
        label: 'Secondary telescopic crown',
        description: 'Secondary part for a telescopic removable structure.',
        category: 'removables-appliances',
        color: '#A0757B',
        legacyType: 'anatomic-coping',
        defaultMinutes: 60,
    },
    {
        id: 'attachment',
        label: 'Attachment',
        description: 'Extra-coronal attachment.',
        category: 'removables-appliances',
        color: '#253441',
        legacyType: 'anatomic-coping',
        defaultMinutes: 25,
    },
    // Bars
    {
        id: 'bar-pillar',
        label: 'Bar pillar',
        description: 'Portion of a bar that connects to the implant.',
        category: 'bars',
        color: '#B04A48',
        legacyType: 'implant-restoration',
        defaultMinutes: 50,
    },
    {
        id: 'bar-segment',
        label: 'Bar segment',
        description: 'Connection between bar pillars.',
        category: 'bars',
        color: '#6D47B5',
        legacyType: 'implant-restoration',
        defaultMinutes: 45,
    },
    {
        id: 'offset-substructure',
        label: 'Offset substructure',
        description: 'Framework for a substructure with a fixed thickness.',
        category: 'bars',
        color: '#808284',
        legacyType: 'anatomic-coping',
        defaultMinutes: 55,
    },
    // Residual dentition
    {
        id: 'antagonist',
        label: 'Antagonist',
        description: 'Define in the opposing jaw to use an antagonist scan.',
        category: 'residual-dentition',
        color: '#D65A34',
        legacyType: 'antagonist',
        defaultMinutes: 0,
    },
    {
        id: 'adjacent-tooth',
        label: 'Adjacent tooth',
        description: 'Healthy tooth to be scanned but not restored.',
        category: 'residual-dentition',
        color: '#E2924B',
        legacyType: 'adjacent-tooth',
        defaultMinutes: 0,
    },
    {
        id: 'omit-in-bridge',
        label: 'Omit in bridge',
        description: 'Missing tooth, not to be restored — enables non-adjacent connectors.',
        category: 'residual-dentition',
        color: '#D7373F',
        legacyType: 'omit-in-bridge',
        defaultMinutes: 0,
    },
];

export function resolveWorkType(id: string): DentalWorkType | null {
    return DENTAL_WORK_TYPES.find((w) => w.id === id) ?? null;
}

export function workTypesByCategory(category: DentalWorkCategory): DentalWorkType[] {
    return DENTAL_WORK_TYPES.filter((w) => w.category === category);
}

// ─── Material catalog ────────────────────────────────────────────────

export interface DentalMaterialEntry {
    id: DentalMaterialType | 'zirconia-translucent' | 'titanium-laser' | 'np-metal-laser';
    label: string;
    color: string;
    previewUrl?: string;
    /** Does the material appear for this production method drop-down? */
    productionTags: Array<'3-axis-milling' | '5-axis-laser-3dprint' | 'outsourced'>;
}

export const DENTAL_MATERIALS: DentalMaterialEntry[] = [
    { id: 'zirconia', label: 'Zirconia', color: '#E3E5E7', previewUrl: '/images/materials/zi.png', productionTags: ['3-axis-milling', '5-axis-laser-3dprint'] },
    { id: 'zirconia-translucent', label: 'Zirconia Translucent', color: '#F0F1F3', productionTags: ['5-axis-laser-3dprint'] },
    { id: 'zirconia-multilayer', label: 'Zirconia Multilayer', color: '#D9DFE6', productionTags: ['5-axis-laser-3dprint'] },
    { id: 'pmma', label: 'Acrylic / PMMA', color: '#E8D9B3', previewUrl: '/images/materials/pmma.png', productionTags: ['3-axis-milling', '5-axis-laser-3dprint'] },
    { id: 'composite', label: 'Composite', color: '#CDB89D', previewUrl: '/images/materials/comp.png', productionTags: ['5-axis-laser-3dprint'] },
    { id: 'np-metal', label: 'NP Metal', color: '#9FA2A4', previewUrl: '/images/materials/np.png', productionTags: ['3-axis-milling', '5-axis-laser-3dprint'] },
    { id: 'np-metal-laser', label: 'NP Metal (Laser)', color: '#8A8D8F', previewUrl: '/images/materials/np_l.png', productionTags: ['5-axis-laser-3dprint'] },
    { id: 'titanium', label: 'Titanium', color: '#8F9193', previewUrl: '/images/materials/ti.png', productionTags: ['5-axis-laser-3dprint'] },
    { id: 'titanium-laser', label: 'Titanium (Laser)', color: '#707274', previewUrl: '/images/materials/ti.png', productionTags: ['5-axis-laser-3dprint'] },
    { id: '3d-print', label: '3D Print', color: '#C1A36E', productionTags: ['5-axis-laser-3dprint'] },
    { id: 'wax', label: 'Wax', color: '#5EAE95', previewUrl: '/images/materials/wax.png', productionTags: ['3-axis-milling', '5-axis-laser-3dprint'] },
    { id: 'lithium-disilicate', label: 'Lithium Disilicate', color: '#7A6BB5', productionTags: ['5-axis-laser-3dprint'] },
];

// ─── Implant options ─────────────────────────────────────────────────

export type ImplantOption =
    | 'none'
    | 'custom-abutment'
    | 'screw-retained'
    | 'post-and-core'
    | 'on-stock-abutment'
    | 'on-substructure-scan';

export interface ImplantOptionEntry {
    id: ImplantOption;
    label: string;
    description: string;
    color: string;
}

export const IMPLANT_OPTIONS: ImplantOptionEntry[] = [
    { id: 'none', label: 'No implant', description: 'Conventional restoration over prepared tooth.', color: '#475569' },
    { id: 'custom-abutment', label: 'Custom Abutment', description: 'Design a custom abutment in addition to the restoration.', color: '#A93226' },
    { id: 'screw-retained', label: 'Screw-retained', description: 'Screw-mounted restoration, no abutment.', color: '#1F9253' },
    { id: 'post-and-core', label: 'Post and core', description: 'Post-and-core + final restoration in a single workflow.', color: '#374151' },
    { id: 'on-stock-abutment', label: 'On stock abutment', description: 'Restoration designed over a stock abutment library.', color: '#E67E22' },
    { id: 'on-substructure-scan', label: 'On substructure scan', description: 'Restoration designed over a previously-designed substructure (e.g. bar).', color: '#8E44AD' },
];

export const MATERIAL_SHADES: string[] = ['Default', 'A', 'B', 'C', 'D', '—'];

export const PRODUCTION_METHODS = [
    { id: '3-axis-milling', label: '3/4-Axis milling' },
    { id: '5-axis-laser-3dprint', label: '5-Axis / Laser / 3D Print' },
    { id: 'outsourced', label: 'Outsourced / Milling center' },
];
