import React, { useEffect, useMemo, useState } from 'react';
import { motion } from 'framer-motion';
import { Box, CheckCircle2, Download, Factory, Layers, Printer, X } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import type { TlantiCase } from '@/stores/tlantidb-case-store';
import { useViewportProfile } from '../hooks/useViewportProfile';
import type { ThemeMode } from '../types';
import { useToast } from './ui/use-toast';
import { commandRegistry } from '@/features/command-palette';

interface FabModuleProps {
  themeMode: ThemeMode;
  onClose: () => void;
  activeCase?: TlantiCase | null;
}

type Format = 'stl' | '3mf' | 'obj' | 'ply';
type MachineProfile = 'mill-5axis' | 'mill-4axis' | 'printer-dlp' | 'printer-fdm' | 'printer-sla';

const FORMATS: Array<{ value: Format; label: string; description: string }> = [
  { value: 'stl', label: 'STL', description: 'Estándar para impresión y manufactura.' },
  { value: '3mf', label: '3MF', description: 'Formato moderno con metadata por restauración.' },
  { value: 'obj', label: 'OBJ', description: 'Compatibilidad con software DCC.' },
  { value: 'ply', label: 'PLY', description: 'Mantiene color por vértice (Truesmile).' },
];

const MACHINES: Array<{ value: MachineProfile; label: string; tolerance: number }> = [
  { value: 'mill-5axis', label: 'Milling · 5 ejes', tolerance: 0.02 },
  { value: 'mill-4axis', label: 'Milling · 4 ejes', tolerance: 0.04 },
  { value: 'printer-dlp', label: 'Print · DLP', tolerance: 0.05 },
  { value: 'printer-sla', label: 'Print · SLA', tolerance: 0.025 },
  { value: 'printer-fdm', label: 'Print · FDM', tolerance: 0.18 },
];

