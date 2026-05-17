/**
 * AppIcon — resolves a semantic icon name through the manifest and renders
 * it as either a CSS-masked SVG (TlantiCAD asset), an <img> (legacy asset),
 * a Lucide React component, or a dashed placeholder when missing.
 *
 * Always use this instead of hard-coding icon paths in components so the
 * manifest stays authoritative and gap reports remain accurate.
 */

import React from 'react';

import { APP_ICONS, type AppIconName } from './icons-manifest';

type AppIconProps = {
    name: AppIconName | (string & {});
    className?: string;
    /** Pixel size; defaults to 16. Applied to width/height and stroke. */
    size?: number;
    'aria-label'?: string;
    'aria-hidden'?: boolean | 'true' | 'false';
    title?: string;
};

export function AppIcon({ name, className, size = 16, title, ...aria }: AppIconProps) {
    const entry = APP_ICONS[name as AppIconName];

    if (!entry) {
        return (
            <span
                role="img"
                aria-label={aria['aria-label'] ?? `Unknown icon: ${name}`}
                title={title ?? `Unknown icon: ${name}`}
                className={['inline-block rounded border border-dashed border-red-400/70', className]
                    .filter(Boolean)
                    .join(' ')}
                style={{ width: size, height: size }}
            />
        );
    }

    const src = entry.source;

    if (src.kind === 'svg') {
        const hidden = aria['aria-hidden'] === true || aria['aria-hidden'] === 'true';
        const label = aria['aria-label'] ?? entry.description;
        return (
            <span
                role={hidden ? undefined : 'img'}
                aria-label={hidden ? undefined : label}
                aria-hidden={hidden}
                title={title ?? entry.description}
                className={['inline-block shrink-0 bg-current', className].filter(Boolean).join(' ')}
                style={{
                    width: size,
                    height: size,
                    WebkitMask: `url("${src.path}") center / contain no-repeat`,
                    mask: `url("${src.path}") center / contain no-repeat`,
                }}
            />
        );
    }

    if (src.kind === 'exocad') {
        return (
            <img
                src={src.path}
                alt={aria['aria-label'] ?? entry.description}
                title={title ?? entry.description}
                width={size}
                height={size}
                className={className}
                draggable={false}
                loading="lazy"
                aria-hidden={aria['aria-hidden']}
            />
        );
    }

    if (src.kind === 'lucide') {
        const Icon = src.icon;
        return (
            <Icon
                width={size}
                height={size}
                className={className}
                aria-label={aria['aria-label']}
                aria-hidden={aria['aria-hidden']}
            />
        );
    }

    // kind === 'missing' — a tagged gap so the user can commission it.
    return (
        <span
            role="img"
            aria-label={aria['aria-label'] ?? `Missing icon: ${name} (${src.hint})`}
            title={title ?? `Missing: ${src.hint}`}
            className={[
                'inline-block rounded border border-dashed border-yellow-400/70 bg-yellow-500/10',
                className,
            ]
                .filter(Boolean)
                .join(' ')}
            style={{ width: size, height: size }}
        />
    );
}
