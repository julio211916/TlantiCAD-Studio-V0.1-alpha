import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

type CheckLevel = 'pass' | 'warn' | 'fail';

interface CheckResult {
    id: string;
    label: string;
    level: CheckLevel;
    detail: string;
    evidence: string[];
}

const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const FRONTEND_ROOT = path.resolve(SCRIPT_DIR, '..');
const ROOT = path.resolve(FRONTEND_ROOT, '..');
const REPORT_DIR = path.join(ROOT, 'docs', 'qa');
const REPORT_JSON = path.join(REPORT_DIR, 'agent-5-local-runtime-report.json');
const REPORT_MD = path.join(REPORT_DIR, 'agent-5-local-runtime-report.md');
const STRICT_COMMAND_COVERAGE = process.argv.includes('--strict-command-coverage');
const STRICT_CHUNKS = process.argv.includes('--strict-chunks');

const QA_CODE_PATHS = [
    'frontend/workspaces/tlanti-cad/features/ai-runtime',
    'frontend/workspaces/tlanti-cad/features/dicom-viewer',
    'frontend/lib/dicom-jobs.ts',
    'frontend/lib/slicer-clinical.ts',
    'frontend/lib/ipc/contracts.ts',
    'frontend/lib/launcher-pending-import.ts',
    'Tauri/src/python_runtime.rs',
    'Tauri/src/trame_slicer_sidecar.rs',
    'Tauri/backend/python/slicer_automated_dental_runtime.py',
    'Tauri/resources/slicer/models/manifest.json',
    'Tauri/src/dental_model_seg.rs',
    'Tauri/src/dicom_jobs.rs',
    'Tauri/src/dicom_segmentation_jobs.rs',
    'Tauri/src/cad_dicom_seg_and_guide.rs',
    'frontend/scripts',
];

const DICOM_JOB_COMMANDS = [
    'dicom_import_prepare_from_path',
    'dicom_series_import_cancel',
    'dicom_series_import_start',
    'dicom_series_job_status',
    'dicom_volume_build_start',
    'dicom_volume_job_status',
];

const DICOM_SEGMENTATION_COMMANDS = [
    'dicom_segmentation_start',
    'dicom_segmentation_job_status',
    'dicom_segmentation_cancel',
    'dicom_segmentation_to_mesh_start',
];

const AI_RUNTIME_COMMANDS = [
    'get_python_bridge_status',
    'sidecar_status',
    'trame_slicer_sidecar_status',
    'trame_slicer_sidecar_start',
    'trame_slicer_sidecar_stop',
    'slicer_runtime_status',
    'slicer_runtime_download',
    'slicer_models_status',
    'slicer_models_download_all',
    'slicer_clinical_job_start',
    'slicer_clinical_job_status',
    'slicer_clinical_job_cancel',
    'get_dental_model_seg_status',
    'run_dental_model_segmentation',
    'stop_dental_model_seg_sidecar',
];

const REQUIRED_COMMANDS = [
    ...DICOM_JOB_COMMANDS,
    ...DICOM_SEGMENTATION_COMMANDS,
    ...AI_RUNTIME_COMMANDS,
];

function walkFiles(relativePath: string): string[] {
    const absolutePath = path.join(ROOT, relativePath);
    if (!existsSync(absolutePath)) return [];
    const stats = statSync(absolutePath);
    if (stats.isFile()) return [relativePath];

    return readdirSync(absolutePath).flatMap((entry) => {
        if (['node_modules', 'target', 'dist', '.git'].includes(entry)) return [];
        return walkFiles(path.join(relativePath, entry));
    });
}

function read(relativePath: string): string {
    return readFileSync(path.join(ROOT, relativePath), 'utf8');
}

function parseTauriGenerateHandlerCommands(source: string): Set<string> {
    const match = source.match(/generate_handler!\s*\[([\s\S]*?)\]\)/m);
    const body = match?.[1] ?? '';
    return new Set(
        [...body.matchAll(/(?:[a-zA-Z_][a-zA-Z0-9_]*::)*([a-zA-Z_][a-zA-Z0-9_]*)\s*,?/g)]
            .map((item) => item[1])
            .filter((name) => !['tauri', 'generate_handler'].includes(name)),
    );
}

