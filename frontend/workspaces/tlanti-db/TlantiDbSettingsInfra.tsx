import React from 'react';

export function LazyPanelFallback({ label }: { label: string }) {
    return (
        <div className="rounded-md border border-border bg-surface-raised px-4 py-5 text-sm text-text-secondary">
            {label}
        </div>
    );
}

export class SettingsErrorBoundary extends React.Component<
    { children: React.ReactNode },
    { error: string | null }
> {
    state = { error: null };

    static getDerivedStateFromError(error: unknown) {
        return { error: error instanceof Error ? error.message : 'Settings panel failed to render.' };
    }

    componentDidCatch(error: unknown) {
        console.error('TlantiDB settings panel render failure', error);
    }

    render() {
        if (this.state.error) {
            return (
                <div className="rounded-md border border-red-500/40 bg-red-500/10 px-4 py-3 text-sm text-red-100">
                    <p className="font-semibold">Settings could not render.</p>
                    <p className="mt-1 text-red-100/80">{this.state.error}</p>
                </div>
            );
        }

        return this.props.children;
    }
}
