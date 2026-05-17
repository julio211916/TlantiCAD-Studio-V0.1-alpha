/**
 * LocalSharePanel — V46.
 *
 * Modal that lists nearby peers (mDNS / Bluetooth / AirDrop) and lets the
 * lab tech send the active case to one of them. Pure presentational; the
 * parent owns peer state + send action.
 *
 * Strings are i18n-keyed (`share.*`).
 */

import React from 'react';

import { useT } from '../../../lib/i18n';
import { intersectTransports, type Peer, type ShareTransport } from '../domain/peer';

export interface LocalSharePanelProps {
    open: boolean;
    onClose: () => void;
    peers: readonly Peer[];
    isDiscovering: boolean;
    activeTransport: ShareTransport;
    localTransports: readonly ShareTransport[];
    onTransportChange: (transport: ShareTransport) => void;
    onSend: (peer: Peer, transport: ShareTransport) => void;
    error?: string | null;
}

const TRANSPORT_LABELS: Record<ShareTransport, string> = {
    airdrop: 'share.transport.airdrop',
    bluetooth: 'share.transport.bluetooth',
    lan: 'share.transport.lan',
};

export function LocalSharePanel({
    open,
    onClose,
    peers,
    isDiscovering,
    activeTransport,
    localTransports,
    onTransportChange,
    onSend,
    error,
}: LocalSharePanelProps) {
    const t = useT();

    if (!open) return null;

    return (
        <div
            role="dialog"
            aria-modal="true"
            aria-label={t('share.title')}
            className="fixed inset-0 z-[150] flex items-center justify-center bg-black/55 p-6 backdrop-blur-sm"
            onClick={(e) => {
                if (e.target === e.currentTarget) onClose();
            }}
        >
            <div className="flex w-full max-w-lg flex-col overflow-hidden rounded-xl border border-border bg-surface-raised shadow-2xl">
                <header className="border-b border-border px-4 py-3">
                    <h2 className="text-sm font-semibold text-text-primary">
                        {t('share.title')}
                    </h2>
                    <p className="text-[11px] text-text-secondary">{t('share.subtitle')}</p>
                </header>

                <nav className="flex border-b border-border bg-surface-sunken/40 text-[11px] uppercase tracking-wider">
                    {(Object.keys(TRANSPORT_LABELS) as ShareTransport[]).map((transport) => {
                        const supported = localTransports.includes(transport);
                        const active = transport === activeTransport;
                        return (
                            <button
                                key={transport}
                                type="button"
                                onClick={() => supported && onTransportChange(transport)}
                                disabled={!supported}
                                className={[
                                    'flex-1 px-3 py-2 transition',
                                    active
                                        ? 'border-b-2 border-sky-400 text-text-primary'
                                        : supported
                                          ? 'text-text-secondary hover:text-text-primary'
                                          : 'cursor-not-allowed text-text-disabled',
                                ].join(' ')}
                            >
                                {t(TRANSPORT_LABELS[transport])}
                            </button>
                        );
                    })}
                </nav>

                <div className="max-h-[50vh] overflow-y-auto">
                    {peers.length === 0 ? (
                        <p className="px-4 py-6 text-center text-[11px] text-text-secondary">
                            {isDiscovering ? '…' : t('share.peers.empty')}
                        </p>
                    ) : (
                        <ul role="listbox" className="flex flex-col">
                            {peers.map((peer) => {
                                const compatible = intersectTransports(peer, localTransports);
                                const canSend = compatible.includes(activeTransport);
                                return (
                                    <li
                                        key={peer.id}
                                        className="flex items-center gap-3 border-b border-border px-3 py-2.5 text-sm"
                                    >
                                        <div className="min-w-0 flex-1">
                                            <div className="truncate font-semibold text-text-primary">
                                                {peer.name}
                                            </div>
                                            <div className="truncate text-[11px] text-text-secondary">
                                                {peer.platform} ·{' '}
                                                {peer.transports
                                                    .map((tt) => t(TRANSPORT_LABELS[tt]))
                                                    .join(' · ')}
                                            </div>
                                            {peer.status === 'sending' ? (
                                                <div className="text-[10px] text-sky-300">…</div>
                                            ) : peer.status === 'success' ? (
                                                <div className="text-[10px] text-emerald-300">✓</div>
                                            ) : peer.status === 'error' && peer.errorMessage ? (
                                                <div className="text-[10px] text-rose-300">
                                                    {peer.errorMessage}
                                                </div>
                                            ) : null}
                                        </div>
                                        <button
                                            type="button"
                                            onClick={() => onSend(peer, activeTransport)}
                                            disabled={!canSend || peer.status === 'sending'}
                                            className="rounded-md bg-sky-500 px-3 py-1.5 text-[11px] font-semibold text-white disabled:opacity-50"
                                        >
                                            {t('share.action.send')}
                                        </button>
                                    </li>
                                );
                            })}
                        </ul>
                    )}
                </div>

                {error ? (
                    <p className="border-t border-rose-500/40 bg-rose-500/10 px-3 py-1.5 text-[11px] text-rose-200">
                        {error}
                    </p>
                ) : null}

                <footer className="flex items-center gap-2 border-t border-border bg-surface-sunken/40 px-4 py-3">
                    <button
                        type="button"
                        onClick={onClose}
                        className="ml-auto rounded-md border border-border bg-surface-sunken px-3 py-1.5 text-xs"
                    >
                        ✕
                    </button>
                </footer>
            </div>
        </div>
    );
}
