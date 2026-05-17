import React from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X, Command, MousePointer2 } from 'lucide-react';
import clsx from 'clsx';
import { ThemeMode } from '../types';

interface ShortcutsPanelProps {
  isOpen: boolean;
  onClose: () => void;
  themeMode: ThemeMode;
  controlScheme: 'cad' | 'standard';
  onControlSchemeChange: (value: 'cad' | 'standard') => void;
}

export const ShortcutsPanel: React.FC<ShortcutsPanelProps> = ({ isOpen, onClose, themeMode, controlScheme, onControlSchemeChange }) => {
  const panelClass = "bg-surface text-text-primary border-border shadow-2xl";
  const headerClass = "bg-surface-raised border-b border-border";
  const keyClass = "inline-flex items-center justify-center min-w-[24px] px-1.5 h-6 rounded border border-border-visible bg-surface-raised text-[10px] font-bold font-mono uppercase mx-0.5 text-text-primary";
  
  const groups = [
    { key: 'A', label: 'Antagonist' },
    { key: 'S', label: 'Scan Body' },
    { key: 'G', label: 'Gingiva' },
    { key: 'E', label: 'Enamel/Physio' },
    { key: 'C', label: 'Connectors' },
    { key: 'W', label: 'Waxup' },
    { key: 'P', label: 'Pre-op' },
    { key: 'V', label: 'Virtual Gingiva' },
    { key: 'T', label: 'Telescopes' },
    { key: 'O', label: 'Other' },
    { key: 'X', label: 'Upper Jaw (Maxilla)' },
    { key: 'N', label: 'Lower Jaw (Mandible)' },
    { key: 'D', label: 'DICOM' },
    { key: 'M', label: 'Merged Parts' }
  ];

  return (
    <AnimatePresence>
      {isOpen && (
        <div className="fixed inset-0 z-[100] flex items-center justify-center pointer-events-none">
          <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" onClick={onClose} />
          
          <motion.div 
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            className={clsx("relative w-[900px] max-w-[95vw] h-[600px] rounded-lg border pointer-events-auto flex flex-col overflow-hidden", panelClass)}
          >
            {/* Header */}
            <div className={clsx("px-6 py-4 flex justify-between items-center", headerClass)}>
              <h2 className="text-xl font-display tracking-tight text-text-display">Hotkeys / Reference</h2>
              <button onClick={onClose} className="p-1 hover:bg-surface-raised rounded transition-colors text-text-secondary hover:text-text-primary" title="Close"><X size={20} /></button>
            </div>

            <div className="flex-1 p-6 overflow-y-auto grid grid-cols-1 md:grid-cols-2 gap-6 bg-surface text-text-primary">
              
              {/* Top Interactions */}
              <div className="col-span-full grid grid-cols-2 gap-4 bg-surface-raised p-4 rounded-lg border border-border">
                <div className="col-span-full flex flex-wrap items-center justify-between gap-3 border-b border-border pb-3">
                  <div>
                    <p className="text-[11px] font-mono uppercase tracking-widest text-text-secondary">Shortcut preset</p>
                    <p className="text-xs text-text-secondary">Choose your preferred mouse and navigation shortcut scheme.</p>
                  </div>
                  <div className="flex gap-2">
                    <button
                      type="button"
                      onClick={() => onControlSchemeChange('cad')}
                      className={clsx('rounded-lg border px-3 py-2 text-xs font-mono uppercase transition-colors', controlScheme === 'cad' ? 'border-text-display bg-surface text-text-display' : 'border-border bg-card text-text-secondary hover:text-text-primary')}
                    >
                      CAD
                    </button>
                    <button
                      type="button"
                      onClick={() => onControlSchemeChange('standard')}
                      className={clsx('rounded-lg border px-3 py-2 text-xs font-mono uppercase transition-colors', controlScheme === 'standard' ? 'border-text-display bg-surface text-text-display' : 'border-border bg-card text-text-secondary hover:text-text-primary')}
                    >
                      Standard
                    </button>
                  </div>
                </div>

                <div className="flex items-center gap-3">
                  <div className="flex items-center gap-1">
                     <span className={keyClass}>CTRL</span>
                     <span>+</span>
                     <MousePointer2 size={14} className="text-text-secondary" />
                  </div>
                  <span className="text-sm font-mono text-text-secondary">Hide object under cursor</span>
                </div>
                
                <div className="flex items-center gap-3">
                   <div className="flex items-center gap-1">
                     <span className={keyClass}>SHIFT</span>
                     <span>+</span>
                     <MousePointer2 size={14} className="text-text-secondary" />
                   </div>
                   <span className="text-sm font-mono text-text-secondary">Toggle transparency</span>
                </div>

                <div className="flex items-center gap-3">
                   <div className="flex items-center gap-1">
                     <span className={keyClass}>CTRL</span>
                     <span>+</span>
                     <span className={keyClass}>Z</span>
                   </div>
                   <span className="text-sm font-mono text-text-secondary">Undo Action</span>
                </div>

                 <div className="flex items-center gap-3">
                   <div className="flex items-center gap-1">
                     <span className={keyClass}>CTRL</span>
                     <span>+</span>
                     <span className={keyClass}>S</span>
                   </div>
                   <span className="text-sm font-mono text-text-secondary">Save Scene</span>
                </div>
              </div>

              {/* Group Selector */}
              <div className="bg-surface-raised p-4 rounded-lg border border-border">
                <h3 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary mb-4 flex items-center gap-2">
                   <span className="w-2 h-2 rounded-full bg-text-display" /> Show/Hide Groups
                </h3>
                <div className="grid grid-cols-2 gap-2">
                   {groups.map(g => (
                      <div key={g.key} className="flex items-center gap-2">
                         <span className={clsx(keyClass, "text-text-display border-text-display bg-transparent")}>{g.key}</span>
                         <span className="text-xs font-mono text-text-secondary">{g.label}</span>
                      </div>
                   ))}
                </div>
              </div>

              {/* Tools & Sculpting */}
              <div className="space-y-4">
                 <div className="bg-surface-raised p-4 rounded-lg border border-border">
                    <h3 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary mb-4 flex items-center gap-2">
                       <span className="w-2 h-2 rounded-full bg-accent" /> Tools
                    </h3>
                    <div className="space-y-2">
                        <div className="flex justify-between items-center border-b border-border-visible pb-1">
                           <span className="text-xs font-mono text-text-secondary">Distance Tool</span>
                           <div className="flex gap-1"><span className={keyClass}>CTRL</span><span className={keyClass}>D</span></div>
                        </div>
                        <div className="flex justify-between items-center border-b border-border-visible pb-1">
                           <span className="text-xs font-mono text-text-secondary">Measure Tool</span>
                           <div className="flex gap-1"><span className={keyClass}>CTRL</span><span className={keyClass}>R</span></div>
                        </div>
                        <div className="flex justify-between items-center border-b border-border-visible pb-1">
                           <span className="text-xs font-mono text-text-secondary">Section Plane</span>
                           <div className="flex gap-1"><span className={keyClass}>CTRL</span><span className={keyClass}>P</span></div>
                        </div>
                    </div>
                 </div>

                 <div className="bg-surface-raised p-4 rounded-lg border border-border">
                    <h3 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary mb-4 flex items-center gap-2">
                       <span className="w-2 h-2 rounded-full bg-success" /> Free Forming
                    </h3>
                    <div className="grid grid-cols-3 gap-2">
                        {[1,2,3,4,5,6].map(n => (
                           <div key={n} className="flex items-center gap-2 bg-surface p-1 rounded border border-border-visible">
                              <span className={keyClass}>{n}</span>
                              <span className="text-[10px] font-mono text-text-disabled">Mode {n}</span>
                           </div>
                        ))}
                    </div>
                 </div>
              </div>

            </div>
          </motion.div>
        </div>
      )}
    </AnimatePresence>
  );
};
