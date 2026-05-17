import React, { useEffect, useMemo, useState } from 'react';
import { motion } from 'framer-motion';
import { Activity, Crosshair, Download, Ruler, Target, X } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import type { TlantiCase } from '@/stores/tlantidb-case-store';
import { useViewportProfile } from '../hooks/useViewportProfile';
import type { ThemeMode } from '../types';
import { useToast } from './ui/use-toast';
import { commandRegistry } from '@/features/command-palette';

interface CephModuleProps {
  themeMode: ThemeMode;
  onClose: () => void;
  activeCase?: TlantiCase | null;
}

type Analysis = 'steiner' | 'ricketts' | 'mcnamara' | 'tweed';

const ANALYSES: Array<{ value: Analysis; label: string; landmarks: number; description: string }> = [
  { value: 'steiner', label: 'Steiner', landmarks: 13, description: 'Análisis clásico con SNA, SNB, ANB y plano oclusal.' },
  { value: 'ricketts', label: 'Ricketts', landmarks: 16, description: 'Foco en eje facial, profundidad mandibular y tendencia de crecimiento.' },
  { value: 'mcnamara', label: 'McNamara', landmarks: 11, description: 'Análisis combinado dental y esquelético con plano de Frankfort.' },
  { value: 'tweed', label: 'Tweed', landmarks: 9, description: 'Triángulo Tweed para diagnóstico de incisivos inferiores.' },
];

const LANDMARK_GROUPS = [
  { name: 'Esqueléticos', items: ['S (Silla)', 'N (Nasion)', 'A (Punto A)', 'B (Punto B)', 'Pog (Pogonion)', 'Go (Gonion)', 'Me (Mentón)'] },
  { name: 'Dentales', items: ['Is (Incisivo superior)', 'Ii (Incisivo inferior)', 'Ms (Molar superior)', 'Mi (Molar inferior)'] },
  { name: 'Tejidos blandos', items: ['Pn (Punta nasal)', 'Sn (Subnasal)', "Ls (Labio sup.)", 'Li (Labio inf.)'] },
];

