import React from 'react';
import { TlantiWorkspacePreloader } from './preloaders/TlantiWorkspacePreloader';

export const Preloader: React.FC = () => {
  return <TlantiWorkspacePreloader visible title="TlantiCAD" subtitle="Dental Workspace" themeMode="dark" />;
};