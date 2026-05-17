import type { CadContextMenuItem, CadContextMenuProvider, CadContextSource } from './context-menu-registry'
import { item } from './context-menu-registry'

const viewportCommon = [
  item({ id: 'viewport.reset-view', label: 'Reset view', commandId: 'cad.view.reset', toolId: 'select' }),
  item({ id: 'viewport.save-view', label: 'Save view', commandId: 'cad.view.save', toolId: 'command-palette' }),
  item({ id: 'viewport.toggle-grid', label: 'Toggle grid', commandId: 'cad.view.toggle-grid', toolId: 'layers-panel' }),
  item({ id: 'viewport.cut-view', label: 'Toggle cut view', commandId: 'cad.tool.clip', toolId: 'clip', separator: true }),
] as const satisfies readonly CadContextMenuItem[]

export const cadContextMenuProvider: CadContextMenuProvider = {
  resolve(source: CadContextSource): readonly CadContextMenuItem[] {
    if (source.kind === 'viewport') {
      return [
        ...viewportCommon,
        item({ id: 'viewport.command-palette', label: 'Command palette', commandId: 'cad.tool.command-palette', toolId: 'command-palette' }),
      ]
    }

    if (source.kind === 'object') {
      return [
        item({ id: 'object.focus', label: 'Focus object', commandId: 'cad.selection.focus', toolId: 'select' }),
        item({ id: 'object.hide', label: 'Hide / show', commandId: 'cad.selection.toggle-visibility', toolId: 'layers-panel' }),
        item({ id: 'object.wireframe', label: 'Toggle wireframe', commandId: 'cad.view.toggle-wireframe', toolId: 'layers-panel' }),
        item({ id: 'object.opacity', label: 'Toggle opacity', commandId: 'cad.view.toggle-opacity', toolId: 'layers-panel' }),
        item({ id: 'object.duplicate', label: 'Duplicate', commandId: 'cad.selection.duplicate', toolId: 'copy-mirror', separator: true }),
        item({ id: 'object.mirror', label: 'Mirror', commandId: 'dental.copy-mirror', toolId: 'copy-mirror' }),
        item({ id: 'object.measure', label: 'Measure from object', commandId: 'cad.tool.measure', toolId: 'measure' }),
        item({ id: 'object.export', label: 'Export selected', commandId: 'manufacturing.export', toolId: 'manufacturing-export', separator: true }),
        item({ id: 'object.delete', label: 'Delete', commandId: 'cad.selection.delete', toolId: 'select', tone: 'danger' }),
      ]
    }

    return []
  },
}

export const dicomContextMenuProvider: CadContextMenuProvider = {
  resolve(source: CadContextSource): readonly CadContextMenuItem[] {
    const hasSeries = source.kind === 'series' ? Boolean(source.seriesId) : source.kind === 'viewport'
    return [
      item({ id: 'dicom.window-level', label: 'Window / level preset', commandId: 'dicom.window-level', toolId: 'dicom-mpr' }),
      item({ id: 'dicom.mpr-layout', label: 'MPR layout', commandId: 'dicom.mpr.layout', toolId: 'dicom-mpr' }),
      item({ id: 'dicom.measure', label: 'Add measurement', commandId: 'cad.tool.measure', toolId: 'measure' }),
      item({
        id: 'dicom.segment',
        label: 'Segment structure',
        commandId: 'dicom.segment.run',
        toolId: 'segment',
        disabledReason: hasSeries ? null : 'Load a DICOM series first',
        separator: true,
      }),
      item({ id: 'dicom.send-mask-cad', label: 'Send mask to CAD', commandId: 'dicom.mask.send-to-cad', toolId: 'segment', disabledReason: hasSeries ? null : 'No active mask/series' }),
      item({ id: 'dicom.sanitize', label: 'Sanitize metadata', commandId: 'dicom.sanitize', toolId: 'dicom-metadata' }),
    ]
  },
}

