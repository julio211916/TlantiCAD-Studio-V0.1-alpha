import React from 'react';
import { AlertTriangle, RefreshCw, X } from 'lucide-react';

import { cn } from '@/lib/utils';
import type { ThemeMode } from '@/types';

interface ModulePanelErrorBoundaryProps {
  label: string;
  themeMode: ThemeMode;
  onClose?: () => void;
  children: React.ReactNode;
}

interface ModulePanelErrorBoundaryState {
  error: Error | null;
}

/**
 * V143 — Per-module error boundary so a render failure inside one clinical
 * module (Splint, Implant, Aligners, Surgical Guide, etc.) does not blow up the
 * whole CAD shell with a white screen. The boundary renders a recoverable card
 * inside the same overlay slot the panel would have occupied.
 */
export class ModulePanelErrorBoundary extends React.Component<
  ModulePanelErrorBoundaryProps,
  ModulePanelErrorBoundaryState
> {
  state: ModulePanelErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ModulePanelErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error(`[TlantiCAD] Module "${this.props.label}" failed to render`, error, info);
  }

  private retry = () => {
    this.setState({ error: null });
  };

  render() {
    if (!this.state.error) {
      return this.props.children;
    }

    const isDark = this.props.themeMode === 'dark';

    return (
      <div
        role="alert"
        className={cn(
          'pointer-events-auto absolute left-16 top-24 z-40 w-80 overflow-hidden rounded-lg border shadow-2xl',
          isDark ? 'bg-surface border-border text-text-primary' : 'bg-surface border-border text-text-primary',
        )}
      >
        <div className="flex items-center justify-between border-b border-border bg-surface-raised p-4">
          <div className="flex items-center gap-2">
            <AlertTriangle size={18} className="text-warning" aria-hidden />
            <h3 className="font-display font-semibold tracking-tight text-text-primary">
              {this.props.label} no pudo abrir
            </h3>
          </div>
          {this.props.onClose && (
            <button
              type="button"
              onClick={this.props.onClose}
              aria-label={`Cerrar ${this.props.label}`}
              className="text-text-secondary transition-colors hover:text-text-primary"
            >
              <X size={16} aria-hidden />
            </button>
          )}
        </div>
        <div className="space-y-3 p-4 text-sm">
          <p className="text-text-secondary">
            Este módulo lanzó un error al renderizar. Tu caso y el resto de TlantiCAD siguen estables.
          </p>
          <pre className="max-h-32 overflow-auto rounded border border-border bg-background/70 p-2 text-[11px] text-text-secondary">
            {this.state.error.message}
          </pre>
          <div className="flex gap-2">
            <button
              type="button"
              onClick={this.retry}
              className="inline-flex items-center gap-1 rounded border border-border-visible bg-surface-raised px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-text-primary transition-colors hover:bg-surface"
            >
              <RefreshCw size={12} aria-hidden /> Reintentar
            </button>
            {this.props.onClose && (
              <button
                type="button"
                onClick={this.props.onClose}
                className="inline-flex items-center gap-1 rounded border border-border px-3 py-1.5 text-xs font-semibold uppercase tracking-wider text-text-secondary transition-colors hover:bg-surface"
              >
                Cerrar módulo
              </button>
            )}
          </div>
        </div>
      </div>
    );
  }
}