export function FabModule({ themeMode, onClose, activeCase }: FabModuleProps) {
  const { toast } = useToast();
  const viewport = useViewportProfile();
  const [format, setFormat] = useState<Format>('stl');
  const [machine, setMachine] = useState<MachineProfile>('mill-5axis');
  const [validateMesh, setValidateMesh] = useState(true);
  const [orientForBuild, setOrientForBuild] = useState(true);
  const [packageJob, setPackageJob] = useState(true);
  const [margin, setMargin] = useState(0.05);

  const machineDef = useMemo(() => MACHINES.find((m) => m.value === machine)!, [machine]);
  const formatDef = useMemo(() => FORMATS.find((f) => f.value === format)!, [format]);

  const restorationCount = useMemo(() => {
    if (!activeCase?.toothMap) return 0;
    return Object.values(activeCase.toothMap).filter((tooth) => tooth?.workTypeId).length;
  }, [activeCase]);

  const exportPackage = () => {
    if (!activeCase) {
      toast('Abre un caso activo antes de exportar el paquete CAM.', 'error');
      return;
    }
    if (restorationCount === 0) {
      toast('No hay restauraciones definidas. Ve al workspace y configura los dientes.', 'error');
      return;
    }
    const filename = `${activeCase.caseNumber}-fab-${machine}.${format}`;
    const manifest = {
      kind: 'fab-job-package',
      createdAt: new Date().toISOString(),
      caseId: activeCase.id,
      caseNumber: activeCase.caseNumber,
      format,
      machine,
      tolerance: machineDef.tolerance,
      margin,
      pipeline: { validateMesh, orientForBuild, packageJob },
      restorations: restorationCount,
    };
    const blob = new Blob([JSON.stringify(manifest, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${filename}.manifest.json`;
    a.click();
    URL.revokeObjectURL(url);
    toast(`Paquete CAM "${filename}" generado.`, 'success');
  };

  useEffect(() => {
    return commandRegistry.registerAll([
      {
        id: 'cad.module.fab.close',
        label: 'Cerrar módulo Fabricación',
        kind: 'navigation',
        keywords: ['fab', 'fabricacion', 'close', 'cerrar'],
        run: onClose,
      },
      {
        id: 'cad.module.fab.export',
        label: 'Fabricación: exportar paquete CAM',
        kind: 'tool',
        keywords: ['fab', 'cam', 'export', 'package', 'paquete'],
        available: () => restorationCount > 0,
        run: exportPackage,
      },
    ]);
  }, [onClose, restorationCount, exportPackage]);

  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className={cn(
        'absolute z-40 flex flex-col overflow-hidden rounded-lg border shadow-2xl',
        viewport.isCompact ? 'inset-3 top-[8rem]' : 'left-16 top-24 bottom-24 w-80',
        'bg-surface border-border text-text-primary',
        themeMode === 'light' && 'bg-surface',
      )}
    >
      <div className="flex items-center justify-between border-b border-border bg-surface-raised p-4">
        <div className="flex items-center gap-2">
          <Factory size={18} className="text-text-display" />
          <h3 className="font-display font-semibold tracking-tight text-text-primary">Fabricación</h3>
        </div>
        <button onClick={onClose} aria-label="Close fabrication module" className="text-text-secondary transition-colors hover:text-text-primary">
          <X size={16} aria-hidden />
        </button>
      </div>

      <div className="flex flex-1 flex-col gap-4 overflow-y-auto bg-surface p-4">
        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="flex items-start justify-between gap-3">
            <div>
              <p className="text-[11px] uppercase text-text-secondary">Caso</p>
              <h4 className="mt-1 text-base font-semibold text-text-display">
                {activeCase ? `${activeCase.caseNumber} · ${activeCase.name}` : 'Sin caso activo'}
              </h4>
              <p className="mt-1 text-xs text-text-secondary">
                {restorationCount} restauraciones listas · tolerancia {machineDef.tolerance.toFixed(2)} mm
              </p>
            </div>
            <Badge variant="outline" className="border-border bg-card text-text-primary">
              {restorationCount}
            </Badge>
          </div>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <label htmlFor="fab-format" className="text-[11px] uppercase text-text-secondary">Formato</label>
          <select
            id="fab-format"
            value={format}
            onChange={(event) => setFormat(event.target.value as Format)}
            className="mt-2 w-full rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary"
          >
            {FORMATS.map((option) => (
              <option key={option.value} value={option.value}>{option.label}</option>
            ))}
          </select>
          <p className="mt-2 text-xs text-text-secondary">{formatDef.description}</p>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <label htmlFor="fab-machine" className="text-[11px] uppercase text-text-secondary">Perfil de máquina</label>
          <select
            id="fab-machine"
            value={machine}
            onChange={(event) => setMachine(event.target.value as MachineProfile)}
            className="mt-2 w-full rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary"
          >
            {MACHINES.map((option) => (
              <option key={option.value} value={option.value}>{option.label}</option>
            ))}
          </select>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="flex flex-col gap-2">
            <div className="flex justify-between text-[11px] uppercase text-text-primary">
              <span>Margen extra</span>
              <span>{margin.toFixed(2)} mm</span>
            </div>
            <input
              type="range"
              min="0.0"
              max="0.30"
              step="0.01"
              value={margin}
              onChange={(event) => setMargin(parseFloat(event.target.value))}
              className="w-full accent-text-display"
              aria-label="Margen extra de manufactura"
            />
          </div>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="mb-3 flex items-center gap-2">
            <Layers size={14} className="text-text-display" />
            <h4 className="text-[11px] uppercase text-text-secondary">Pipeline</h4>
          </div>
          <div className="space-y-2 text-sm">
            <label className="flex items-center justify-between gap-2">
              <span className="flex items-center gap-2 text-text-primary"><CheckCircle2 size={14} className={validateMesh ? 'text-green-400' : 'text-text-secondary'} /> Validar malla</span>
              <input type="checkbox" checked={validateMesh} onChange={(event) => setValidateMesh(event.target.checked)} className="accent-text-display" />
            </label>
            <label className="flex items-center justify-between gap-2">
              <span className="flex items-center gap-2 text-text-primary"><Box size={14} className={orientForBuild ? 'text-green-400' : 'text-text-secondary'} /> Orientar para impresión</span>
              <input type="checkbox" checked={orientForBuild} onChange={(event) => setOrientForBuild(event.target.checked)} className="accent-text-display" />
            </label>
            <label className="flex items-center justify-between gap-2">
              <span className="flex items-center gap-2 text-text-primary"><Printer size={14} className={packageJob ? 'text-green-400' : 'text-text-secondary'} /> Empaquetar trabajo CAM</span>
              <input type="checkbox" checked={packageJob} onChange={(event) => setPackageJob(event.target.checked)} className="accent-text-display" />
            </label>
          </div>
        </section>

        <button
          type="button"
          onClick={exportPackage}
          disabled={restorationCount === 0}
          className="mt-auto flex w-full items-center justify-center gap-2 rounded bg-text-display py-3 text-xs font-bold uppercase tracking-widest text-black transition-colors hover:bg-white disabled:cursor-not-allowed disabled:opacity-50"
        >
          <Download size={16} /> Exportar paquete CAM
        </button>
      </div>
    </motion.div>
  );
}
