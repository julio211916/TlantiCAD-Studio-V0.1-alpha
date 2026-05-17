import type { ArticulatorPreset, ArticulatorVendorEntry } from '../domain/articulator-vendor';

export interface ArticulatorLibraryPort {
    list(): Promise<{ vendors: ArticulatorVendorEntry[]; backend: 'filesystem' | 'mock' }>;
    getPreset(vendorId: string): Promise<ArticulatorPreset>;
}
