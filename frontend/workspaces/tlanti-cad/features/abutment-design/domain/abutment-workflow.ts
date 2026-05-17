import type { CadCommandId, CadCommandOwner, CadCommandPayload, CadCommandRunResult } from '@/core';

export type AbutmentWorkflowStepId =
    | 'platform-context'
    | 'profile-margin'
    | 'collar-emergence'
    | 'surface-adaptation'
    | 'screw-channel'
    | 'validate-export';

export type AbutmentOperationId =
    | 'select-implant-platform'
    | 'create-cross-section-profile'
    | 'create-margin-loop'
    | 'generate-collar-body'
    | 'adapt-to-active-surface'
    | 'boolean-cut-screw-channel'
    | 'cleanup-abutment-mesh'
    | 'export-abutment-report';

export type AbutmentJobKind =
    | 'implant-platform-resolve'
    | 'abutment-cross-section'
    | 'abutment-margin-loop'
    | 'abutment-collar-body'
    | 'abutment-shrinkwrap'
    | 'abutment-boolean-cut'
    | 'abutment-mesh-cleanup'
    | 'abutment-export-package';

export interface AbutmentScriptPatternReference {
    source: 'featuresaddreplicate/Abutments/scripts';
    representativeFiles: readonly string[];
    blenderConcepts: readonly string[];
    portRule: string;
}

export interface AbutmentOperationDefinition {
    id: AbutmentOperationId;
    label: string;
    owner: CadCommandOwner;
    toolId: string;
    commandId: CadCommandId;
    jobKind: AbutmentJobKind;
    requiredInputs: readonly string[];
    outputRefs: readonly string[];
    scriptPattern: AbutmentScriptPatternReference;
    performanceRule: string;
}

export interface AbutmentWorkflowStepDefinition {
    id: AbutmentWorkflowStepId;
    label: string;
    userGoal: string;
    operations: readonly AbutmentOperationId[];
    requiredAssets: readonly string[];
    outputAssets: readonly string[];
    acceptanceCriteria: readonly string[];
}

export interface AbutmentWorkflowDefinition {
    id: 'custom-abutment-industrial-v1';
    label: string;
    moduleId: 'tlanticad-abutment';
    steps: readonly AbutmentWorkflowStepDefinition[];
    operations: readonly AbutmentOperationDefinition[];
    migrationNotes: readonly string[];
}

export interface AbutmentReplicaAssetRef {
    id: string;
    role: 'tool-mesh' | 'profile-preset' | 'reference-tooth' | 'vertex-group-template';
    label: string;
    publicPath: `/library/feature-replicas/abutments/${string}`;
    consumedBy: readonly AbutmentOperationId[];
}

export const ABUTMENT_REPLICA_ASSET_REFS = [
    assetRef('generic-solid', 'tool-mesh', 'Generic solid body tool', '/library/feature-replicas/abutments/tools/GenericSolid.stl', ['generate-collar-body', 'cleanup-abutment-mesh']),
    assetRef('tooth-reference', 'reference-tooth', 'Reference tooth OBJ', '/library/feature-replicas/abutments/tools/tooth.obj', ['select-implant-platform', 'create-margin-loop']),
    assetRef('vertex-groups-template', 'vertex-group-template', 'Vertex group template mesh', '/library/feature-replicas/abutments/tools/VertGroups.stl', ['create-margin-loop', 'adapt-to-active-surface']),
    assetRef('profile-rectangle', 'profile-preset', 'Rectangle emergence profile', '/library/feature-replicas/abutments/designs/Rectangle.stl', ['create-cross-section-profile']),
    assetRef('profile-round', 'profile-preset', 'Round emergence profile', '/library/feature-replicas/abutments/designs/Round.stl', ['create-cross-section-profile']),
    assetRef('profile-shoulder', 'profile-preset', 'Shoulder emergence profile', '/library/feature-replicas/abutments/designs/Shoulder.stl', ['create-cross-section-profile']),
    assetRef('profile-clip', 'profile-preset', 'Clip emergence profile', '/library/feature-replicas/abutments/designs/Clip.stl', ['create-cross-section-profile']),
    assetRef('profile-default', 'profile-preset', 'Default emergence profile', '/library/feature-replicas/abutments/designs/default.stl', ['create-cross-section-profile']),
] as const satisfies readonly AbutmentReplicaAssetRef[];

