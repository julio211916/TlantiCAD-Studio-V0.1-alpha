import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

type Level = 'pass' | 'warn' | 'fail';

interface Asset {
  file: string;
  bytes: number;
}

interface LoadableEntry {
  files?: string[];
}

interface BudgetRule {
  id: string;
  label: string;
  ownerPattern: RegExp;
  heavyPattern?: RegExp;
  maxOwnedBytes: number;
  maxSingleChunkBytes: number;
}

const forbiddenOutputPatterns = [
  {
    label: 'VolView placeholder workspace',
    pattern: /VolViewDentalWorkspace|frontend\/volview-react|@\/volview-react/i,
  },
  {
    label: 'legacy Vite budget contract',
    pattern: /dist\/assets|\.vite\/manifest\.json/i,
  },
];

const frontendRoot = process.cwd();
const nextRoot = join(frontendRoot, '.next');
const outChunksDir = join(frontendRoot, 'out', '_next', 'static', 'chunks');
const fallbackChunksDir = join(frontendRoot, '.next', 'static', 'chunks');
const loadableManifestPath = join(nextRoot, 'react-loadable-manifest.json');
const buildManifestPath = join(nextRoot, 'build-manifest.json');

const rules: BudgetRule[] = [
  {
    id: 'tlantidb-shell',
    label: 'TlantiDB shell',
    ownerPattern: /workspaces\/tlanti-db|TlantiDB/i,
    heavyPattern: /cornerstone|dicom|vtk|three|@react-three|transformers|onnx|cadinterface|cadworkspace/i,
    maxOwnedBytes: 950 * 1024,
    maxSingleChunkBytes: 360 * 1024,
  },
  {
    id: 'dicom-vtk',
    label: 'DICOM / VTK / trame bridge',
    ownerPattern: /dicom|Dicom|trame|slicer|cornerstone|vtk|VolView/i,
    maxOwnedBytes: 2_400 * 1024,
    maxSingleChunkBytes: 1_600 * 1024,
  },
  {
    id: 'cad-three',
    label: 'CAD / Three',
    ownerPattern: /CadInterface|CadWorkspace|three|@react-three|three-stdlib|mesh|cad/i,
    maxOwnedBytes: 5_000 * 1024,
    maxSingleChunkBytes: 1_700 * 1024,
  },
  {
    id: 'ai-runtime',
    label: 'AI / HuggingFace',
    ownerPattern: /ai-runtime|local-cad-copilot|transformers|onnx|huggingface/i,
    maxOwnedBytes: 1_400 * 1024,
    maxSingleChunkBytes: 900 * 1024,
  },
];

function readJson<T>(path: string): T {
  return JSON.parse(readFileSync(path, 'utf8')) as T;
}

function walkChunks(root: string): Asset[] {
  const assets: Asset[] = [];
  function visit(path: string) {
    for (const child of readdirSync(path)) {
      const absolute = join(path, child);
      const stats = statSync(absolute);
      if (stats.isDirectory()) {
        visit(absolute);
        continue;
      }
      if (stats.isFile() && /\.(js|css)$/.test(child)) {
        assets.push({
          file: relative(frontendRoot, absolute),
          bytes: stats.size,
        });
      }
    }
  }
  visit(root);
  return assets.sort((a, b) => b.bytes - a.bytes);
}

