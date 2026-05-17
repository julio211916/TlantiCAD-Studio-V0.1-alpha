# Asset And DICOM Handles

Production clinical IO uses handles.

```ts
type ClinicalAssetHandle =
  | { kind: "mesh"; meshKey: string; storagePath: string }
  | { kind: "volume"; volumeKey: string; manifestPath: string }
  | { kind: "labelmap"; labelmapKey: string; manifestPath: string }
  | { kind: "library-asset"; assetId: string; storagePath: string };
```

## Rules

- React receives metadata, progress and preview handles.
- Tauri resolves local paths and job handles.
- Python reads DICOM from disk and writes manifests/artifacts.
- Rust owns mesh streaming, repair, boolean, offset, export and cacheable compute.
