import React, { lazy } from 'react';

import type { Language, ThemeMode } from '@/types';

const CadInterface = lazy(() => import('./CadInterface').then((module) => ({ default: module.CadInterface })));

export interface CadWorkspaceHostProps {
  language: Language;
  setLanguage: (lang: Language) => void;
  themeMode: ThemeMode;
  setThemeMode: (mode: ThemeMode) => void;
  caseId?: string;
  moduleId?: string;
  onBackToDb: () => void;
}

export function createCadWorkspaceInstanceKey({ caseId, moduleId }: Pick<CadWorkspaceHostProps, 'caseId' | 'moduleId'>) {
  return `${caseId ?? 'no-case'}:${moduleId ?? 'cad'}`;
}

export function CadWorkspaceHost(props: CadWorkspaceHostProps) {
  const workspaceKey = createCadWorkspaceInstanceKey(props);

  return <CadInterface key={`cad:${workspaceKey}`} {...props} />;
}
