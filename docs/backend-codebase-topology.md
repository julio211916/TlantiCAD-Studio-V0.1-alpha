# TlantiCAD Backend Codebase Topology

Fecha: 2026-04-30

Este documento fija la frontera actual del backend Tauri/Rust. No es una idea futura: refleja que compila hoy y separa el codigo vivo de las carpetas que quedaron de otros linajes.

## Verdad activa

- App de escritorio: `Tauri/Cargo.toml`.
- Workspace Rust activo: `Tauri/src/Cargo.toml`.
- DB destino: `Tauri/src/tlanticad-workspace`, publicado como package `tlanticad-db`.
- Compute destino: `Tauri/src/tlanticad-compute`.
- IPC vivo: `Tauri/src/lib.rs` registra los comandos que realmente llegan al frontend.

## Capas actuales

### Desktop adapter

`Tauri/src/lib.rs` y los modulos `Tauri/src/*.rs` son adaptadores Tauri. Pueden mapear DTOs, validar requests, resolver paths de app y llamar crates de dominio. No deben convertirse en repositorios paralelos ni routers de algoritmos.

### DB canonica

`tlanticad-db` debe absorber por slices:

- `case_repository.rs` -> `repository/cases.rs`
- `case_storage.rs` -> `repository/assets.rs`
- `clinical_jobs.rs` -> `repository/clinical_jobs.rs`
- `mesh_vault.rs` -> `repository/mesh_vault.rs`
- `cad_parameters_store.rs` -> `repository/parameters.rs`

Mientras esa migracion ocurre, los modulos Tauri existentes siguen siendo el runtime vivo. La ruta unica de DB objetivo es `storage_layout::database_path()` (`TlantiCADData/database/tlanticad.sqlite`).

### Compute canonico

`tlanticad-compute` es el unico dueno de:

- descubrimiento de backends (`ComputeRouter::available_backend_pool`)
- ranking por operacion (`BenchProfile`)
- decision por op (`ComputeRouter::picked_kind_for`)
- fallback a CPU

`Tauri/src/cad_compute_router.rs` queda como adaptador IPC/estado/persistencia del perfil. No debe reconstruir listas CPU/GPU por su cuenta.

### Runtime Python

La regla objetivo es una sola ruta:

- `python_runtime.rs` resuelve configuracion de Python.
- `sidecar-manager` supervisa procesos largos cuando se migre al build activo.
- `dicom_jobs.rs` orquesta jobs y artefactos, no debe sumar nuevos spawners ad-hoc.

## Carpetas en cuarentena

No importarlas en codigo nuevo hasta migrarlas o eliminarlas:

- `Tauri/src/database`
- `Tauri/src/db`
- `Tauri/src/dental-database`
- `Tauri/src/cad-db`
- `Tauri/src/commands`
- `Tauri/src/dental-commands`
- `Tauri/src/python-bridge`
- `Tauri/src/sidecar-manager`
- `Tauri/src/cad-core`
- `Tauri/src/dental-core`
- `Tauri/src/patients`
- `Tauri/src/clinical`

Motivo: no estan en el manifiesto activo del app ni en el workspace activo, o contienen schemas/comandos/modelos que compiten con rutas vivas. Conectarlas tal cual duplicaria pacientes, casos, migraciones e IPC.

## Bug corregido

`Tauri/sql/001_core_schema.sql` solo permite `cases.status` en `draft`, `in-progress`, `ready`, `archived`. `case_repository` insertaba `new`; ahora crea casos como `draft`.

## Comando de inspeccion

El runtime expone `inspect_backend_topology` para que el frontend, tests o QA puedan ver la topologia actual sin leer el arbol a mano. Ese comando devuelve:

- manifests activos
- crate canonico DB
- crate canonico compute
- modulos runtime activos
- carpetas en cuarentena
- reglas de arquitectura