function parseLiteralCommandCalls(source: string): Set<string> {
    return new Set(
        [
            ...source.matchAll(
                /['"`]((?:dicom|slicer|trame_slicer|sidecar|get_python|get_dental|run_dental|stop_dental)[a-z0-9_]+)['"`]/g,
            ),
        ]
            .map((item) => item[1]),
    );
}

function levelForWarnings(warnings: string[], strict = false): CheckLevel {
    if (warnings.length === 0) return 'pass';
    return strict ? 'fail' : 'warn';
}

function checkOfflineImports(files: string[]): CheckResult {
    const violations: string[] = [];
    const remoteUrl = /https?:\/\/(?!127\.0\.0\.1|localhost|\[::1\]|::1)/;
    const genericBackendOrigin = /\bBACKEND_ORIGIN\b/;
    const blockedCloudPackage = ['@google', 'genai'].join('/');

    for (const file of files) {
        if (file === 'frontend/scripts/agent5-dicom-ai-qa.ts') continue;
        const source = read(file);
        if (source.includes(blockedCloudPackage)) {
            violations.push(`${file}: imports blocked cloud AI package`);
        }
        if (genericBackendOrigin.test(source)) {
            violations.push(`${file}: imports generic BACKEND_ORIGIN`);
        }
        if (remoteUrl.test(source)) {
            violations.push(`${file}: contains non-loopback HTTP URL`);
        }
    }

    return {
        id: 'offline-imports',
        label: 'Offline imports',
        level: violations.length ? 'fail' : 'pass',
        detail: violations.length
            ? 'Owned DICOM/IA paths still reference remote or generic backend imports.'
            : 'Owned DICOM/IA paths avoid remote APIs and generic backend origins.',
        evidence: violations,
    };
}

function checkHttpCapabilityGates(): CheckResult {
    const adapterFiles = [
        'frontend/workspaces/tlanti-cad/features/ai-runtime/infrastructure/backend-ai-runtime-adapter.ts',
        'frontend/workspaces/tlanti-cad/features/dicom-viewer/infrastructure/backend-segmentation-adapter.ts',
    ];
    const missing = adapterFiles.filter((file) => {
        const source = read(file);
        return !source.includes('enabled?: boolean') || !source.includes('assertCapability');
    });

    return {
        id: 'http-capability-gates',
        label: 'HTTP adapter capability gates',
        level: missing.length ? 'fail' : 'pass',
        detail: missing.length
            ? 'One or more generic HTTP adapters can run without an explicit capability gate.'
            : 'Generic HTTP adapters are explicit opt-in and loopback constrained.',
        evidence: missing,
    };
}

function checkCancellation(): CheckResult {
    const segmentationAdapter = read('frontend/workspaces/tlanti-cad/features/dicom-viewer/infrastructure/backend-segmentation-adapter.ts');
    const runner = read('frontend/workspaces/tlanti-cad/features/dicom-viewer/ui/useSegmentationRunner.ts');
    const violations: string[] = [];

    if (/async\s+cancel\s*\(\s*_jobId/.test(segmentationAdapter)) {
        violations.push('backend-segmentation-adapter.ts still ignores jobId in cancel().');
    }
    if (/no-op|returns without error/i.test(segmentationAdapter)) {
        violations.push('backend-segmentation-adapter.ts still documents silent no-op cancellation.');
    }
    if (!segmentationAdapter.includes('/cancel')) {
        violations.push('backend-segmentation-adapter.ts does not call a cancel endpoint.');
    }
    if (!runner.includes('Cancellation failed:')) {
        violations.push('useSegmentationRunner.ts does not surface cancellation failure in job state.');
    }

    return {
        id: 'cancellation',
        label: 'Cancellation behavior',
        level: violations.length ? 'fail' : 'pass',
        detail: violations.length
            ? 'Cancellation can still fail silently.'
            : 'Cancellation failure is visible and the HTTP adapter no longer no-ops.',
        evidence: violations,
    };
}

function checkStubMarkers(files: string[]): CheckResult {
    const criticalMarkers = [
        /fake success/i,
        /silent fake/i,
        /TODO:\s*implement/i,
        /throw new Error\(['"`]TODO/i,
        /return Promise\.resolve\(\)/,
    ];
    const findings: string[] = [];

    for (const file of files) {
        if (file === 'frontend/scripts/agent5-dicom-ai-qa.ts') continue;
        const source = read(file);
        const lines = source.split('\n');
        for (let lineIndex = 0; lineIndex < lines.length; lineIndex += 1) {
            const line = lines[lineIndex];
            if (/\bnever\b/i.test(line)) continue;
            for (const marker of criticalMarkers) {
                if (marker.test(line)) {
                    findings.push(`${file}:${lineIndex + 1}: ${marker}`);
                }
            }
        }
    }

    return {
        id: 'stub-markers',
        label: 'Critical stub markers',
        level: findings.length ? 'fail' : 'pass',
        detail: findings.length
            ? 'Critical stub/no-real-work markers remain in owned paths.'
            : 'No critical fake-success or TODO implementation markers found in owned paths.',
        evidence: findings,
    };
}

function checkCommandCoverage(): CheckResult {
    const activeLibPath = 'Tauri/src/lib.rs';
    const alternateLibPath = 'Tauri/src/lib 2.rs';
    const activeSource = existsSync(path.join(ROOT, activeLibPath)) ? read(activeLibPath) : '';
    const alternateSource = existsSync(path.join(ROOT, alternateLibPath)) ? read(alternateLibPath) : '';
    const activeCommands = parseTauriGenerateHandlerCommands(activeSource);
    const contractsSource = read('frontend/lib/ipc/contracts.ts');
    const dicomJobsSource = read('frontend/lib/dicom-jobs.ts');
    const clinicalWorkspaceSource = read('frontend/workspaces/tlanti-cad/features/dicom-viewer/ui/DicomClinicalWorkspace.tsx');
    const frontendCommands = new Set([
        ...parseLiteralCommandCalls(dicomJobsSource),
        ...parseLiteralCommandCalls(clinicalWorkspaceSource),
    ]);
    const missingInActive = REQUIRED_COMMANDS.filter((command) => !activeCommands.has(command));
    const missingInContracts = REQUIRED_COMMANDS.filter((command) => !contractsSource.includes(`${command}:`));
    const frontendMissingInTauri = [...frontendCommands]
        .filter((command) =>
            /^(dicom_|slicer_|trame_slicer_|sidecar_|get_python|get_dental|run_dental|stop_dental)/.test(command),
        )
        .filter((command) => !activeCommands.has(command));
    const presentInAlternate = REQUIRED_COMMANDS.filter((command) => alternateSource.includes(command));
    const evidence = [
        ...missingInActive.map((command) => `${activeLibPath}: missing ${command}`),
        ...missingInContracts.map((command) => `frontend/lib/ipc/contracts.ts: missing ${command}`),
        ...frontendMissingInTauri.map((command) => `frontend caller references ${command}, but active Tauri handler does not register it`),
        ...presentInAlternate.map((command) => `${alternateLibPath}: contains ${command}`),
    ];
    const warnings = [...missingInActive, ...missingInContracts, ...frontendMissingInTauri];

    return {
        id: 'command-coverage',
        label: 'Tauri command coverage',
        level: levelForWarnings(warnings, STRICT_COMMAND_COVERAGE),
        detail: warnings.length
            ? 'Required DICOM/IA local commands are not all registered in the active Tauri invoke handler.'
            : 'Required DICOM/IA local commands are registered in the active Tauri invoke handler.',
        evidence,
    };
}

function checkPythonDicomVtkContract(): CheckResult {
    const requirementsPath = 'Tauri/backend/python/requirements.txt';
    const healthcheckPath = 'Tauri/backend/python/dicom_healthcheck.py';
    const vtkHealthcheckPath = 'Tauri/backend/python/vtk_healthcheck.py';
    const runtimePath = 'Tauri/src/python_runtime.rs';
    const clinicalSmokePath = 'frontend/scripts/clinical-hardening-smoke.ts';
    const slicerRuntimePath = 'Tauri/backend/python/slicer_automated_dental_runtime.py';
    const slicerManifestPath = 'Tauri/resources/slicer/models/manifest.json';
    const findings: string[] = [];

    if (!existsSync(path.join(ROOT, requirementsPath)) || !read(requirementsPath).includes('pydicom==')) {
        findings.push(`${requirementsPath}: missing pinned pydicom dependency`);
    }
    if (!existsSync(path.join(ROOT, requirementsPath)) || !read(requirementsPath).includes('vtk==')) {
        findings.push(`${requirementsPath}: missing pinned vtk dependency`);
    }
    if (!existsSync(path.join(ROOT, healthcheckPath)) || !read(healthcheckPath).includes('pydicom.dcmread')) {
        findings.push(`${healthcheckPath}: missing pydicom.dcmread healthcheck`);
    }
    if (!existsSync(path.join(ROOT, vtkHealthcheckPath)) || !read(vtkHealthcheckPath).includes('vtkSphereSource')) {
        findings.push(`${vtkHealthcheckPath}: missing VTK smoke healthcheck`);
    }
    if (!read(runtimePath).includes('get_python_bridge_status')) {
        findings.push(`${runtimePath}: missing Python bridge status command`);
    }
    if (!read(runtimePath).includes('sidecar_status')) {
        findings.push(`${runtimePath}: missing sidecar status command`);
    }
    if (!read(clinicalSmokePath).includes('runPydicomHealthcheck')) {
        findings.push(`${clinicalSmokePath}: smoke fixture does not validate DICOM through pydicom`);
    }
    if (!read(clinicalSmokePath).includes('vtkVersion')) {
        findings.push(`${clinicalSmokePath}: smoke fixture does not report VTK availability`);
    }
    if (!existsSync(path.join(ROOT, slicerRuntimePath)) || !read(slicerRuntimePath).includes('download_all_models')) {
        findings.push(`${slicerRuntimePath}: missing SlicerAutomatedDentalTools model downloader`);
    }
    if (!existsSync(path.join(ROOT, slicerManifestPath))) {
        findings.push(`${slicerManifestPath}: missing Slicer model manifest`);
    } else {
        const manifest = read(slicerManifestPath);
        for (const required of ['amasss-cbct-segmentation', 'dentalsegmentator-adult-nnunet', 'ali-cbct-landmarks', 'aso-cbct-orientation', 'clic-impacted-canines']) {
            if (!manifest.includes(required)) {
                findings.push(`${slicerManifestPath}: missing required clinical model ${required}`);
            }
        }
        if (!manifest.includes('"silentDownload": true') || !manifest.includes('SlicerAutomatedDentalTools')) {
            findings.push(`${slicerManifestPath}: silent download policy or extension source missing`);
        }
    }

    return {
        id: 'python-dicom-vtk-contract',
        label: 'Python DICOM/VTK contract',
        level: findings.length ? 'fail' : 'pass',
        detail: findings.length
            ? 'Clinical DICOM/volume path is not fully owned by local Python/pydicom/VTK.'
            : 'Clinical DICOM fixture, VTK healthcheck and SlicerAutomatedDentalTools model downloader are explicitly owned locally.',
        evidence: findings,
    };
}

function checkChunks(): CheckResult {
    const budget = spawnSync('bun', ['run', 'bundle:budget'], {
        cwd: FRONTEND_ROOT,
        encoding: 'utf8',
    });
    const output = `${budget.stdout ?? ''}\n${budget.stderr ?? ''}`.trim();
    if (budget.status !== 0) {
        return {
            id: 'chunks',
            label: 'Frontend chunk budget',
            level: levelForWarnings([output], STRICT_CHUNKS),
            detail: 'Next bundle budget failed or the fresh .next/out assets are missing.',
            evidence: output.split('\n').slice(-16),
        };
    }

    const jsonStart = output.indexOf('{');
    const jsonEnd = output.lastIndexOf('}');
    const report = jsonStart >= 0 && jsonEnd > jsonStart ? JSON.parse(output.slice(jsonStart, jsonEnd + 1)) as {
        chunksDir?: string;
        totalJsBytes?: number;
        forbiddenOutputReferences?: string[];
        results?: Array<{ label: string; total: number; largest?: { file: string; bytes: number } | null }>;
    } : null;
    const findings = report?.forbiddenOutputReferences ?? [];
    const evidence = [
        `chunksDir: ${report?.chunksDir ?? '<unknown>'}`,
        `totalJsBytes: ${report?.totalJsBytes ?? '<unknown>'}`,
        ...(report?.results ?? []).map((result) => {
            const largest = result.largest ? `${result.largest.file} (${Math.round(result.largest.bytes / 1024)} KiB)` : '<none>';
            return `${result.label}: ${Math.round(result.total / 1024)} KiB total, largest ${largest}`;
        }),
    ];

    return {
        id: 'chunks',
        label: 'Frontend chunk budget',
        level: levelForWarnings(findings, STRICT_CHUNKS),
        detail: findings.length
            ? 'Next bundle budget found forbidden output references.'
            : 'Next bundle budget passed for TlantiDB, DICOM/VTK/trame, CAD/Three and AI chunks.',
        evidence: [...findings, ...evidence],
    };
}

function renderMarkdown(results: CheckResult[]): string {
    const lines = [
        '# Agent 5 DICOM/IA Local QA Report',
        '',
        `Generated: ${new Date().toISOString()}`,
        '',
        '| Check | Level | Detail |',
        '| --- | --- | --- |',
        ...results.map((result) => `| ${result.label} | ${result.level} | ${result.detail.replace(/\|/g, '\\|')} |`),
        '',
        '## Evidence',
        '',
    ];

    for (const result of results) {
        lines.push(`### ${result.label}`, '');
        if (result.evidence.length === 0) {
            lines.push('- No findings.', '');
            continue;
        }
        for (const item of result.evidence) {
            lines.push(`- ${item}`);
        }
        lines.push('');
    }

    lines.push(
        '## Performance Gate Notes',
        '',
        '- Browser DICOM imports over 512 MiB or 512 instances should move to Tauri chunked reads.',
        '- Clinical DICOM metadata, anonymization and study inspection must use local Python/pydicom before pixel/volume work enters AI.',
        '- DICOM volume-to-mesh gates should use local Python/VTK for volume IO, marching cubes and mesh preprocessing before Rust artifact persistence.',
        '- Generic HTTP adapters must remain disabled unless a local loopback sidecar owns the capability.',
        '- Cancellation must produce a visible failed/cancelled state; no silent no-op is acceptable.',
        '- Chunk regressions should be fixed in frontend/scripts/bundle-budget.ts budgets for TlantiDB, DICOM/VTK/trame, CAD/Three and AI.',
        '',
    );

    return `${lines.join('\n')}\n`;
}

function main() {
    const codeFiles = QA_CODE_PATHS.flatMap(walkFiles).filter((file) => /\.(ts|tsx|rs)$/.test(file));
    const results = [
        checkOfflineImports(codeFiles),
        checkHttpCapabilityGates(),
        checkCancellation(),
        checkStubMarkers(codeFiles),
        checkCommandCoverage(),
        checkPythonDicomVtkContract(),
        checkChunks(),
    ];

    mkdirSync(REPORT_DIR, { recursive: true });
    writeFileSync(
        REPORT_JSON,
        JSON.stringify(
            {
                generatedAt: new Date().toISOString(),
                strictCommandCoverage: STRICT_COMMAND_COVERAGE,
                strictChunks: STRICT_CHUNKS,
                results,
            },
            null,
            2,
        ),
    );
    writeFileSync(REPORT_MD, renderMarkdown(results));

    for (const result of results) {
        const icon = result.level === 'pass' ? 'PASS' : result.level === 'warn' ? 'WARN' : 'FAIL';
        console.log(`${icon} ${result.label}: ${result.detail}`);
    }
    console.log(`Report: ${path.relative(ROOT, REPORT_MD)}`);

    if (results.some((result) => result.level === 'fail')) {
        process.exitCode = 1;
    }
}

main();
