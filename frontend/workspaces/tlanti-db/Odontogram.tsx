import React, { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { X } from 'lucide-react';
import { ToothConfigPanel } from './ToothConfigPanel';

type Surface = 'top' | 'right' | 'bottom' | 'left' | 'center';

export interface SurfaceState {
  color: string;
  type: string;
}

interface ToothProps {
  number: number;
  states: Partial<Record<Surface, SurfaceState>>;
  onSurfaceClick: (toothNumber: number, surface: Surface) => void;
  onToothSelect: (toothNumber: number) => void;
}

const Tooth: React.FC<ToothProps> = ({ number, states, onSurfaceClick, onToothSelect }) => {
  return (
    <div className="flex flex-col items-center m-0.5">
      <button 
        onClick={() => onToothSelect(number)}
        className="text-[10px] font-bold mb-1 text-gray-600 dark:text-gray-400 hover:text-blue-500 dark:hover:text-blue-400 transition-colors cursor-pointer"
      >
        {number}
      </button>
      <svg 
        viewBox="0 0 100 100" 
        className="w-8 h-8 sm:w-10 sm:h-10 cursor-pointer drop-shadow-sm"
        onContextMenu={(e) => { e.preventDefault(); onToothSelect(number); }}
      >
        {/* Top */}
        <path 
          d="M 18.18 18.18 A 45 45 0 0 1 81.82 18.18 L 64.14 35.86 A 20 20 0 0 0 35.86 35.86 Z" 
          fill={states.top?.color || 'transparent'} 
          stroke="currentColor" 
          strokeWidth="3"
          onClick={() => onSurfaceClick(number, 'top')}
          className="hover:opacity-70 transition-opacity text-gray-400 dark:text-gray-500"
        />
        {/* Right */}
        <path 
          d="M 81.82 18.18 A 45 45 0 0 1 81.82 81.82 L 64.14 64.14 A 20 20 0 0 0 64.14 35.86 Z" 
          fill={states.right?.color || 'transparent'} 
          stroke="currentColor" 
          strokeWidth="3"
          onClick={() => onSurfaceClick(number, 'right')}
          className="hover:opacity-70 transition-opacity text-gray-400 dark:text-gray-500"
        />
        {/* Bottom */}
        <path 
          d="M 81.82 81.82 A 45 45 0 0 1 18.18 81.82 L 35.86 64.14 A 20 20 0 0 0 64.14 64.14 Z" 
          fill={states.bottom?.color || 'transparent'} 
          stroke="currentColor" 
          strokeWidth="3"
          onClick={() => onSurfaceClick(number, 'bottom')}
          className="hover:opacity-70 transition-opacity text-gray-400 dark:text-gray-500"
        />
        {/* Left */}
        <path 
          d="M 18.18 81.82 A 45 45 0 0 1 18.18 18.18 L 35.86 35.86 A 20 20 0 0 0 35.86 64.14 Z" 
          fill={states.left?.color || 'transparent'} 
          stroke="currentColor" 
          strokeWidth="3"
          onClick={() => onSurfaceClick(number, 'left')}
          className="hover:opacity-70 transition-opacity text-gray-400 dark:text-gray-500"
        />
        {/* Center */}
        <circle 
          cx="50" cy="50" r="20" 
          fill={states.center?.color || 'transparent'} 
          stroke="currentColor" 
          strokeWidth="3"
          onClick={() => onSurfaceClick(number, 'center')}
          className="hover:opacity-70 transition-opacity text-gray-400 dark:text-gray-500"
        />
      </svg>
    </div>
  );
};

const UPPER_RIGHT = [18, 17, 16, 15, 14, 13, 12, 11];
const UPPER_LEFT = [21, 22, 23, 24, 25, 26, 27, 28];
const LOWER_RIGHT = [48, 47, 46, 45, 44, 43, 42, 41];
const LOWER_LEFT = [31, 32, 33, 34, 35, 36, 37, 38];

export const Odontogram = ({ onClose }: { onClose?: () => void }) => {
  const [toothStates, setToothStates] = useState<Record<number, Partial<Record<Surface, SurfaceState>>>>({});
  const [selectedColor, setSelectedColor] = useState<string>('#ef4444'); // Default red
  const [selectedType, setSelectedType] = useState<string>('Caries');
  const [selectedTooth, setSelectedTooth] = useState<number | null>(null);

  const handleSurfaceClick = (number: number, surface: Surface) => {
    setToothStates(prev => {
      const current = prev[number]?.[surface];
      const isSame = current?.color === selectedColor && current?.type === selectedType;
      return {
        ...prev,
        [number]: {
          ...(prev[number] || {}),
          [surface]: isSame ? undefined : { color: selectedColor, type: selectedType }
        }
      };
    });
  };

  const renderArch = (right: number[], left: number[]) => (
    <div className="flex justify-center gap-2 sm:gap-4 mb-8">
      <div className="flex gap-0.5 sm:gap-1">
        {right.map(num => (
          <Tooth 
            key={num} 
            number={num} 
            states={toothStates[num] || {}} 
            onSurfaceClick={handleSurfaceClick} 
            onToothSelect={setSelectedTooth}
          />
        ))}
      </div>
      <div className="w-2 sm:w-4 border-l-2 border-gray-300 dark:border-gray-600"></div>
      <div className="flex gap-0.5 sm:gap-1">
        {left.map(num => (
          <Tooth 
            key={num} 
            number={num} 
            states={toothStates[num] || {}} 
            onSurfaceClick={handleSurfaceClick} 
            onToothSelect={setSelectedTooth}
          />
        ))}
      </div>
    </div>
  );

  return (
    <motion.div 
      initial={{ opacity: 0, y: 20, scale: 0.95 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: 20, scale: 0.95 }}
      className="absolute inset-4 md:inset-10 z-[100] bg-white dark:bg-gray-900 rounded-2xl shadow-2xl border border-gray-200 dark:border-gray-800 flex flex-col overflow-hidden"
    >
      <div className="flex justify-between items-center p-4 border-b border-gray-200 dark:border-gray-800">
        <h2 className="text-xl font-bold text-gray-800 dark:text-gray-200">Digital Odontogram</h2>
        {onClose && (
          <button onClick={onClose} className="p-2 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-full transition-colors text-gray-500 dark:text-gray-400">
            <X size={20} />
          </button>
        )}
      </div>
      
      <div className="p-4 bg-gray-50 dark:bg-gray-800/50 flex flex-wrap justify-center items-center gap-4 sm:gap-6 border-b border-gray-200 dark:border-gray-800">
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Condition:</label>
          <select 
            value={selectedType} 
            onChange={(e) => setSelectedType(e.target.value)}
            className="p-2 rounded-lg border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 text-sm focus:ring-2 focus:ring-blue-500 outline-none text-gray-800 dark:text-gray-200"
          >
            <option value="Caries">Caries</option>
            <option value="Restoration">Restoration</option>
            <option value="Extracted">Extracted</option>
            <option value="Crown">Crown</option>
            <option value="Implant">Implant</option>
            <option value="Other">Other</option>
          </select>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">Color:</label>
          <div className="relative w-8 h-8 rounded-full overflow-hidden border-2 border-gray-300 dark:border-gray-600 shadow-sm">
            <input 
              type="color" 
              value={selectedColor} 
              onChange={(e) => setSelectedColor(e.target.value)} 
              className="absolute -top-2 -left-2 w-12 h-12 cursor-pointer"
            />
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-4 sm:p-8 flex flex-col items-center justify-start sm:justify-center bg-gray-100/50 dark:bg-gray-900/50 relative">
        <div className="bg-white dark:bg-gray-800 p-6 sm:p-10 rounded-2xl shadow-sm border border-gray-200 dark:border-gray-700 max-w-full overflow-x-auto">
          <div className="min-w-[560px] sm:min-w-[700px]">
            {renderArch(UPPER_RIGHT, UPPER_LEFT)}
            {renderArch(LOWER_RIGHT, LOWER_LEFT)}
          </div>
        </div>
        
        <AnimatePresence>
          {selectedTooth !== null && (
            <ToothConfigPanel 
              toothNumber={selectedTooth} 
              onClose={() => setSelectedTooth(null)} 
            />
          )}
        </AnimatePresence>
      </div>
    </motion.div>
  );
};
