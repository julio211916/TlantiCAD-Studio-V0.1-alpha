/**
 * Tauri bridge for the case-storage stack (V238 + V239 + V240 + V241).
 *
 * Each helper invokes its Rust counterpart when running inside Tauri; in the
 * browser preview it returns deterministic stubs so panel UI stays usable.
 */

import { isTauriRuntime } from '../platform/desktop-system';
import { logger } from './logger';

export interface CaseStoragePaths {
    appData: string;
    documents: string;
    defaultLibrary: string;
    defaultCasesRoot: string;
}

export interface CaseValidationReport {
    ok: boolean;
    caseId: string | null;
    caseNumber: string | null;
    schemaVersion: string | null;
    assetsDirPresent: boolean;
    interopDirPresent: boolean;
    auditLogPresent: boolean;
    messages: string[];
}

export interface EncryptedBlob {
    nonceB64: string;
    ciphertextB64: string;
}

export interface AuditEntry {
    seq: number;
    timestamp: string;
    prevHmac: string;
    event: Record<string, unknown>;
    hmac: string;
}

async function invokeIfTauri<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    if (!isTauriRuntime()) {
        throw { kind: 'tauri-bridge-not-available' };
    }
    const { invoke } = await import('@tauri-apps/api/core');
    return invoke<T>(command, args);
}

export async function resolveCaseStoragePaths(): Promise<CaseStoragePaths | null> {
    try {
        return await invokeIfTauri<CaseStoragePaths>('resolve_paths');
    } catch (err) {
        logger.warn('resolve_paths fallback (browser)', err);
        return null;
    }
}

export async function validateCaseFolder(folder: string): Promise<CaseValidationReport | null> {
    try {
        return await invokeIfTauri<CaseValidationReport>('validate_case', { folder });
    } catch (err) {
        logger.warn('validate_case fallback', err);
        return null;
    }
}

export async function caseBlobEncrypt(plaintext: string, keyB64: string): Promise<EncryptedBlob | null> {
    try {
        return await invokeIfTauri<EncryptedBlob>('case_blob_encrypt', {
            request: { plaintext, keyB64 },
        });
    } catch (err) {
        logger.warn('case_blob_encrypt fallback', err);
        return null;
    }
}

export async function caseBlobDecrypt(blob: EncryptedBlob, keyB64: string): Promise<string | null> {
    try {
        return await invokeIfTauri<string>('case_blob_decrypt', {
            request: {
                keyB64,
                nonceB64: blob.nonceB64,
                ciphertextB64: blob.ciphertextB64,
            },
        });
    } catch (err) {
        logger.warn('case_blob_decrypt fallback', err);
        return null;
    }
}

export async function appendAuditEntry(
    caseFolder: string,
    event: Record<string, unknown>,
    hmacKeyB64: string,
): Promise<AuditEntry | null> {
    try {
        return await invokeIfTauri<AuditEntry>('audit_append', {
            request: { caseFolder, event, hmacKeyB64 },
        });
    } catch (err) {
        logger.warn('audit_append fallback', err);
        return null;
    }
}