export function CephModule({ themeMode, onClose, activeCase }: CephModuleProps) {
  const { toast } = useToast();
  const viewport = useViewportProfile();
  const [analysis, setAnalysis] = useState<Analysis>('steiner');
  const [calibrationMm, setCalibrationMm] = useState(10);
  const [placedLandmarks, setPlacedLandmarks] = useState<Record<string, boolean>>({});

  const analysisDef = useMemo(() => ANALYSES.find((a) => a.value === analysis)!, [analysis]);
  const totalLandmarks = useMemo(() => LANDMARK_GROUPS.reduce((sum, group) => sum + group.items.length, 0), []);
  const placedCount = Object.values(placedLandmarks).filter(Boolean).length;
  const isReady = placedCount >= analysisDef.landmarks;

  const placeLandmark = (label: string) => {
    setPlacedLandmarks((current) => ({ ...current, [label]: !current[label] }));
  };

  const exportAnalysis = () => {
    if (!activeCase) {
      toast('Abre un caso activo antes de exportar el análisis cefalométrico.', 'error');
      return;
    }
    if (!isReady) {
      toast(`Coloca al menos ${analysisDef.landmarks} landmarks para el análisis ${analysisDef.label}.`, 'error');
      return;
    }
    const filename = `${activeCase.caseNumber}-ceph-${analysis}.json`;
    const payload = {
      kind: 'cephalometric-analysis',
      createdAt: new Date().toISOString(),
      caseId: activeCase.id,
      caseNumber: activeCase.caseNumber,
      analysis,
      calibrationMm,
      landmarks: placedLandmarks,
    };
    const blob = new Blob([JSON.stringify(payload, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    a.click();
    URL.revokeObjectURL(url);
    toast('Análisis cefalométrico exportado.', 'success');
  };

  useEffect(() => {
    return commandRegistry.registerAll([
      {
        id: 'cad.module.ceph.close',
        label: 'Cerrar módulo Cefalometría',
        kind: 'navigation',
        keywords: ['ceph', 'cefalometria', 'close', 'cerrar'],
        run: onClose,
      },
      {
        id: 'cad.module.ceph.export',
        label: 'Cefalometría: exportar análisis',
        kind: 'tool',
        keywords: ['ceph', 'export', 'analysis', 'analisis'],
        available: () => isReady,
        run: exportAnalysis,
      },
    ]);
  }, [onClose, isReady, exportAnalysis]);

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
          <Activity size={18} className="text-text-display" />
          <h3 className="font-display font-semibold tracking-tight text-text-primary">Cefalometría</h3>
        </div>
        <button onClick={onClose} aria-label="Close cephalometric analysis" className="text-text-secondary transition-colors hover:text-text-primary">
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
            </div>
            <Badge variant="outline" className="border-border bg-card text-text-primary">
              {placedCount}/{totalLandmarks}
            </Badge>
          </div>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <label htmlFor="ceph-analysis" className="text-[11px] uppercase text-text-secondary">Análisis</label>
          <select
            id="ceph-analysis"
            value={analysis}
            onChange={(event) => setAnalysis(event.target.value as Analysis)}
            className="mt-2 w-full rounded border border-border-visible bg-card p-2 text-sm text-text-primary outline-none focus:border-text-primary"
          >
            {ANALYSES.map((option) => (
              <option key={option.value} value={option.value}>
                {option.label} · {option.landmarks} landmarks
              </option>
            ))}
          </select>
          <p className="mt-2 text-sm text-text-secondary">{analysisDef.description}</p>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="mb-3 flex items-center gap-2">
            <Ruler size={14} className="text-text-display" />
            <h4 className="text-[11px] uppercase text-text-secondary">Calibración</h4>
          </div>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min="5"
              max="20"
              step="1"
              value={calibrationMm}
              onChange={(event) => setCalibrationMm(parseInt(event.target.value, 10))}
              className="flex-1 accent-text-display"
              aria-label="Distancia de calibración en milímetros"
            />
            <span className="font-mono text-sm text-text-primary">{calibrationMm} mm</span>
          </div>
          <p className="mt-2 text-xs text-text-secondary">Marca dos puntos en la regla del Rx para calibrar la escala.</p>
        </section>

        <section className="rounded-2xl border border-border bg-surface-raised p-4">
          <div className="mb-3 flex items-center gap-2">
            <Target size={14} className="text-text-display" />
            <h4 className="text-[11px] uppercase text-text-secondary">Landmarks</h4>
          </div>
          <div className="space-y-3">
            {LANDMARK_GROUPS.map((group) => (
              <div key={group.name}>
                <p className="mb-1 text-[10px] uppercase tracking-wider text-text-secondary">{group.name}</p>
                <div className="flex flex-wrap gap-1.5">
                  {group.items.map((item) => {
                    const placed = !!placedLandmarks[item];
                    return (
                      <button
                        key={item}
                        type="button"
                        onClick={() => placeLandmark(item)}
                        className={cn(
                          'inline-flex items-center gap-1 rounded border px-2 py-1 text-[11px] transition',
                          placed
                            ? 'border-emerald-400/60 bg-emerald-500/15 text-emerald-200'
                            : 'border-border bg-card text-text-secondary hover:bg-surface-raised',
                        )}
                      >
                        <Crosshair size={10} aria-hidden /> {item}
                      </button>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        </section>

        <button
          type="button"
          onClick={exportAnalysis}
          disabled={!isReady}
          className="mt-auto flex w-full items-center justify-center gap-2 rounded bg-text-display py-3 text-xs font-bold uppercase tracking-widest text-black transition-colors hover:bg-white disabled:cursor-not-allowed disabled:opacity-50"
        >
          <Download size={16} /> Exportar análisis
        </button>
      </div>
    </motion.div>
  );
}
