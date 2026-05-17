import React from 'react';

import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';
import { Cpu, Image as ImageIcon, Server, TriangleAlert } from 'lucide-react';

export type TlantiOhifRenderMode = 'loading' | 'cornerstone' | 'local-preview' | 'backend-preview' | 'error';

interface TlantiOhifRenderStatusProps {
    className?: string;
    mode: TlantiOhifRenderMode;
}

const MODE_COPY: Record<TlantiOhifRenderMode, { icon: React.ComponentType<{ size?: number; className?: string }>; label: string; tone: string }> = {
    loading: {
        icon: Cpu,
        label: 'Preparando render',
        tone: 'border-border-visible bg-surface text-text-secondary',
    },
    cornerstone: {
        icon: Cpu,
        label: 'Cornerstone render',
        tone: 'border-emerald-400/40 bg-emerald-500/10 text-emerald-300',
    },
    'local-preview': {
        icon: ImageIcon,
        label: 'Preview local',
        tone: 'border-[#FA93FA]/40 bg-[#FA93FA]/10 text-[#FA93FA]',
    },
    'backend-preview': {
        icon: Server,
        label: 'Preview backend',
        tone: 'border-sky-400/40 bg-sky-500/10 text-sky-300',
    },
    error: {
        icon: TriangleAlert,
        label: 'Render con error',
        tone: 'border-red-500/30 bg-red-500/10 text-red-300',
    },
};

export function TlantiOhifRenderStatus({ className, mode }: TlantiOhifRenderStatusProps) {
    const { icon: Icon, label, tone } = MODE_COPY[mode];

    return (
        <Badge
            variant="outline"
            className={cn('gap-1.5 rounded-full px-3 py-1 text-[11px] font-medium tabular-nums', tone, className)}
        >
            <Icon size={12} />
            {label}
        </Badge>
    );
}