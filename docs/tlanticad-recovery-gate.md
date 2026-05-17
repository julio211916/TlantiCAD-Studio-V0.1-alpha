# TlantiCAD Recovery Gate

## Contract

Run:

```bash
bun run recovery:gate -- --phase=all
```

Phases:

- `source`: active roots, Rust workspace members and risky source artifacts.
- `frontend`: Next/TS entrypoints, Tailwind workspace scanning, workload-first shell wiring and remote route policy.
- `ipc`: active invokes through wrapper, DICOM handle commands and Tauri command registration.
- `tauri`: active Tauri app metadata, CSP and loopback dev URL.
- `python`: local sidecar structure and DICOM/AI dependency declarations.
- `assets`: STL/OBJ/PLY/library scan with isolated build IO budget.
- `runtime`: desktop runtime manifest, preloader context, case-browser module launch, VolView DICOM lazy routing and gate command exposure.

## Failure Policy

The active `frontend` + `Tauri` contract fails on missing roots, broken Cargo metadata, direct active invoke bypasses, null CSP, non-loopback active routes, missing workload routing, missing DICOM handle commands or workspace roots that are not scanned by Tailwind.

The gate is now strict for clinical runtime code: source copy artifacts are not allowed under active Rust roots, and frontend runtime roots must be offline-only. Documentation comments, tests, generated Next output and generated Emscripten glue are excluded from remote-route detection so diagnostics stay focused on executable risk.

TlantiDB, preloader context and TlantiCAD now share one clinical route contract: workload presets resolve to canonical module IDs, DICOM opens the VolView dental workspace lazily, and CAD modules stay on the mesh/CAD host. DICOM and mesh workflows must move handles, paths, manifests and job IDs across IPC; React must not move full clinical file buffers.

The manual under `docs/manual/` is part of the contract. A build that removes the offline clinical contract, routing map, build/gate guide or asset-handle guide is treated as an architecture regression.

## Maintainability Impact

The gate becomes the executable architecture checklist. New modules must declare roots, commands, workload targets, handle contracts and runtime expectations instead of depending on tribal knowledge. This keeps TlantiDB, preloader, TlantiCAD, VolView, Tauri and Python aligned as the workstation grows.

## Build IO Impact

Heavy libraries are scanned only in `assets`. Source, frontend and IPC phases stay fast enough for frequent diagnosis, while Tailwind scans the active workspace roots so UI classes are generated without broad vendor traversal.

## Diagnostic Latency Impact

The gate prints phase timing, which makes slow checks visible. Cargo metadata is kept in `tauri`; full `cargo check` remains an explicit validation step because it is slower and depends on native dependencies. DICOM/Mesh/Python checks should prefer manifest and health probes first, then run heavier asset scans only when the `assets` or `all` phase is requested.

## Current Validation Snapshot

- `bun run tlanticad:typecheck`: passes.
- `bun run tlanticad:build`: passes.
- `cargo metadata --manifest-path Tauri/src/Cargo.toml --no-deps`: passes.
- `cargo check --manifest-path Tauri/src/Cargo.toml`: passes without Rust warnings.
- `cargo build --manifest-path Tauri/src/Cargo.toml`: passes without Rust warnings.
- `bun run recovery:gate -- --phase=all`: passes with `failures=0 warnings=0`.
- `curl -I http://127.0.0.1:1420/`: returns `HTTP/1.1 200 OK` after Next compiles the real workspace route.
