# `Tauri/src` — mapa activo del backend Rust

Esta carpeta mezcla dos realidades:

1. `Tauri/src/Cargo.toml` es el workspace Rust activo de librerias `tlanticad-*`.
2. `Tauri/src/*.rs` contiene el adaptador Tauri vivo registrado por `Tauri/Cargo.toml`.

Las carpetas laterales que no estan en `Tauri/Cargo.toml` ni en `Tauri/src/Cargo.toml`
quedan en cuarentena: no se importan en codigo nuevo hasta migrarlas por slices al crate
canonico correcto.

El mapa operativo completo vive en `docs/backend-codebase-topology.md` y tambien se
expone por IPC con `inspect_backend_topology`.

## Estado 2026-04-30

- App activa: `Tauri/Cargo.toml`.
- Workspace activo: `Tauri/src/Cargo.toml`.
- DB destino: `Tauri/src/tlanticad-workspace` (`package = "tlanticad-db"`).
- Compute destino: `Tauri/src/tlanticad-compute`.
- IPC vivo: comandos registrados en `Tauri/src/lib.rs`.
- Cuarentena: `database`, `db`, `dental-database`, `cad-db`, `commands`,
  `dental-commands`, `python-bridge`, `sidecar-manager`, `cad-core`, `dental-core`,
  `patients`, `clinical`.

## Migracion pendiente

## Estructura objetivo (post-MP-014)

```
crates/
├── tlanticad-core/                  # Tipos base, IDs, Result<T>, units
├── tlanticad-geometry/              # B-rep, PCA, KD-tree, BVH
├── tlanticad-mesh/                  # Mesh ops, region grow, drape
├── tlanticad-csg/                   # Manifold + csgrs + OCCT (consolidar 3 fuentes)
├── tlanticad-formats/               # STL/OBJ/PLY/glTF/DICOM
├── tlanticad-crown/                 # Crown engine
├── tlanticad-abutment/              # Abutment edit + production
├── tlanticad-bridge/                # Connectors + framework
├── tlanticad-implant/               # Manager + fixation guide
├── tlanticad-articulator/           # Bonwill + jaw motion + planes
├── tlanticad-endo/                  # Chamber + canal PCA
├── tlanticad-freeform/              # Brush engine + paint-pull + specialty
├── tlanticad-dicom/                 # DICOM parser + segmentation
├── tlanticad-ai-runtime/            # Model registry + ONNX
├── tlanticad-compute/               # ComputeBackend + CPU/GPU/NPU router
├── tlanticad-project/               # Case repo + watcher + parameters
├── tlanticad-rendering/             # R3F bridges
├── tlanticad-tauri-bindings/        # Macros para Tauri commands
├── tlanticad-wasm/                  # wasm-bindgen wrapper
└── tlanticad-occt/                  # OpenCascade (feature flag)
```

## Reglas

- Cada crate público con `#[deny(missing_docs)]` en su `lib.rs`.
- Cada feature CAD se publica en dos capas: `tlanticad_<feature>` (Rust) + `tlanticad_<feature>::wasm`.
- Ningún algoritmo de >50 LOC vive en `apps/desktop/src/commands/*.rs`.
- Toda op pesada pasa por `tlanticad-compute::ComputeRouter`.
- Ninguna carpeta en cuarentena se conecta al build activo sin eliminar schemas, modelos o
  comandos duplicados.
