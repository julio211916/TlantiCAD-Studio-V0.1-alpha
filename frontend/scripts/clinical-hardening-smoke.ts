import { spawn, type ChildProcessWithoutNullStreams } from 'node:child_process';
import { once } from 'node:events';
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { setTimeout as delay } from 'node:timers/promises';

import puppeteer from 'puppeteer';

const BASE_URL = process.env.SMOKE_BASE_URL ?? 'http://127.0.0.1:3000/';
const SHOULD_SPAWN_VITE = !process.argv.includes('--no-spawn');
const ROOT = process.cwd();
const FIXTURE_DIR = path.join(ROOT, 'fixtures', 'clinical-smoke');
const REPORT_DIR = path.join(ROOT, 'docs', 'qa');
const STL_FIXTURE = path.join(FIXTURE_DIR, 'prep-scan-tooth-26.stl');
const DICOM_FIXTURE = path.join(FIXTURE_DIR, 'cbct-metadata-fixture.dcm');
const PYDICOM_HEALTHCHECK = path.join(ROOT, 'backend', 'python', 'dicom_healthcheck.py');

interface ClinicalSmokeReport {
  generatedAt: string;
  baseUrl: string;
  fixture: {
    stlPath: string;
    stlFacets: number;
    dicomPath: string;
    dicomBytes: number;
    dicomHasPreamble: boolean;
    pydicomVersion: string;
    vtkVersion: string;
    pydicomPatientId: string;
    pydicomModality: string;
    pydicomRows: number;
    pydicomColumns: number;
  };
  ui: {
    shellVisible: boolean;
    canvasVisible: boolean;
    clinicalWorkflowStageCount: number;
    exportBlockedWithoutDesktop: boolean;
    alignmentModalVisible: boolean;
    blockedBannerVisible: boolean;
  };
  chunks: Array<{ file: string; bytes: number; warning: boolean }>;
}

function writeElement(buffer: Buffer[], tagGroup: number, tagElement: number, vr: string, value: string | Buffer) {
  const rawValue = typeof value === 'string' ? Buffer.from(value, 'ascii') : value;
  const valueBytes = rawValue.length % 2 === 0 ? rawValue : Buffer.concat([rawValue, Buffer.from([0x20])]);
  const header = Buffer.alloc(8);
  header.writeUInt16LE(tagGroup, 0);
  header.writeUInt16LE(tagElement, 2);
  header.write(vr, 4, 2, 'ascii');
  header.writeUInt16LE(valueBytes.length, 6);
  buffer.push(header, valueBytes);
}

function ensureDicomFixture() {
  mkdirSync(FIXTURE_DIR, { recursive: true });
  const chunks: Buffer[] = [];
  chunks.push(Buffer.alloc(128, 0), Buffer.from('DICM', 'ascii'));
  writeElement(chunks, 0x0002, 0x0002, 'UI', '1.2.840.10008.5.1.4.1.1.2');
  writeElement(chunks, 0x0002, 0x0010, 'UI', '1.2.840.10008.1.2.1');
  writeElement(chunks, 0x0008, 0x0060, 'CS', 'CT');
  writeElement(chunks, 0x0010, 0x0010, 'PN', 'SERRANO^ROSA');
  writeElement(chunks, 0x0010, 0x0020, 'LO', '2026-1485');
  writeElement(chunks, 0x0020, 0x000d, 'UI', '1.2.826.0.1.3680043.10.2026.1485');
  writeElement(chunks, 0x0028, 0x0010, 'US', Buffer.from([64, 0]));
  writeElement(chunks, 0x0028, 0x0011, 'US', Buffer.from([64, 0]));
  writeFileSync(DICOM_FIXTURE, Buffer.concat(chunks));
}

function candidatePythonPaths() {
  const configured = process.env.TLANTI_PYTHON_PATH;
  const venvPython = process.platform === 'win32'
    ? path.join(ROOT, '.tlanticad', 'python', '.venv', 'Scripts', 'python.exe')
    : path.join(ROOT, '.tlanticad', 'python', '.venv', 'bin', 'python3');
  return [configured, venvPython, 'python3', 'python'].filter(Boolean) as string[];
}