export const ABUTMENT_OPERATION_DEFINITIONS = [
    operation({
        id: 'select-implant-platform',
        label: 'Select implant platform',
        owner: 'tauri-command',
        toolId: 'implant-library',
        commandId: 'implant.library.open',
        jobKind: 'implant-platform-resolve',
        requiredInputs: ['implant library manifest', 'case tooth indication'],
        outputRefs: ['implant platform ref', 'connection geometry ref'],
        representativeFiles: ['NbnbvC454R.py', 'Iu98VVcs34.py'],
        blenderConcepts: ['Implant_Abutment collection', 'library object selection'],
        portRule: 'Resolve local implant manifests through Tauri/Rust; do not load Blender add-on binaries.',
        performanceRule: 'Read library metadata first; preview meshes load lazily by hash.',
    }),
    operation({
        id: 'create-cross-section-profile',
        label: 'Create cross-section profile',
        owner: 'rust-core',
        toolId: 'abutment-cross-section',
        commandId: 'implant.abutment.cross-section.create',
        jobKind: 'abutment-cross-section',
        requiredInputs: ['profile preset', 'axis ref'],
        outputRefs: ['profile curve ref', 'cross-section mesh ref'],
        representativeFiles: ['6H7X3WbcNk.py', '68r86yht67.py', 'zqsqpth675.py', '3GD6f3RUFJ.py'],
        blenderConcepts: ['Cross_Section collection', 'CURVE object', 'bevel_object profile'],
        portRule: 'Recreate profile generation as parametric curve/mesh in Rust; imported STL presets are assets only.',
        performanceRule: 'Generate profile buffers once per preset and reuse them across preview/export.',
    }),
    operation({
        id: 'create-margin-loop',
        label: 'Create margin loop',
        owner: 'rust-core',
        toolId: 'abutment-margin-loop',
        commandId: 'implant.abutment.margin-loop.create',
        jobKind: 'abutment-margin-loop',
        requiredInputs: ['gingiva surface mesh', 'implant axis', 'selected tooth/implant target'],
        outputRefs: ['margin loop curve', 'protected vertex ranges'],
        representativeFiles: ['gty768iukl.py', 'jL8usgAnFJ.py', 'r56wqax09l.py'],
        blenderConcepts: ['Margins collection', 'ActiveSurface', 'vertex groups: Save, Border, Linie, Point'],
        portRule: 'Represent margins as explicit curve assets and topology ranges, not object names in a Blender scene.',
        performanceRule: 'Use BVH nearest-neighbor queries for loop projection; avoid O(n*m) vertex scans.',
    }),
    operation({
        id: 'generate-collar-body',
        label: 'Generate collar / emergence body',
        owner: 'rust-core',
        toolId: 'abutment-collar',
        commandId: 'implant.abutment.collar.generate',
        jobKind: 'abutment-collar-body',
        requiredInputs: ['cross-section profile', 'margin loop curve', 'emergence params'],
        outputRefs: ['collar mesh ref', 'emergence profile ref'],
        representativeFiles: ['5n4n3n6789.py', '465yu89olp.py', 'cvf45aw21q.py', 'dkie64gdne.py'],
        blenderConcepts: ['Collar collection', 'Free_Formed object', 'loop/top/outer vertex groups'],
        portRule: 'Generate collar as a derived mesh with explicit params and provenance.',
        performanceRule: 'Keep interactive controls in Three preview; commit one backend job per accepted edit batch.',
    }),
    operation({
        id: 'adapt-to-active-surface',
        label: 'Adapt to active surface',
        owner: 'rust-core',
        toolId: 'abutment-shrinkwrap',
        commandId: 'implant.abutment.surface-adapt',
        jobKind: 'abutment-shrinkwrap',
        requiredInputs: ['collar mesh ref', 'gingiva/prep surface mesh', 'stick/free control points'],
        outputRefs: ['adapted abutment mesh ref', 'distance/intersection map'],
        representativeFiles: ['XXce5ehGs4.py', 'MPczt6yPCu.py', 'Y5Cgn8Znbj.py'],
        blenderConcepts: ['SHRINKWRAP modifier', 'VERTEX_WEIGHT_PROXIMITY', 'stick/free control groups'],
        portRule: 'Port shrinkwrap as projection job with persisted control-point states from the UI.',
        performanceRule: 'Build one acceleration structure per source mesh and cache by mesh hash.',
    }),
    operation({
        id: 'boolean-cut-screw-channel',
        label: 'Boolean cut screw channel',
        owner: 'rust-core',
        toolId: 'abutment-screw-channel',
        commandId: 'implant.abutment.screw-channel.cut',
        jobKind: 'abutment-boolean-cut',
        requiredInputs: ['adapted abutment mesh', 'screw channel axis', 'tool diameter'],
        outputRefs: ['screw-channel mesh ref', 'angle validation report'],
        representativeFiles: ['456thy90pl.py', '3er8io56mv.py', 'su4v56ak5m.py', 'ui9o03edfv.py'],
        blenderConcepts: ['BOOLEAN modifier', 'CuttingTool', 'INTERSECT/DIFFERENCE/UNION'],
        portRule: 'Run booleans through Rust manifold/CSG jobs; UI only previews tool placement.',
        performanceRule: 'Use cancellable jobs and emit progress; never boolean on the render thread.',
    }),
    operation({
        id: 'cleanup-abutment-mesh',
        label: 'Cleanup abutment mesh',
        owner: 'rust-core',
        toolId: 'abutment-cleanup',
        commandId: 'implant.abutment.mesh-cleanup',
        jobKind: 'abutment-mesh-cleanup',
        requiredInputs: ['generated abutment mesh'],
        outputRefs: ['manufacturing-ready mesh', 'mesh QA report'],
        representativeFiles: ['Ghg565TrfR0.py', 'KSqoiTdb5G.py', 'fy67jefr56.py'],
        blenderConcepts: ['REMESH', 'SMOOTH', 'WELD', 'DECIMATE', 'delete largest face/manifold cleanup'],
        portRule: 'Replace Blender modifiers with deterministic mesh repair/QA services.',
        performanceRule: 'Batch cleanup operations into one mesh job to reduce read/write churn.',
    }),
    operation({
        id: 'export-abutment-report',
        label: 'Export abutment package',
        owner: 'python-sidecar',
        toolId: 'abutment-report',
        commandId: 'implant.abutment.export-package',
        jobKind: 'abutment-export-package',
        requiredInputs: ['validated abutment mesh', 'implant metadata', 'case manifest'],
        outputRefs: ['abutment STL', 'construction info JSON', 'planning PDF'],
        representativeFiles: ['G67YhFgerm.py', 'hewyfbr65h.py', 'hted35svfy.py', 'mMAjw78rm5.py'],
        blenderConcepts: ['reportlab PDF', 'STL/XML export', 'versioned .blend save'],
        portRule: 'Export portable case artifacts, not Blender files; reports must include params and source hashes.',
        performanceRule: 'Write reports off the UI thread and stream large STL output through the asset vault.',
    }),
] as const satisfies readonly AbutmentOperationDefinition[];

