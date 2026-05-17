import React, { useState } from 'react';
import { motion } from 'framer-motion';
import { ArrowLeft, Check, Eraser } from 'lucide-react';
import clsx from 'clsx';

interface ToothConfigPanelProps {
  toothNumber: number;
  onClose: () => void;
}

const CATEGORIES = [
  {
    title: 'Crowns and copings',
    options: [
      { id: 'anatomic_crown', label: 'Anatomic crown', color: 'bg-[#9c27b0]' },
      { id: 'coping', label: 'Coping', color: 'bg-[#009688]' },
      { id: 'pressed_crown', label: 'Pressed crown', color: 'bg-[#c0ca33]' },
      { id: 'eggshell_crown', label: 'Eggshell crown (Provisional)', color: 'bg-[#ab47bc]' },
      { id: 'overlay', label: 'Overlay', color: 'bg-[#78909c]' },
      { id: 'offset_coping', label: 'Offset coping', color: 'bg-[#4caf50]' },
    ]
  },
  {
    title: 'Pontics and Mockup',
    options: [
      { id: 'anatomic_pontic', label: 'Anatomic pontic', color: 'bg-[#b71c1c]' },
      { id: 'reduced_pontic', label: 'Reduced pontic', color: 'bg-[#e53935]' },
      { id: 'pressed_pontic', label: 'Pressed pontic', color: 'bg-[#42a5f5]' },
      { id: 'eggshell_pontic', label: 'Eggshell pontic (Provisional)', color: 'bg-[#ba68c8]' },
      { id: 'mockup', label: 'Mockup', color: 'bg-[#e57373]' },
    ]
  },
  {
    title: 'Inlays, onlays and veneers',
    options: [
      { id: 'inlay_onlay', label: 'Inlay/Onlay', color: 'bg-[#388e3c]' },
      { id: 'offset_inlay', label: 'Offset inlay', color: 'bg-[#1976d2]' },
      { id: 'veneer', label: 'Veneer', color: 'bg-[#26a69a]' },
    ]
  },
  {
    title: 'Digital copy milling',
    options: [
      { id: 'anatomic_waxup', label: 'Anatomic waxup', color: 'bg-[#2e7d32]' },
      { id: 'reduced_waxup', label: 'Reduced waxup', color: 'bg-[#616161]' },
      { id: 'pontic_waxup', label: 'Pontic waxup', color: 'bg-[#3f51b5]' },
    ]
  },
  {
    title: 'Removables and appliances',
    options: [
      { id: 'full_denture', label: 'Full denture', color: 'bg-[#4dd0e1]' },
      { id: 'partial_denture', label: 'Partial denture', color: 'bg-[#d4e157]' },
      { id: 'bite_splint', label: 'Bite splint', color: 'bg-[#455a64]' },
      { id: 'primary_telescopic', label: 'Primary telescopic crown', color: 'bg-[#e06666]' },
      { id: 'secondary_telescopic', label: 'Secondary telescopic crown', color: 'bg-[#8d6e63]' },
      { id: 'attachment', label: 'Attachment', color: 'bg-[#006064]' },
    ]
  },
  {
    title: 'Bars',
    options: [
      { id: 'bar_pillar', label: 'Bar pillar', color: 'bg-[#827717]' },
      { id: 'bar_segment', label: 'Bar segment', color: 'bg-[#6a1b9a]' },
      { id: 'offset_substructure', label: 'Offset substructure', color: 'bg-[#a1887f]' },
    ]
  },
  {
    title: 'Residual dentition',
    options: [
      { id: 'antagonist', label: 'Antagonist', color: 'bg-[#ff9800]' },
      { id: 'adjacent_tooth', label: 'Adjacent tooth', color: 'bg-[#ffb300]' },
      { id: 'omit_in_bridge', label: 'Omit in bridge', color: 'bg-[#f44336]' },
    ]
  }
];

const MATERIALS = [
  { id: 'zirconia', label: 'Zirconia', color: '#f0f0f0' },
  { id: 'zirconia_trans', label: 'Zirconia Translucent', color: '#e8e8e8' },
  { id: 'zirconia_multi', label: 'Zirconia Multilayer', color: '#e0e0e0' },
  { id: 'acrylic_pmma', label: 'Acrylic/PMMA', color: '#f5f5dc' },
  { id: 'composite', label: 'Composite', color: '#fffdd0' },
  { id: 'np_metal', label: 'NP Metal', color: '#b0c4de' },
  { id: 'titanium', label: 'Titanium', color: '#87ceeb' },
  { id: 'np_metal_laser', label: 'NP Metal (Laser)', color: '#778899' },
  { id: 'titanium_laser', label: 'Titanium (Laser)', color: '#708090' },
  { id: '3d_print', label: '3D Print', color: '#d3d3d3' },
  { id: 'wax', label: 'WAX', color: '#2e8b57' },
  { id: 'lithium_disilicate', label: 'Lithium Disilicate', color: '#dda0dd' },
];

const PRESETS = [
  { id: 'std_zirconia', label: 'Standard Zirconia', material: 'zirconia' },
  { id: 'ht_emax', label: 'High Translucency Emax', material: 'lithium_disilicate' },
  { id: 'temp_pmma', label: 'Temporary PMMA', material: 'acrylic_pmma' },
];