export const implantContextMenuProvider: CadContextMenuProvider = {
  resolve(source: CadContextSource): readonly CadContextMenuItem[] {
    if (source.kind === 'object' && source.objectKind === 'abutment') {
      return [
        item({ id: 'abutment.profile', label: 'Create cross-section profile', commandId: 'implant.abutment.cross-section.create', toolId: 'abutment-cross-section' }),
        item({ id: 'abutment.margin-loop', label: 'Create margin loop', commandId: 'implant.abutment.margin-loop.create', toolId: 'abutment-margin-loop' }),
        item({ id: 'abutment.collar', label: 'Generate collar / emergence body', commandId: 'implant.abutment.collar.generate', toolId: 'abutment-collar' }),
        item({ id: 'abutment.adapt', label: 'Adapt to active surface', commandId: 'implant.abutment.surface-adapt', toolId: 'abutment-shrinkwrap', separator: true }),
        item({ id: 'abutment.screw-channel', label: 'Cut screw channel', commandId: 'implant.abutment.screw-channel.cut', toolId: 'abutment-screw-channel' }),
        item({ id: 'abutment.cleanup', label: 'Cleanup mesh', commandId: 'implant.abutment.mesh-cleanup', toolId: 'abutment-cleanup' }),
        item({ id: 'abutment.export', label: 'Export abutment package', commandId: 'implant.abutment.export-package', toolId: 'abutment-report', separator: true }),
      ]
    }

    if (source.kind === 'object' && source.objectKind === 'sleeve') {
      return [
        item({ id: 'guide.sleeve-edit', label: 'Edit sleeve', commandId: 'guide.sleeve.edit', toolId: 'sleeve-controls' }),
        item({ id: 'guide.validate-drill', label: 'Validate drill protocol', commandId: 'guide.drill.validate', toolId: 'guide-wizard' }),
        item({ id: 'guide.export', label: 'Export guide', commandId: 'manufacturing.export', toolId: 'guide-export' }),
      ]
    }

    return [
      item({ id: 'implant.change-system', label: 'Change implant system', commandId: 'implant.library.open', toolId: 'implant-library' }),
      item({ id: 'implant.scan-body', label: 'Match scan body', commandId: 'implant.scan-body.match', toolId: 'scan-body' }),
      item({ id: 'implant.trajectory', label: 'Set trajectory', commandId: 'implant.trajectory.set', toolId: 'implant-axis' }),
      item({ id: 'implant.nerve-sinus', label: 'Toggle nerve / sinus', commandId: 'implant.anatomy.toggle', toolId: 'nerve-mark' }),
      item({ id: 'implant.add-sleeve', label: 'Add sleeve', commandId: 'guide.sleeve.add', toolId: 'sleeve-controls', separator: true }),
    ]
  },
}

export const splintContextMenuProvider: CadContextMenuProvider = {
  resolve(): readonly CadContextMenuItem[] {
    return [
      item({ id: 'splint.occlusion-map', label: 'Occlusion map', commandId: 'splint.occlusion.map', toolId: 'contacts' }),
      item({ id: 'splint.offset', label: 'Set offset', commandId: 'mesh.offset', toolId: 'offset' }),
      item({ id: 'splint.trim', label: 'Trim splint', commandId: 'mesh.trim', toolId: 'trim' }),
      item({ id: 'splint.export', label: 'Export splint', commandId: 'manufacturing.export', toolId: 'splint-export' }),
    ]
  },
}

export const cephContextMenuProvider: CadContextMenuProvider = {
  resolve(source: CadContextSource): readonly CadContextMenuItem[] {
    return [
      item({ id: 'ceph.landmark-add', label: source.kind === 'object' ? 'Edit landmark' : 'Add landmark', commandId: 'ceph.landmark.edit', toolId: 'ceph-landmarks' }),
      item({ id: 'ceph.measurement', label: 'Add measurement', commandId: 'ceph.measurement.add', toolId: 'measure' }),
      item({ id: 'ceph.report', label: 'Generate report', commandId: 'manufacturing.export', toolId: 'manufacturing-export' }),
    ]
  },
}

export const alignerContextMenuProvider: CadContextMenuProvider = {
  resolve(): readonly CadContextMenuItem[] {
    return [
      item({ id: 'aligner.stage', label: 'Edit stage', commandId: 'ortho.stage.edit', toolId: 'ortho-setup' }),
      item({ id: 'aligner.attachment', label: 'Add attachment', commandId: 'ortho.attachment.add', toolId: 'aligner-attachments' }),
      item({ id: 'aligner.ipr', label: 'Set IPR', commandId: 'ortho.ipr.set', toolId: 'aligner-attachments' }),
      item({ id: 'aligner.collision', label: 'Collision check', commandId: 'ortho.collision.check', toolId: 'aligner-attachments' }),
    ]
  },
}

export function createDefaultCadContextMenuProviders(): ReadonlyMap<string, CadContextMenuProvider> {
  return new Map<string, CadContextMenuProvider>([
    ['cad', cadContextMenuProvider],
    ['model-creator', cadContextMenuProvider],
    ['partials', cadContextMenuProvider],
    ['fab', cadContextMenuProvider],
    ['dicom', dicomContextMenuProvider],
    ['implant', implantContextMenuProvider],
    ['guide', implantContextMenuProvider],
    ['splint', splintContextMenuProvider],
    ['ceph', cephContextMenuProvider],
    ['aligners', alignerContextMenuProvider],
    ['orthocad', alignerContextMenuProvider],
  ])
}