function normalizeChunkPath(file: string): string {
  return file
    .replace(/^out\/_next\/static\/chunks\//, '')
    .replace(/^\.next\/static\/chunks\//, '')
    .replace(/^static\/chunks\//, '')
    .replace(/^_next\/static\/chunks\//, '');
}

function buildAssetMap(assets: Asset[]) {
  const map = new Map<string, Asset>();
  for (const asset of assets) {
    map.set(normalizeChunkPath(asset.file), asset);
  }
  return map;
}

function findOwnedAssets(rule: BudgetRule, loadableManifest: Record<string, LoadableEntry>, assetByChunk: Map<string, Asset>): Asset[] {
  const owned = new Map<string, Asset>();
  for (const [owner, entry] of Object.entries(loadableManifest)) {
    if (!rule.ownerPattern.test(owner)) continue;
    for (const file of entry.files ?? []) {
      const asset = assetByChunk.get(normalizeChunkPath(file));
      if (asset) owned.set(asset.file, asset);
    }
  }
  return [...owned.values()].sort((a, b) => b.bytes - a.bytes);
}

function evaluateRule(rule: BudgetRule, ownedAssets: Asset[], loadableManifest: Record<string, LoadableEntry>) {
  const total = ownedAssets.reduce((sum, asset) => sum + asset.bytes, 0);
  const largest = ownedAssets[0] ?? null;
  const heavyLeaks: string[] = [];

  if (rule.heavyPattern) {
    for (const [owner, entry] of Object.entries(loadableManifest)) {
      if (!rule.ownerPattern.test(owner)) continue;
      if (rule.id === 'tlantidb-shell' && !/App\.tsx\s*->.*TlantiDbWorkspace/i.test(owner)) continue;
      const files = entry.files ?? [];
      if (rule.heavyPattern.test(owner) || files.some((file) => rule.heavyPattern?.test(file))) {
        heavyLeaks.push(`${owner} -> ${files.join(', ') || '<no emitted file>'}`);
      }
    }
  }

  const failures: string[] = [];
  if (total > rule.maxOwnedBytes) {
    failures.push(`${rule.label} owns ${total} bytes; budget is ${rule.maxOwnedBytes}`);
  }
  if (largest && largest.bytes > rule.maxSingleChunkBytes) {
    failures.push(`${rule.label} largest chunk ${largest.file} is ${largest.bytes} bytes; budget is ${rule.maxSingleChunkBytes}`);
  }
  if (heavyLeaks.length > 0) {
    failures.push(`${rule.label} initial/loadable graph references forbidden heavy owners`);
  }

  return {
    rule: rule.id,
    label: rule.label,
    level: failures.length ? 'fail' as Level : 'pass' as Level,
    total,
    maxOwnedBytes: rule.maxOwnedBytes,
    maxSingleChunkBytes: rule.maxSingleChunkBytes,
    largest,
    ownedAssets,
    heavyLeaks,
    failures,
  };
}

function findForbiddenOutputReferences(assets: Asset[], loadableManifest: Record<string, LoadableEntry>) {
  const offenders: string[] = [];
  const manifestText = JSON.stringify(loadableManifest);
  for (const forbidden of forbiddenOutputPatterns) {
    if (forbidden.pattern.test(manifestText)) {
      offenders.push(`react-loadable-manifest references ${forbidden.label}`);
    }
  }

  for (const asset of assets) {
    const contents = readFileSync(join(frontendRoot, asset.file), 'utf8');
    for (const forbidden of forbiddenOutputPatterns) {
      if (forbidden.pattern.test(contents)) {
        offenders.push(`${asset.file} contains ${forbidden.label}`);
      }
    }
  }

  return offenders;
}

function main() {
  const chunksDir = existsSync(outChunksDir) ? outChunksDir : fallbackChunksDir;
  if (!existsSync(chunksDir)) {
    throw new Error('Next chunks not found. Run `bun run --cwd frontend build` before bundle:budget.');
  }
  if (!existsSync(loadableManifestPath)) {
    throw new Error('Missing .next/react-loadable-manifest.json. Run `bun run --cwd frontend build` before bundle:budget.');
  }
  if (!existsSync(buildManifestPath)) {
    throw new Error('Missing .next/build-manifest.json. Run `bun run --cwd frontend build` before bundle:budget.');
  }

  const assets = walkChunks(chunksDir);
  const assetByChunk = buildAssetMap(assets);
  const loadableManifest = readJson<Record<string, LoadableEntry>>(loadableManifestPath);
  const buildManifest = readJson<Record<string, unknown>>(buildManifestPath);
  const results = rules.map((rule) => evaluateRule(rule, findOwnedAssets(rule, loadableManifest, assetByChunk), loadableManifest));
  const forbiddenOutputReferences = findForbiddenOutputReferences(assets, loadableManifest);
  const totalJsBytes = assets.filter((asset) => asset.file.endsWith('.js')).reduce((sum, asset) => sum + asset.bytes, 0);
  const largest = assets.slice(0, 16);

  const report = {
    chunksDir: relative(frontendRoot, chunksDir),
    totalAssets: assets.length,
    totalJsBytes,
    largest,
    rootMainFiles: (buildManifest as { rootMainFiles?: string[] }).rootMainFiles ?? [],
    forbiddenOutputReferences,
    results,
  };

  console.log(JSON.stringify(report, null, 2));

  const failures = results.flatMap((result) => result.failures.map((failure) => `${result.rule}: ${failure}`));
  failures.push(...forbiddenOutputReferences);
  if (failures.length > 0) {
    throw new Error(`Next bundle budget failed:\n${failures.join('\n')}`);
  }
}

main();
