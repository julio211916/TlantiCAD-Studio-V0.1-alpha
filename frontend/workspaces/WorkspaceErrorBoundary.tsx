import React from 'react';
import { AlertTriangle, RefreshCw } from 'lucide-react';

import { Button } from '@/components/ui/button';
import { WORKSPACE_TITLES, type AppWorkspaceId } from '@/components/workspaces/workspace.config';
import { cn } from '@/lib/utils';
import type { ThemeMode } from '@/types';

interface WorkspaceErrorBoundaryProps {
  activeWorkspace: AppWorkspaceId;
  children: React.ReactNode;
  onBackToDb?: () => void;
  themeMode: ThemeMode;
}

interface WorkspaceErrorBoundaryState {
  error: Error | null;
  errorInfo: React.ErrorInfo | null;
}

export class WorkspaceErrorBoundary extends React.Component<WorkspaceErrorBoundaryProps, WorkspaceErrorBoundaryState> {
  state: WorkspaceErrorBoundaryState = {
    error: null,
    errorInfo: null,
  };

  static getDerivedStateFromError(error: Error): Partial<WorkspaceErrorBoundaryState> {
    return { error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    this.setState({ errorInfo });
    console.error('[TlantiCAD] Workspace render failure', error, errorInfo);
  }

  componentDidUpdate(previousProps: WorkspaceErrorBoundaryProps) {
    if (previousProps.activeWorkspace !== this.props.activeWorkspace && this.state.error) {
      this.setState({ error: null, errorInfo: null });
    }
  }

  private reloadWorkspace = () => {
    this.setState({ error: null, errorInfo: null });
  };

  render() {
    if (!this.state.error) {
      return this.props.children;
    }

    const workspaceTitle = WORKSPACE_TITLES[this.props.activeWorkspace];
    const isDark = this.props.themeMode === 'dark';

    return (
      <main
        className={cn(
          'grid min-h-dvh place-items-center px-6 py-10',
          isDark ? 'bg-[#080908] text-text-primary' : 'bg-[#f4f2ef] text-[#25212d]'
        )}
      >
        <section className="w-full max-w-xl rounded-md border border-border bg-card p-6 shadow-xl">
          <div className="flex items-start gap-4">
            <img src="/logoo.svg" alt="TlantiCAD Studio" className="size-12 shrink-0 object-contain" />
            <div className="min-w-0 flex-1 space-y-3">
              <div>
                <p className="text-xs font-semibold uppercase tracking-[0.18em] text-text-secondary">
                  {workspaceTitle}
                </p>
                <h1 className="mt-1 text-xl font-semibold text-text-display">No se pudo renderizar esta pantalla.</h1>
              </div>
              <p className="text-sm leading-6 text-text-secondary">
                TlantiCAD aisló el fallo para evitar una ventana negra. Reintenta la vista o vuelve al gestor de casos.
              </p>
              <pre className="max-h-32 overflow-auto rounded border border-border bg-background/70 p-3 text-xs text-text-secondary">
                {this.state.error.message}
              </pre>
              <div className="flex flex-wrap gap-2">
                <Button type="button" size="sm" onClick={this.reloadWorkspace}>
                  <RefreshCw className="mr-2 size-4" />
                  Reintentar
                </Button>
                {this.props.onBackToDb && (
                  <Button type="button" size="sm" variant="outline" onClick={this.props.onBackToDb}>
                    Volver a casos
                  </Button>
                )}
              </div>
            </div>
            <AlertTriangle className="size-5 shrink-0 text-warning" aria-hidden="true" />
          </div>
        </section>
      </main>
    );
  }
}
