# Workspace Routing

The canonical route shape is:

```text
/?workspace=<tlantidb|tlanticad>&caseId=<case-id>&module=<module-id>
```

## Canonical Module IDs

`cad`, `dicom`, `model-creator`, `partials`, `implant`, `guide`, `splint`, `ceph`, `fab`, `aligners`, `orthocad`.

## Workload Mapping

- `crown-bridge` -> `cad`
- `implant-planning` -> `implant`
- `dicom-cbct` and `dicom-viewer` -> `dicom`
- `smile-design` -> `orthocad`
- `splint-guide` -> `splint`
- `orthodontics` -> `aligners`

All module launches must pass through this normalizer before entering `TlantiCAD`.