async function runPydicomHealthcheck(dicomPath: string) {
  let lastError = '';
  for (const pythonPath of candidatePythonPaths()) {
    try {
      const child = spawn(pythonPath, [PYDICOM_HEALTHCHECK, dicomPath], {
        cwd: ROOT,
        stdio: ['ignore', 'pipe', 'pipe'],
      });
      const stdoutChunks: Buffer[] = [];
      const stderrChunks: Buffer[] = [];
      child.stdout.on('data', (chunk) => stdoutChunks.push(Buffer.from(chunk)));
      child.stderr.on('data', (chunk) => stderrChunks.push(Buffer.from(chunk)));
      const [code] = await once(child, 'exit') as [number | null];
      const stdout = Buffer.concat(stdoutChunks).toString('utf8').trim();
      const stderr = Buffer.concat(stderrChunks).toString('utf8').trim();
      if (code === 0) {
        return JSON.parse(stdout) as {
          ok: boolean;
          pydicomVersion: string;
          vtkVersion: string;
          patientId: string;
          modality: string;
          rows: number;
          columns: number;
        };
      }
      lastError = `${pythonPath}: ${stderr || stdout || `exit ${code}`}`;
    } catch (error) {
      lastError = `${pythonPath}: ${error instanceof Error ? error.message : String(error)}`;
    }
  }

  throw new Error(
    `pydicom and VTK are required for DICOM clinical smoke. Install with: python3 -m pip install -r backend/python/requirements.txt. Last error: ${lastError}`,
  );
}

async function inspectFixtures() {
  if (!existsSync(STL_FIXTURE)) {
    throw new Error(`Missing STL fixture: ${STL_FIXTURE}`);
  }
  ensureDicomFixture();
  const stl = readFileSync(STL_FIXTURE, 'utf8');
  const dicom = readFileSync(DICOM_FIXTURE);
  const stlFacets = (stl.match(/facet normal/g) ?? []).length;
  if (stlFacets < 12) {
    throw new Error(`STL fixture is too small for clinical smoke: ${stlFacets} facets`);
  }
  if (dicom.subarray(128, 132).toString('ascii') !== 'DICM') {
    throw new Error('DICOM fixture is missing Part 10 DICM preamble');
  }
  const pydicom = await runPydicomHealthcheck(DICOM_FIXTURE);

  return {
    stlPath: STL_FIXTURE,
    stlFacets,
    dicomPath: DICOM_FIXTURE,
    dicomBytes: dicom.byteLength,
    dicomHasPreamble: true,
    pydicomVersion: pydicom.pydicomVersion,
    vtkVersion: pydicom.vtkVersion,
    pydicomPatientId: pydicom.patientId,
    pydicomModality: pydicom.modality,
    pydicomRows: pydicom.rows,
    pydicomColumns: pydicom.columns,
  };
}

function collectChunkReport() {
  const assetsDir = path.join(ROOT, 'dist', 'assets');
  if (!existsSync(assetsDir)) return [];
  const files = Array.from(new Set(readdirSync(assetsDir)));
  return files
    .filter((file) => file.endsWith('.js'))
    .map((file) => {
      const bytes = statSync(path.join(assetsDir, file)).size;
      return { file, bytes, warning: bytes > 750_000 };
    })
    .sort((a, b) => b.bytes - a.bytes);
}

async function waitForServer(url: string, attempts = 90) {
  for (let index = 0; index < attempts; index += 1) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Server not ready yet.
    }
    await delay(1000);
  }
  throw new Error(`Smoke server did not become ready: ${url}`);
}

function startViteDev() {
  const viteCommand = process.platform === 'win32' ? 'node_modules/.bin/vite.cmd' : 'node_modules/.bin/vite';
  const child = spawn(viteCommand, ['--host', '127.0.0.1', '--port', '3000', '--strictPort'], {
    cwd: ROOT,
    env: process.env,
    stdio: 'pipe',
  });
  child.stdout.on('data', (chunk) => process.stdout.write(chunk));
  child.stderr.on('data', (chunk) => process.stderr.write(chunk));
  return child;
}