export const ABUTMENT_WORKFLOW_DEFINITION: AbutmentWorkflowDefinition = {
    id: 'custom-abutment-industrial-v1',
    label: 'Custom Abutment: platform -> emergence -> adapt -> screw channel -> export',
    moduleId: 'tlanticad-abutment',
    steps: [
        step('platform-context', 'Platform and case context', 'Confirm implant platform, gingiva/prep scan, target tooth and axis before mesh generation.', ['select-implant-platform'], ['implant-plan', 'gingiva-scan'], ['implant platform ref'], ['Local implant library is resolved', 'No file picker appears after opening an existing case']),
        step('profile-margin', 'Profile and margin loop', 'Create the cross-section preset and explicit margin loop used by the abutment body.', ['create-cross-section-profile', 'create-margin-loop'], ['gingiva-scan', 'implant platform ref'], ['profile curve ref', 'margin loop curve'], ['Profile is parametric or asset-backed', 'Margin loop is persisted as a curve asset']),
        step('collar-emergence', 'Collar and emergence body', 'Generate the collar/body from margin, profile and emergence parameters.', ['generate-collar-body'], ['profile curve ref', 'margin loop curve'], ['collar mesh ref'], ['Body has provenance params', 'Preview commit creates a derived mesh handle']),
        step('surface-adaptation', 'Surface adaptation', 'Project free/stuck control regions onto the active surface and produce distance maps.', ['adapt-to-active-surface'], ['collar mesh ref', 'gingiva surface mesh'], ['adapted mesh ref', 'distance map'], ['Projection uses cached acceleration structure', 'Intersection warnings are reviewable']),
        step('screw-channel', 'Screw channel and boolean cut', 'Place straight or angulated screw channel and validate material/tool clearance.', ['boolean-cut-screw-channel'], ['adapted mesh ref'], ['screw-channel mesh ref', 'angle validation report'], ['Angle limits are enforced', 'Boolean runs as cancellable Rust job']),
        step('validate-export', 'Validate and export', 'Clean mesh, validate thickness and write STL/construction/report artifacts.', ['cleanup-abutment-mesh', 'export-abutment-report'], ['screw-channel mesh ref'], ['abutment STL', 'construction info JSON', 'planning PDF'], ['Mesh QA passes', 'Export includes hashes and implant metadata']),
    ],
    operations: ABUTMENT_OPERATION_DEFINITIONS,
    migrationNotes: [
        'Blender scripts are reference-only: do not execute copied .py, .so or .pyd files in TlantiCAD runtime.',
        'Object-name conventions such as Cross_Section, CURVE, Collar and ActiveSurface become typed scene objects and asset refs.',
        'Modifiers such as Shrinkwrap, Boolean, Remesh and Solidify become Rust/Python jobs with progress, cancellation and artifacts.',
    ],
};

