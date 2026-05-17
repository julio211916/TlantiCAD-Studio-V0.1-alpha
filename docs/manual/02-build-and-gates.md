# Build And Gates

Primary gate:

```bash
bun run recovery:gate -- --phase=all
```

Required validation:

```bash
bun run --cwd frontend typecheck
bun run --cwd frontend build
bun run --cwd frontend bundle:budget
bun run --cwd frontend dicom-ai:qa -- --strict-command-coverage
cargo metadata --manifest-path Tauri/Cargo.toml --no-deps
cargo check --manifest-path Tauri/Cargo.toml
```

## Gate Ownership

- `source`: active roots, Cargo members, copied artifacts.
- `frontend`: Next/Tailwind workspace scan, offline runtime routes, workload shell.
- `ipc`: command registration and IPC contract.
- `tauri`: app manifest, CSP, loopback dev URL, cargo check.
- `python`: FastAPI/Python sidecar structure and DICOM/AI deps.
- `assets`: heavy asset scan isolated from fast diagnostics.
- `runtime`: preloader, workspace routing, DICOM clinical lazy route and trame-slicer sidecar contract.
