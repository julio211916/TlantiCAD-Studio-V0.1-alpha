# TlantiCAD 100 Sprint Roadmap

## S001-S010 Foundation

Tailwind/workspaces, shell visible, preloader, aliases, routing, hydration, runtime bridge, browser smoke, docs base, gate UI.

## S011-S020 TlantiDB DentalDB

Workload wizard, case browser module open, required assets, readiness bar, case status, patient/lab panels, search, snapshots, XML/reveal/export, tests.

## S021-S030 IPC/Tauri Contract

Parse `generate_handler`, sync contracts, remove direct invokes, map missing commands, split capabilities, CSP loopback allowlist, command coverage, no remote refs, cargo checks, docs.

## S031-S040 Python Sidecar

Unify port/health, supervisor lifecycle, logs/PID, health alias, model manifest, ONNX blocked/ready states, DICOM fixture, smoke API, package notes, gate.

## S041-S050 DICOM Engine

Path-based import, manifest, metadata, thumbnails, preview tiles, VolView handle wiring, VTK real volume, cleanup/dispose, DICOM smoke, no full-buffer gate.

## S051-S060 MeshVault And Assets

Asset handles, STL/OBJ/PLY path import, library indexing, virtualized Mesh Library, LOD, culling, instancing, ref-count dispose, large asset benchmark, IO budget.

## S061-S070 CAD Modules

CAD shell registry, crown, margin, insertion axis, bridge connector, abutment, articulator, guide, splint, model creator, module smoke per case.

## S071-S080 Clinical Workflows

Import -> Clean -> Segment -> Design -> Validate -> Export, DICOM -> segmentation -> mesh, implant planning, surgical guide, splints, orthodontics, smile/ortho, audit trail, case graph.

## S081-S090 Performance Hardening

Bundle budget, startup budget, lazy routes, React selectors, virtualization, workers, Rust jobs, Python session cache, GPU memory audit, benchmark dashboard.

## S091-S100 Clinical Release

Offline network-block gate, PHI/DICOM de-id, model hashes/licensing, packaged sidecar, macOS dylibs, Tauri dev/build smoke, recovery docs, manual, release checklist, final acceptance.