export const ToothConfigPanel: React.FC<ToothConfigPanelProps> = ({ toothNumber, onClose }) => {
  const [selectedOption, setSelectedOption] = useState<string | null>('offset_inlay');
  const [selectedMaterial, setSelectedMaterial] = useState<string | null>('zirconia');

  return (
    <motion.div 
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: 20 }}
      className="absolute inset-0 z-50 bg-surface flex flex-col overflow-hidden text-text-primary"
    >
      {/* Header */}
      <div className="flex items-center p-4 border-b border-border">
        <button onClick={onClose} className="p-2 mr-2 hover:bg-surface-raised rounded transition-colors">
          <ArrowLeft size={24} className="text-text-secondary hover:text-text-primary" />
        </button>
        <h2 className="text-3xl font-display tracking-tight text-text-primary">
          Tooth <span className="font-semibold">{toothNumber}</span>
        </h2>
        <span className="ml-4 text-[11px] font-mono uppercase tracking-widest text-text-secondary">Material configuration (local): Default</span>
      </div>

      <div className="flex flex-1 overflow-hidden">
        {/* Left Column: Options */}
        <div className="w-2/3 overflow-y-auto p-6 border-r border-border">
          {CATEGORIES.map((category) => (
            <div key={category.title} className="mb-6">
              <h3 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary mb-3">{category.title}</h3>
              <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 gap-2">
                {category.options.map((opt) => (
                  <button
                    key={opt.id}
                    onClick={() => setSelectedOption(opt.id)}
                    className={clsx(
                      "relative flex items-center p-2 rounded text-white text-sm font-medium transition-transform active:scale-95 text-left",
                      opt.color,
                      selectedOption === opt.id ? "ring-2 ring-text-display ring-offset-2 ring-offset-surface" : "hover:brightness-110"
                    )}
                  >
                    <div className="w-8 h-8 mr-2 bg-white/20 rounded flex items-center justify-center shrink-0">
                      {/* Placeholder for icon */}
                      <div className="w-5 h-5 bg-white/50 rounded-sm" />
                    </div>
                    <span className="leading-tight">{opt.label}</span>
                    {selectedOption === opt.id && (
                      <div className="absolute top-1 right-1 bg-text-display rounded-full p-0.5 shadow-sm">
                        <Check size={12} className="text-black" />
                      </div>
                    )}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>

        {/* Right Column: Materials */}
        <div className="w-1/3 flex flex-col bg-surface-raised">
          <div className="p-6 border-b border-border">
            <h3 className="text-2xl font-display tracking-tight text-text-primary mb-4">Material</h3>
            
            <div className="mb-6">
              <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary mb-2">Material Presets</h4>
              <div className="flex flex-col gap-2">
                {PRESETS.map(preset => (
                  <button
                    key={preset.id}
                    onClick={() => setSelectedMaterial(preset.material)}
                    className={clsx(
                      "text-left px-3 py-2 rounded border transition-colors text-sm font-mono",
                      selectedMaterial === preset.material 
                        ? "border-text-display bg-text-display/10 text-text-display" 
                        : "border-border-visible hover:bg-surface text-text-primary"
                    )}
                  >
                    {preset.label}
                  </button>
                ))}
              </div>
            </div>

            <h4 className="text-[11px] font-mono uppercase tracking-widest text-text-secondary mb-2">All Materials</h4>
            <select className="w-full p-2 border border-border-visible rounded bg-surface text-text-primary font-mono text-sm mb-4 outline-none focus:border-text-display">
              <option>5-Axis / Laser / 3D Print</option>
            </select>
            
            <div className="grid grid-cols-2 gap-4 overflow-y-auto max-h-[250px] pr-2">
              {MATERIALS.map((mat) => (
                <button
                  key={mat.id}
                  onClick={() => setSelectedMaterial(mat.id)}
                  className={clsx(
                    "flex flex-col items-center p-2 rounded-lg border-2 transition-all",
                    selectedMaterial === mat.id 
                      ? "border-text-display bg-text-display/10" 
                      : "border-transparent hover:border-border-visible"
                  )}
                >
                  <div 
                    className="w-16 h-16 rounded-full mb-2 shadow-inner border border-border-visible"
                    style={{ backgroundColor: mat.color }}
                  />
                  <span className="text-[11px] font-mono uppercase tracking-widest text-center text-text-primary">{mat.label}</span>
                </button>
              ))}
            </div>
          </div>
          
          <div className="p-6 flex-1">
             <p className="text-[11px] font-mono uppercase tracking-widest text-text-secondary mb-2">You are using the following material configuration:</p>
             <p className="font-bold text-text-primary mb-4">exocad default</p>
             <a href="#" className="text-text-display hover:underline text-sm font-mono">More info on this material selection...</a>
          </div>

          {/* Footer Actions */}
          <div className="p-4 border-t border-border flex justify-end gap-4 bg-surface">
            <button 
              onClick={() => { setSelectedOption(null); setSelectedMaterial(null); }}
              className="flex items-center gap-2 px-6 py-3 rounded border border-border-visible text-text-primary hover:bg-surface-raised transition-colors font-mono text-sm uppercase tracking-widest"
            >
              <Eraser size={18} /> Clear
            </button>
            <button 
              onClick={onClose}
              className="flex items-center gap-2 px-8 py-3 rounded bg-text-display hover:bg-text-display/90 text-black transition-colors font-mono text-sm uppercase tracking-widest font-bold"
            >
              <Check size={18} /> OK
            </button>
          </div>
        </div>
      </div>
    </motion.div>
  );
};
