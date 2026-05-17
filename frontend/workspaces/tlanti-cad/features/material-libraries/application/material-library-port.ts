import type { MaterialLibraryEntry } from '../domain/material-library';

export interface MaterialLibraryPort {
    listLocal(): Promise<MaterialLibraryEntry[]>;
    /**
     * Material libraries received via P2P from peers on the LAN
     * (TlantiShare). Returns the list of bundles that landed in the local
     * inbox; the protocol that fills it is `local_share_listen` →
     * `tlantishare://recv-complete`. No central server.
     */
    listPeerLibraries(): Promise<MaterialLibraryEntry[]>;
    /** Forget a previously received peer library bundle. */
    forgetPeerLibrary(libraryId: string): Promise<{ ok: boolean }>;
}