export function resolveAbutmentOperation(id: string): AbutmentOperationDefinition | null {
    return ABUTMENT_OPERATION_DEFINITIONS.find((operationDef) => operationDef.id === id) ?? null;
}

export function resolveAbutmentWorkflowStep(id: string): AbutmentWorkflowStepDefinition | null {
    return ABUTMENT_WORKFLOW_DEFINITION.steps.find((stepDef) => stepDef.id === id) ?? null;
}

export function listAbutmentOperationsForStep(stepId: string): readonly AbutmentOperationDefinition[] {
    const workflowStep = resolveAbutmentWorkflowStep(stepId);
    if (!workflowStep) return [];
    return workflowStep.operations
        .map((operationId) => resolveAbutmentOperation(operationId))
        .filter(Boolean) as AbutmentOperationDefinition[];
}

export function listAbutmentReplicaAssetsForOperation(operationId: AbutmentOperationId): readonly AbutmentReplicaAssetRef[] {
    return ABUTMENT_REPLICA_ASSET_REFS.filter((asset) => asset.consumedBy.includes(operationId));
}

export function createAbutmentOperationCommand(
    operationId: AbutmentOperationId,
    payload: CadCommandPayload = {},
): CadCommandRunResult {
    const operationDef = resolveAbutmentOperation(operationId);
    if (!operationDef) {
        return {
            accepted: false,
            command: {
                id: `implant.abutment.${operationId}` as CadCommandId,
                label: operationId,
                owner: 'react-ui',
                effect: 'ui-state',
                payload,
                createdAt: new Date(0).toISOString(),
            },
            issues: [`Unknown abutment operation ${operationId}`],
        };
    }

    return {
        accepted: true,
        command: {
            id: operationDef.commandId,
            label: operationDef.label,
            owner: operationDef.owner,
            effect: operationDef.owner === 'three-render' ? 'scene-preview' : 'job-start',
            payload: { ...payload, toolId: operationDef.toolId },
            createdAt: new Date(0).toISOString(),
        },
        queuedJobKind: operationDef.jobKind,
        issues: [],
    };
}

export function validateAbutmentWorkflowDefinition(): readonly string[] {
    const issues: string[] = [];
    const operationIds = new Set(ABUTMENT_OPERATION_DEFINITIONS.map((operationDef) => operationDef.id));
    const toolIds = new Set(ABUTMENT_OPERATION_DEFINITIONS.map((operationDef) => operationDef.toolId));

    if (ABUTMENT_WORKFLOW_DEFINITION.steps.length < 6) {
        issues.push('Abutment workflow must cover platform, profile, collar, adaptation, screw channel and export.');
    }

    for (const stepDef of ABUTMENT_WORKFLOW_DEFINITION.steps) {
        for (const operationId of stepDef.operations) {
            if (!operationIds.has(operationId)) {
                issues.push(`${stepDef.id} references missing operation ${operationId}`);
            }
        }
    }

    for (const operationDef of ABUTMENT_OPERATION_DEFINITIONS) {
        if (!operationDef.scriptPattern.representativeFiles.length) {
            issues.push(`${operationDef.id} needs at least one reference script mapping.`);
        }
        if (operationDef.scriptPattern.portRule.toLowerCase().includes('execute blender')) {
            issues.push(`${operationDef.id} must not execute Blender scripts inside TlantiCAD.`);
        }
        if (!toolIds.has(operationDef.toolId)) {
            issues.push(`${operationDef.id} missing tool id.`);
        }
    }

    return issues;
}

function operation(input: Omit<AbutmentOperationDefinition, 'scriptPattern'> & {
    representativeFiles: readonly string[];
    blenderConcepts: readonly string[];
    portRule: string;
}): AbutmentOperationDefinition {
    const { representativeFiles, blenderConcepts, portRule, ...rest } = input;
    return {
        ...rest,
        scriptPattern: {
            source: 'featuresaddreplicate/Abutments/scripts',
            representativeFiles,
            blenderConcepts,
            portRule,
        },
    };
}

function assetRef(
    id: string,
    role: AbutmentReplicaAssetRef['role'],
    label: string,
    publicPath: AbutmentReplicaAssetRef['publicPath'],
    consumedBy: readonly AbutmentOperationId[],
): AbutmentReplicaAssetRef {
    return { id, role, label, publicPath, consumedBy };
}

function step(
    id: AbutmentWorkflowStepId,
    label: string,
    userGoal: string,
    operations: readonly AbutmentOperationId[],
    requiredAssets: readonly string[],
    outputAssets: readonly string[],
    acceptanceCriteria: readonly string[],
): AbutmentWorkflowStepDefinition {
    return { id, label, userGoal, operations, requiredAssets, outputAssets, acceptanceCriteria };
}
