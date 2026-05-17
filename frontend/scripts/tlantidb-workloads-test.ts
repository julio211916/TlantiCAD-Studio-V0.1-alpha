import assert from 'node:assert/strict';

import { createDefaultCase, type TlantiCaseAsset } from '../lib/tlantidb-case-store';
import {
  applyWorkloadToCase,
  getMissingRequiredAssets,
  getNextWorkloadAction,
  inferWorkloadFromCase,
  TLANTIDB_WORKLOAD_PRESETS,
} from '../lib/tlantidb-workloads';

function asset(role: TlantiCaseAsset['role']): TlantiCaseAsset {
  return {
    id: crypto.randomUUID(),
    category: role === 'dicom-study' ? 'dicom' : role.includes('photo') || role === 'shade-reference' ? 'image' : role.includes('scan') ? 'model' : 'document',
    name: `${role}.fixture`,
    role,
    tags: [role],
    relativePath: `fixtures/${role}.fixture`,
    importedAt: new Date().toISOString(),
  };
}

assert.equal(TLANTIDB_WORKLOAD_PRESETS.length, 7, 'all planned TlantiDB workload presets are registered');

for (const preset of TLANTIDB_WORKLOAD_PRESETS) {
  assert.ok(preset.moduleTarget, `${preset.id} has a module target`);
  assert.ok(preset.requiredAssetRoles.length > 0, `${preset.id} has required assets`);
  assert.ok(preset.defaultToothPatch.restorationType, `${preset.id} has tooth defaults`);
}

const legacyCrown = createDefaultCase({
  toothMap: {
    'tooth-11': {
      selected: true,
      antagonist: false,
      condition: 'anatomic-coping',
      restorationType: 'anatomic-coping',
    },
  },
});
assert.equal(inferWorkloadFromCase(legacyCrown).id, 'crown-bridge', 'legacy restorative case infers crown workflow');
assert.deepEqual(getMissingRequiredAssets(legacyCrown), ['lab-prescription', 'prep-scan']);
assert.equal(getNextWorkloadAction(legacyCrown).blocked, true);

const implant = createDefaultCase({
  toothMap: {
    'tooth-13': {
      selected: true,
      antagonist: false,
      condition: 'implant-restoration',
      restorationType: 'implant-restoration',
      implantMode: 'custom-abutment',
    },
  },
  assets: [asset('dicom-study')],
});
assert.equal(inferWorkloadFromCase(implant).id, 'implant-planning', 'DICOM plus implant tooth infers implant planning');
assert.deepEqual(getMissingRequiredAssets({ ...implant, ...applyWorkloadToCase(implant, 'implant-planning') }), ['lab-prescription']);

const readyCrown = {
  ...legacyCrown,
  ...applyWorkloadToCase(legacyCrown, 'crown-bridge'),
  assets: [asset('lab-prescription'), asset('prep-scan')],
};
assert.deepEqual(getMissingRequiredAssets(readyCrown), []);
assert.equal(getNextWorkloadAction(readyCrown).blocked, false);

console.log('TlantiDB workload presets, inference, required assets, and next actions passed.');
