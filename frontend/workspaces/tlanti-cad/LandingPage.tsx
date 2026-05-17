import React from 'react';
import { motion } from 'framer-motion';
import { THEME } from '../types';
import { ChevronRight, Box } from 'lucide-react';
import { LiquidMetalButton } from './ui/liquid-metal-button';

interface Props {
  onEnter: () => void;
}

export const LandingPage: React.FC<Props> = ({ onEnter }) => {
  return (
    <div className="relative flex h-dvh w-full flex-col items-center justify-center overflow-hidden bg-[#010101]">
      {/* Abstract Background Elements */}
      <div className="absolute top-[-20%] left-[-10%] w-[50vw] h-[50vw] rounded-full bg-purple-900/20 blur-[120px]" />
      <div className="absolute bottom-[-20%] right-[-10%] w-[50vw] h-[50vw] rounded-full bg-pink-900/20 blur-[120px]" />
      
      {/* Grid Pattern Overlay */}
      <div className="absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.03)_1px,transparent_1px)] bg-[size:40px_40px]" />

      <motion.div 
        initial={{ opacity: 0, y: 40 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.8, ease: [0.22, 1, 0.36, 1] }}
        className="z-10 text-center max-w-4xl px-6"
      >
        <div className="mb-6 flex justify-center">
          <motion.div 
            animate={{ rotate: 360 }}
            transition={{ duration: 20, repeat: Infinity, ease: "linear" }}
            className={`p-5 rounded-3xl ${THEME.glassDark} border-white/20`}
          >
            <Box className="w-14 h-14 text-pink-400" />
          </motion.div>
        </div>

        <h1 className="text-6xl md:text-8xl font-bold tracking-tighter mb-6 text-white">
          Tlanti<span className={THEME.gradientText}>4CAD</span>
        </h1>
        
        <p className="text-lg md:text-2xl text-gray-400 mb-10 max-w-2xl mx-auto font-light">
          Professional browser-based 3D modeling & DICOM visualization. 
          Real algebraic tools. No installation required.
        </p>

        <div className="flex justify-center">
          <LiquidMetalButton label="Launch Workspace" onClick={onEnter} />
        </div>
      </motion.div>

      {/* Footer Features */}
      <div className="absolute bottom-10 flex gap-8 text-xs text-gray-500 uppercase tracking-widest">
        <span>DICOM Support</span>
        <span>•</span>
        <span>Mesh Analysis</span>
        <span>•</span>
        <span>Algebraic Tools</span>
      </div>
    </div>
  );
};