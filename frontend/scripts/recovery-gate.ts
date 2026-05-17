import { spawnSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, extname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

type Phase = 'source' | 'frontend' | 'ipc' | 'tauri' | 'python' | 'assets' | 'runtime';
type Severity = 'pass' | 'warn' | 'fail';

interface Finding {
  severity: Severity;
  phase: Phase;
  message: string;
  detail?: string;
}

const SCRIPT_DIR = dirname(fileURLToPath(import.meta.url));
const FRONTEND_ROOT = resolve(SCRIPT_DIR, '..');
const REPO_ROOT = resolve(FRONTEND_ROOT, '..');
const PHASES: Phase[] = ['source', 'frontend', 'ipc', 'tauri', 'python', 'assets', 'runtime'];
const TEXT_EXTENSIONS = new Set(['.ts', '.tsx', '.js', '.jsx', '.rs', '.toml', '.json', '.py', '.md', '.vue']);
const HEAVY_SKIP_DIRS = new Set(['.git', 'node_modules', 'target', 'dist', 'out', 'build', '.next', '.venv', 'venv', '__pycache__', 'thirdparty']);

function rel(path: string): string {
  return relative(REPO_ROOT, path) || '.';
}

function finding(severity: Severity, phase: Phase, message: string, detail?: string): Finding {
  return { severity, phase, message, detail };
}

function readText(path: string): string {
  return readFileSync(path, 'utf8');
}

function stripComments(text: string): string {
  return text
    .replace(/\/\*[\s\S]*?\*\//g, '')
    .replace(/(^|[^:])\/\/.*$/gm, '$1');
}

function runCommand(command: string, args: string[], cwd = REPO_ROOT) {
  const result = spawnSync(command, args, { cwd, encoding: 'utf8', timeout: 120_000 });
  return {
    ok: result.status === 0,
    stdout: result.stdout ?? '',
    stderr: result.stderr ?? '',
  };
}

function walkFiles(root: string, options: { maxFiles?: number; includeHeavy?: boolean } = {}): string[] {
  const maxFiles = options.maxFiles ?? 20_000;
  const files: string[] = [];

  function visit(path: string) {
    if (files.length >= maxFiles || !existsSync(path)) return;
    const stats = statSync(path);
    if (stats.isDirectory()) {
      const name = path.split(/[\\/]/).at(-1) ?? '';
      if (!options.includeHeavy && HEAVY_SKIP_DIRS.has(name)) return;
      for (const child of readdirSync(path)) visit(join(path, child));
      return;
    }
    if (stats.isFile()) files.push(path);
  }

  visit(root);
  return files;
}

function readCssArtifacts(): string {
  const cssRoots = [
    resolve(REPO_ROOT, 'frontend/.next/static/css'),
    resolve(REPO_ROOT, 'frontend/out/_next/static/css'),
  ];
  const files = cssRoots.flatMap((root) => (
    existsSync(root)
      ? walkFiles(root, { maxFiles: 200, includeHeavy: true }).filter((file) => extname(file) === '.css')
      : []
  ));
  return files.map((file) => readText(file)).join('\n');
}

function parseArgs(): Phase[] {
  const rawPhase = process.argv.find((arg) => arg.startsWith('--phase='))?.split('=')[1] ?? 'all';
  if (rawPhase === 'all') return PHASES;
  const requested = rawPhase.split(',').map((item) => item.trim()).filter(Boolean);
  const invalid = requested.filter((item) => !PHASES.includes(item as Phase));
  if (invalid.length > 0) {
    throw new Error(`Invalid recovery phase: ${invalid.join(', ')}. Valid phases: ${PHASES.join(', ')}, all`);
  }
  return requested as Phase[];
}

function checkRequiredPaths(phase: Phase, paths: string[]): Finding[] {
  return paths.map((path) => {
    const absolute = resolve(REPO_ROOT, path);
    return existsSync(absolute)
      ? finding('pass', phase, `Required path present: ${path}`)
      : finding('fail', phase, `Required path missing: ${path}`);
  });
}

function parseCargoMemberPaths(cargoTomlPath: string): string[] {
  const text = readText(cargoTomlPath);
  const membersMatch = text.match(/members\s*=\s*\[([\s\S]*?)\]/m);
  if (!membersMatch) return [];
  return [...membersMatch[1].matchAll(/"([^"]+)"/g)].map((match) => match[1]);
}

function parseTauriCommands(rustPath: string): string[] {
  if (!existsSync(rustPath)) return [];
  const text = readText(rustPath);
  const match = text.match(/generate_handler!\s*\[([\s\S]*?)\]/m);
  if (!match) return [];
  return [...match[1].matchAll(/\b([a-zA-Z_][a-zA-Z0-9_]*)\b/g)]
    .map((item) => item[1])
    .filter((name) => !['tauri', 'generate_handler'].includes(name));
}

function findRemoteRoutes(paths: string[], includeHeavy = false): string[] {
  const hits: string[] = [];
  const ignoredPathPattern = /(^|\/)(__tests__|tests?|out|\.next|emscripten-build)(\/|$)|\.(test|spec|d)\.[cm]?[tj]sx?$/i;
  const allowedHosts = new Set(['127.0.0.1', 'localhost', '[::1]', 'www.w3.org']);
  for (const root of paths) {
    const absolute = resolve(REPO_ROOT, root);
    for (const file of walkFiles(absolute, { maxFiles: 12_000, includeHeavy })) {
      const repoPath = rel(file);
      if (ignoredPathPattern.test(repoPath)) continue;
      if (!TEXT_EXTENSIONS.has(extname(file))) continue;
      const text = stripComments(readText(file));
      const urls = text.matchAll(/https?:\/\/([^'"`\s/)]+)/gi);
      for (const match of urls) {
        const host = match[1]?.toLowerCase().replace(/:\d+$/, '');
        if (host && !allowedHosts.has(host)) {
          hits.push(repoPath);
          break;
        }
      }
      if (hits.length > 80) return hits;
    }
  }
  return hits;
}

function sourcePhase(): Finding[] {
  const findings = [
    ...checkRequiredPaths('source', [
      'frontend/package.json',
      'frontend/App.tsx',
      'frontend/core',
      'frontend/io',
      'frontend/composables',
      'frontend/workspaces/tlanti-cad/features/dicom-viewer',
      'frontend/workspaces/tlanti-cad/features/ai-runtime',
      'frontend/workspaces/tlanti-cad',
      'frontend/workspaces/tlanti-db',
      'Tauri/src/Cargo.toml',
      'Tauri/Cargo.toml',
    'Tauri/backend/app',
    'Tauri/backend/modules',
    'docs/manual/00-product-contract.md',
    'docs/manual/01-offline-clinical-contract.md',
    'docs/manual/02-build-and-gates.md',
    'docs/manual/03-workspace-routing.md',
    'docs/manual/04-asset-and-dicom-handles.md',
    'docs/sprints/000-roadmap-index.md',
  ]),
  ];

  const cargoToml = resolve(REPO_ROOT, 'Tauri/src/Cargo.toml');
  if (existsSync(cargoToml)) {
    for (const member of parseCargoMemberPaths(cargoToml)) {
      const memberCargo = resolve(REPO_ROOT, 'Tauri/src', member, 'Cargo.toml');
      findings.push(
        existsSync(memberCargo)
          ? finding('pass', 'source', `Cargo member resolved: ${member}`)
          : finding('fail', 'source', `Cargo member has no Cargo.toml: ${member}`),
      );
    }
  }

  const copyArtifacts = [
    ...walkFiles(resolve(REPO_ROOT, 'Tauri/src'), { maxFiles: 8000 }),
    ...walkFiles(resolve(REPO_ROOT, 'frontend'), { maxFiles: 20_000 }),
  ]
    .filter((file) => /\s[0-9]+\.(rs|toml)$|(\.bak|\.orig)$/i.test(file))
    .filter((file) => !/(^|\/)(out|\.next|node_modules)(\/|$)/.test(rel(file)))
    .concat(
      walkFiles(resolve(REPO_ROOT, 'frontend'), { maxFiles: 20_000 })
        .filter((file) => /(^|\/)Copia de /i.test(rel(file)))
        .filter((file) => !/(^|\/)(out|\.next|node_modules)(\/|$)/.test(rel(file))),
    )
    .map(rel);
  if (copyArtifacts.length > 0) {
    findings.push(finding('fail', 'source', 'Source copy artifacts detected in active roots', copyArtifacts.slice(0, 20).join(', ')));
  }

  return findings;
}

function frontendPhase(): Finding[] {
  const findings = checkRequiredPaths('frontend', [
    'frontend/next.config.mjs',
    'frontend/postcss.config.mjs',
    'frontend/tsconfig.json',
    'frontend/app/layout.tsx',
    'frontend/app/page.tsx',
    'frontend/App.tsx',
    'frontend/runtime/LocalRuntimeBridge.tsx',
    'frontend/lib/dicom-jobs.ts',
    'frontend/lib/tlantidb-workloads.ts',
    'frontend/lib/workspace-routing.ts',
    'frontend/lib/workspace-orchestrator.ts',
    'frontend/workspaces/workspace.config.ts',
    'frontend/workspaces/index.ts',
    'frontend/workspaces/TlantiWorkspacePreloader.tsx',
    'frontend/workspaces/tlanti-db/TlantiDbWorkspace.tsx',
    'frontend/workspaces/tlanti-db/TlantiDbWorkloadWizard.tsx',
    'frontend/workspaces/tlanti-cad/TlantiCadWorkspace.tsx',
    'frontend/workspaces/tlanti-cad/features/dicom-viewer/ui/DicomClinicalWorkspace.tsx',
    'frontend/workspaces/tlanti-db/tlantidb-lazy-panels.ts',
    'frontend/core/use-cases/imaging-workflow-use-case.ts',
    'frontend/core/adapters/volview-local-imaging-engine.ts',
    'frontend/core/domain/mesh-engine.ts',
    'frontend/core/adapters/meshlib-vtk-algebra-engine.ts',
    'frontend/core/adapters/local-mesh-uri-resolver.ts',
    'frontend/core/use-cases/mesh-engine-workflow-use-case.ts',
    'Tauri/src/trame_slicer_sidecar.rs',
    'Tauri/backend/python/trame_slicer_sidecar.py',
    'Tauri/backend/python/slicer_automated_dental_runtime.py',
    'Tauri/backend/python/requirements-trame.txt',
    'Tauri/resources/slicer/models/manifest.json',
    'Tauri/resources/slicer/extensions/SlicerAutomatedDentalTools/README.md',
    'Tauri/meshlib',
    'Tauri/meshlib-wasm/src/index.ts',
    'frontend/io/vtk',
  ]);

  const remoteHits = findRemoteRoutes(['frontend/App.tsx', 'frontend/runtime']);
  findings.push(
    remoteHits.length === 0
      ? finding('pass', 'frontend', 'Active frontend app has no non-loopback remote routes')
      : finding('fail', 'frontend', 'Active frontend app contains non-loopback remote routes', remoteHits.join(', ')),
  );

  const existingRemoteHits = findRemoteRoutes(['frontend'], false).slice(0, 35);
  findings.push(
    existingRemoteHits.length === 0
      ? finding('pass', 'frontend', 'Frontend runtime roots are offline-only')
      : finding('fail', 'frontend', 'Frontend runtime roots contain non-loopback remote routes', existingRemoteHits.join(', ')),
  );

  const globalsCss = readText(resolve(REPO_ROOT, 'frontend/styles/globals.css'));
  const packageJson = readText(resolve(REPO_ROOT, 'frontend/package.json'));
  const postcssConfig = readText(resolve(REPO_ROOT, 'frontend/postcss.config.mjs'));

  findings.push(
    packageJson.includes('"@tailwindcss/postcss"') && postcssConfig.includes("'@tailwindcss/postcss'")
      ? finding('pass', 'frontend', 'Tailwind v4 PostCSS plugin is configured')
      : finding('fail', 'frontend', 'Tailwind v4 requires @tailwindcss/postcss in frontend/postcss.config.mjs'),
  );

  for (const sourceRoot of ['../workspaces/**/*', '../ui/**/*', '../core/**/*']) {
    findings.push(
      globalsCss.includes(sourceRoot)
        ? finding('pass', 'frontend', `Tailwind source scans ${sourceRoot}`)
        : finding('fail', 'frontend', `Tailwind source must scan ${sourceRoot}`),
    );
  }

  const baseCss = readText(resolve(REPO_ROOT, 'frontend/styles/base.css'));
  findings.push(
    baseCss.includes('.hidden') && baseCss.includes('display: none !important')
      ? finding('pass', 'frontend', 'Native hidden inputs have CSS failsafe')
      : finding('fail', 'frontend', 'Native hidden inputs need CSS failsafe outside generated Tailwind utilities'),
  );

  const cssArtifacts = readCssArtifacts();
  findings.push(
    cssArtifacts.length > 0
      ? finding('pass', 'frontend', 'Compiled CSS artifacts are available for runtime validation')
      : finding('fail', 'frontend', 'Compiled CSS artifacts missing; run frontend build before clinical gate'),
  );
  if (cssArtifacts.length > 0) {
    findings.push(
      /@source\s|@tailwind\s+utilities/.test(cssArtifacts)
        ? finding('fail', 'frontend', 'Compiled CSS still contains raw Tailwind directives')
        : finding('pass', 'frontend', 'Compiled CSS has no raw Tailwind directives'),
    );
    findings.push(
      /\.size-9\s*\{/.test(cssArtifacts) && /\.hidden\s*\{\s*display:\s*none/.test(cssArtifacts)
        ? finding('pass', 'frontend', 'Compiled CSS includes workspace sizing and hidden utilities')
        : finding('fail', 'frontend', 'Compiled CSS is missing .size-9 or .hidden utilities used by active workspaces'),
    );
  }

  return findings;
}

function ipcPhase(): Finding[] {
  const findings = checkRequiredPaths('ipc', [
    'frontend/lib/ipc/client.ts',
    'frontend/lib/ipc/contracts.ts',
    'frontend/core/adapters/tauri-mesh-vault.ts',
  ]);

  const activeCommands = parseTauriCommands(resolve(REPO_ROOT, 'Tauri/src/lib.rs'));
  for (const command of ['get_runtime_info', 'inspect_backend_topology', 'local_backend_endpoint', 'mesh_vault_import_start', 'mesh_vault_find', 'cad_compute_run_bench', 'dicom_series_import_start', 'dicom_volume_build_start', 'dicom_segmentation_cancel', 'trame_slicer_sidecar_status', 'trame_slicer_sidecar_start', 'trame_slicer_sidecar_stop', 'slicer_runtime_status', 'slicer_runtime_download', 'slicer_models_status', 'slicer_models_download_all', 'slicer_clinical_job_start', 'slicer_clinical_job_status', 'slicer_clinical_job_cancel']) {
    findings.push(
      activeCommands.includes(command)
        ? finding('pass', 'ipc', `Tauri command registered: ${command}`)
        : finding('fail', 'ipc', `Tauri command missing: ${command}`),
    );
  }

  return findings;
}

function tauriPhase(): Finding[] {
  const findings = checkRequiredPaths('tauri', [
    'Tauri/src/Cargo.toml',
    'Tauri/tauri.conf.json',
    'Tauri/src/lib.rs',
    'Tauri/src/main.rs',
  ]);

  const metadata = runCommand('cargo', ['metadata', '--manifest-path', 'Tauri/src/Cargo.toml', '--no-deps']);
  findings.push(
    metadata.ok
      ? finding('pass', 'tauri', 'Rust workspace cargo metadata passes')
      : finding('fail', 'tauri', 'Rust workspace cargo metadata failed', metadata.stderr.trim() || metadata.stdout.trim()),
  );

  const appMetadata = runCommand('cargo', ['metadata', '--manifest-path', 'Tauri/Cargo.toml', '--no-deps']);
  findings.push(
    appMetadata.ok
      ? finding('pass', 'tauri', 'Tauri app cargo metadata passes')
      : finding('fail', 'tauri', 'Tauri app cargo metadata failed', appMetadata.stderr.trim() || appMetadata.stdout.trim()),
  );

  const appCheck = runCommand('cargo', ['check', '--manifest-path', 'Tauri/Cargo.toml']);
  findings.push(
    appCheck.ok
      ? finding('pass', 'tauri', 'Tauri app cargo check passes')
      : finding('fail', 'tauri', 'Tauri app cargo check failed', appCheck.stderr.trim() || appCheck.stdout.trim()),
  );

  const config = readText(resolve(REPO_ROOT, 'Tauri/tauri.conf.json'));
  findings.push(
    /"frontendDist":\s*"\.\.\/frontend\/out"/.test(config)
      ? finding('pass', 'tauri', 'Tauri frontendDist points to original frontend tree')
      : finding('fail', 'tauri', 'Tauri frontendDist must point to ../frontend/out'),
  );
  findings.push(
    /bun run dev/.test(config) && /cd \.\.\/frontend/.test(config)
      ? finding('pass', 'tauri', 'Tauri beforeDevCommand launches original frontend tree')
      : finding('fail', 'tauri', 'Tauri beforeDevCommand must launch ../frontend'),
  );
  findings.push(
    /http:\/\/127\.0\.0\.1:1420/.test(config)
      ? finding('pass', 'tauri', 'Tauri dev URL is loopback-only')
      : finding('fail', 'tauri', 'Tauri dev URL must be 127.0.0.1 loopback'),
  );
  findings.push(
    /"csp":\s*null/.test(config)
      ? finding('fail', 'tauri', 'Tauri CSP cannot be null')
      : finding('pass', 'tauri', 'Tauri CSP is explicit'),
  );

  return findings;
}

function pythonPhase(): Finding[] {
  const findings = checkRequiredPaths('python', [
    'Tauri/backend/app',
    'Tauri/backend/modules',
    'Tauri/backend/python',
    'Tauri/backend/python/requirements.txt',
  ]);

  const pyFiles = walkFiles(resolve(REPO_ROOT, 'Tauri/backend'), { maxFiles: 6000 })
    .filter((file) => extname(file) === '.py');
  findings.push(
    pyFiles.length > 0
      ? finding('pass', 'python', `Python sidecar files discovered: ${pyFiles.length}`)
      : finding('fail', 'python', 'Python sidecar has no .py files'),
  );

  const requirementsPath = resolve(REPO_ROOT, 'Tauri/backend/python/requirements.txt');
  const requirements = existsSync(requirementsPath) ? readText(requirementsPath).toLowerCase() : '';
  for (const dependency of ['fastapi', 'pydicom']) {
    findings.push(
      requirements.includes(dependency)
        ? finding('pass', 'python', `Python dependency declared: ${dependency}`)
        : finding('warn', 'python', `Python dependency not declared in requirements.txt: ${dependency}`),
    );
  }
  findings.push(
    /onnxruntime|torch/.test(requirements)
      ? finding('pass', 'python', 'Local AI runtime dependency declared')
      : finding('warn', 'python', 'No ONNX Runtime or Torch dependency declared; AI healthcheck remains structural'),
  );

  return findings;
}

function assetsPhase(): Finding[] {
  const findings = checkRequiredPaths('assets', ['Tauri/icons', 'Tauri/library']);
  const files = [
    ...walkFiles(resolve(REPO_ROOT, 'Tauri/icons'), { maxFiles: 30_000, includeHeavy: true }),
    ...walkFiles(resolve(REPO_ROOT, 'Tauri/library'), { maxFiles: 30_000, includeHeavy: true }),
  ];
  const counts = new Map<string, number>();
  let bytes = 0;
  for (const file of files) {
    const extension = extname(file).toLowerCase() || '<none>';
    counts.set(extension, (counts.get(extension) ?? 0) + 1);
    bytes += statSync(file).size;
  }
  const meshCount = ['.stl', '.obj', '.ply'].reduce((sum, ext) => sum + (counts.get(ext) ?? 0), 0);
  findings.push(finding(meshCount > 0 ? 'pass' : 'warn', 'assets', `Dental mesh assets counted: ${meshCount}`, `scannedFiles=${files.length} bytes=${bytes}`));
  findings.push(finding('pass', 'assets', 'Build IO budget applied: heavy asset traversal is isolated to assets phase'));
  return findings;
}

function runtimePhase(): Finding[] {
  const findings = checkRequiredPaths('runtime', [
    'frontend/App.tsx',
    'frontend/runtime/LocalRuntimeBridge.tsx',
    'Tauri/src/lib.rs',
    'docs/tlanticad-recovery-gate.md',
  ]);
  const source = readText(resolve(REPO_ROOT, 'Tauri/src/lib.rs'));
  findings.push(
    source.includes('local_backend_endpoint')
      ? finding('pass', 'runtime', 'Runtime exposes local backend endpoint command')
      : finding('fail', 'runtime', 'Runtime does not expose local backend endpoint command'),
  );
  findings.push(
    source.includes('open_workspace_window') && source.includes('format!("/?{}"')
      ? finding('pass', 'runtime', 'Workspace windows use Next-compatible root query route')
      : finding('fail', 'runtime', 'Workspace windows must open /?workspace=... for Next/Tauri parity'),
  );
  const app = readText(resolve(REPO_ROOT, 'frontend/App.tsx'));
  findings.push(
    app.includes('buildWorkspacePreloaderState') && app.includes('onSyncWorkspaceContext')
      ? finding('pass', 'runtime', 'App shell connects preloader, TlantiDB and TlantiCAD context')
      : finding('fail', 'runtime', 'App shell does not connect preloader, TlantiDB and TlantiCAD context'),
  );
  const dbShell = readText(resolve(REPO_ROOT, 'frontend/workspaces/TlantiDB.tsx'));
  findings.push(
    dbShell.includes('TlantiDbWorkloadWizard') && dbShell.includes('onOpenModule={openCaseModuleFromBrowser}')
      ? finding('pass', 'runtime', 'TlantiDB connects workload wizard and case-browser module launch')
      : finding('fail', 'runtime', 'TlantiDB must connect workload wizard and case-browser module launch'),
  );
  const cadWorkspace = readText(resolve(REPO_ROOT, 'frontend/workspaces/tlanti-cad/TlantiCadWorkspace.tsx'));
  findings.push(
    cadWorkspace.includes('DicomClinicalWorkspace') && cadWorkspace.includes("moduleId === 'dicom'") && !cadWorkspace.includes('volview-react')
      ? finding('pass', 'runtime', 'DICOM module lazy-loads canonical clinical workspace')
      : finding('fail', 'runtime', 'DICOM module must lazy-load features/dicom-viewer DicomClinicalWorkspace, not volview-react'),
  );
  const clinicalWorkspace = readText(resolve(REPO_ROOT, 'frontend/workspaces/tlanti-cad/features/dicom-viewer/ui/DicomClinicalWorkspace.tsx'));
  for (const required of ['dicomSeriesImportStart', 'dicomVolumeBuildStart', 'dicomSegmentationStart', 'dicomSegmentationToMeshStart', 'slicerClinicalJobStart', 'slicerModelsDownloadAll', 'trame_slicer_sidecar_start', 'trame_slicer_sidecar_status']) {
    findings.push(
      clinicalWorkspace.includes(required)
        ? finding('pass', 'runtime', `DICOM clinical workspace owns ${required}`)
        : finding('fail', 'runtime', `DICOM clinical workspace missing ${required}`),
    );
  }
  return findings;
}

function runPhase(phase: Phase): Finding[] {
  const startedAt = performance.now();
  let findings: Finding[];
  switch (phase) {
    case 'source':
      findings = sourcePhase();
      break;
    case 'frontend':
      findings = frontendPhase();
      break;
    case 'ipc':
      findings = ipcPhase();
      break;
    case 'tauri':
      findings = tauriPhase();
      break;
    case 'python':
      findings = pythonPhase();
      break;
    case 'assets':
      findings = assetsPhase();
      break;
    case 'runtime':
      findings = runtimePhase();
      break;
  }
  findings.push(finding('pass', phase, `Phase timing: ${phase}`, `${Math.round(performance.now() - startedAt)}ms`));
  return findings;
}

const requestedPhases = parseArgs();
const findings = requestedPhases.flatMap(runPhase);
let failures = 0;
let warnings = 0;

for (const item of findings) {
  if (item.severity === 'fail') failures += 1;
  if (item.severity === 'warn') warnings += 1;
  const prefix = item.severity.toUpperCase();
  console.log(`[${prefix}] [${item.phase}] ${item.message}`);
  if (item.detail) console.log(`      ${item.detail}`);
}

console.log(`\nrecovery:gate phases=${requestedPhases.join(',')} failures=${failures} warnings=${warnings}`);
if (failures > 0) process.exit(1);