async function runUiSmoke() {
  const browser = await puppeteer.launch({
    headless: true,
    defaultViewport: { width: 1536, height: 960 },
  });

  try {
    const page = await browser.newPage();
    const url = new URL(BASE_URL);
    url.searchParams.set('workspace', 'tlanticad');
    url.searchParams.set('caseId', 'clinical-hardening-smoke');
    url.searchParams.set('module', 'crown');
    await page.goto(url.toString(), { waitUntil: 'domcontentloaded', timeout: 45_000 });
    await page.waitForSelector('[data-testid="cad-desktop-shell"]', { timeout: 30_000 });
    await page.waitForSelector('[data-testid="cad-three-viewport"]', { timeout: 30_000 });
    await page.waitForSelector('[data-testid="clinical-crown-panel"]', { timeout: 30_000 });

    const qa = await page.evaluate(() => {
      const api = window.__TLANTICAD_CAD_SHELL_QA__;
      if (!api) throw new Error('Missing CAD shell QA API');
      const workflow = api.getClinicalWorkflow();
      const snapshot = api.getSnapshot();
      return {
        workflow,
        entityCount: snapshot.entities.length,
        activeTool: snapshot.activeTool,
        gridVisible: snapshot.gridVisible,
      };
    });

    await page.click('[data-testid="confirm-alignment"]');
    await page.waitForFunction(() => !document.querySelector('[data-testid="crown-alignment-modal"]'));
    await page.click('[data-testid="run-crown-slice"]');
    await page.waitForSelector('[data-testid="crown-export-blocked"]', { timeout: 5000 });

    return {
      shellVisible: true,
      canvasVisible: true,
      clinicalWorkflowStageCount: qa.workflow.stageCount,
      exportBlockedWithoutDesktop: qa.workflow.exportBlockedWithoutDesktop,
      alignmentModalVisible: true,
      blockedBannerVisible: true,
    };
  } finally {
    await browser.close();
  }
}

function writeReport(report: ClinicalSmokeReport) {
  mkdirSync(REPORT_DIR, { recursive: true });
  writeFileSync(
    path.join(REPORT_DIR, 'clinical-hardening-smoke-report.json'),
    `${JSON.stringify(report, null, 2)}\n`,
  );
  const chunkLines = report.chunks
    .slice(0, 12)
    .map((chunk) => `- ${chunk.warning ? 'WARN' : 'OK'} ${chunk.file}: ${Math.round(chunk.bytes / 1024)} KiB`)
    .join('\n');
  writeFileSync(
    path.join(REPORT_DIR, 'clinical-hardening-smoke-report.md'),
    [
      '# Clinical Hardening Smoke Report',
      '',
      `Generated: ${report.generatedAt}`,
      '',
      '## Fixtures',
      '',
      `- STL: ${report.fixture.stlPath} (${report.fixture.stlFacets} facets)`,
      `- DICOM: ${report.fixture.dicomPath} (${report.fixture.dicomBytes} bytes, Part 10 preamble: ${report.fixture.dicomHasPreamble})`,
      `- pydicom: ${report.fixture.pydicomVersion} (${report.fixture.pydicomModality}, ${report.fixture.pydicomRows}x${report.fixture.pydicomColumns}, patient ${report.fixture.pydicomPatientId})`,
      `- VTK: ${report.fixture.vtkVersion} available for volume-to-mesh gates`,
      '',
      '## Guided UI Smoke',
      '',
      `- Shell visible: ${report.ui.shellVisible}`,
      `- Canvas visible: ${report.ui.canvasVisible}`,
      `- Crown stages: ${report.ui.clinicalWorkflowStageCount}`,
      `- Browser export blocked without desktop runtime: ${report.ui.exportBlockedWithoutDesktop}`,
      `- Crown blocked banner visible: ${report.ui.blockedBannerVisible}`,
      '',
      '## Chunk Pressure',
      '',
      chunkLines || '- No build assets found. Run `npm run build` before reading chunk pressure.',
      '',
    ].join('\n'),
  );
}

async function main() {
  const fixture = await inspectFixtures();
  let viteProcess: ChildProcessWithoutNullStreams | null = null;

  try {
    if (SHOULD_SPAWN_VITE) {
      viteProcess = startViteDev();
      await waitForServer(BASE_URL);
    }

    const ui = await runUiSmoke();
    const report: ClinicalSmokeReport = {
      generatedAt: new Date().toISOString(),
      baseUrl: BASE_URL,
      fixture,
      ui,
      chunks: collectChunkReport(),
    };
    writeReport(report);
    console.log('Clinical hardening smoke passed.');
  } finally {
    if (viteProcess) {
      viteProcess.kill('SIGTERM');
      await Promise.race([once(viteProcess, 'exit'), delay(4000)]);
    }
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
