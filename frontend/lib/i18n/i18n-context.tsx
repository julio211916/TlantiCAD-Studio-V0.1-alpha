/**
 * I18nProvider + useT hook (V262 scaffold).
 *
 * Lightweight in-tree i18n — no `react-i18next` dependency.  Locale is
 * stored in localStorage so it survives reloads. Default locale is `es`;
 * other locales fall back to es when a key is missing.
 */

import React, { createContext, useCallback, useContext, useEffect, useMemo, useState } from 'react';

import {
    DEFAULT_LOCALE,
    MESSAGES,
    SUPPORTED_LOCALES,
    pickInitialLocale,
    type MessageCatalog,
    type SupportedLocale,
} from './messages';

const STORAGE_KEY = 'tlanticad.locale';

interface I18nContextValue {
    locale: SupportedLocale;
    setLocale: (next: SupportedLocale) => void;
    t: (key: string, params?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

function readStoredLocale(): SupportedLocale {
    if (typeof window === 'undefined') return DEFAULT_LOCALE;
    try {
        const raw = window.localStorage.getItem(STORAGE_KEY);
        if (raw && (SUPPORTED_LOCALES as string[]).includes(raw)) {
            return raw as SupportedLocale;
        }
    } catch {
        // ignore
    }
    return pickInitialLocale();
}

function format(template: string, params?: Record<string, string | number>): string {
    if (!params) return template;
    return template.replace(/\{(\w+)\}/g, (_, key) =>
        params[key] !== undefined ? String(params[key]) : `{${key}}`,
    );
}

export function I18nProvider({ children }: { children: React.ReactNode }) {
    const [locale, setLocaleState] = useState<SupportedLocale>(() => readStoredLocale());

    useEffect(() => {
        if (typeof window === 'undefined') return;
        try {
            window.localStorage.setItem(STORAGE_KEY, locale);
        } catch {
            // ignore (private mode, etc.)
        }
        if (typeof document !== 'undefined') {
            document.documentElement.setAttribute('lang', locale);
        }
    }, [locale]);

    const setLocale = useCallback((next: SupportedLocale) => {
        setLocaleState(next);
    }, []);

    const t = useCallback(
        (key: string, params?: Record<string, string | number>) => {
            const catalog: MessageCatalog = MESSAGES[locale] ?? MESSAGES[DEFAULT_LOCALE];
            const template =
                catalog[key] ?? MESSAGES[DEFAULT_LOCALE][key] ?? key;
            return format(template, params);
        },
        [locale],
    );

    const value = useMemo<I18nContextValue>(
        () => ({ locale, setLocale, t }),
        [locale, setLocale, t],
    );

    return <I18nContext.Provider value={value}>{children}</I18nContext.Provider>;
}

export function useI18n(): I18nContextValue {
    const ctx = useContext(I18nContext);
    if (!ctx) {
        // Fallback when used outside provider — important for hooks called from
        // legacy code paths. Returns a non-reactive resolver against the default.
        return {
            locale: DEFAULT_LOCALE,
            setLocale: () => undefined,
            t: (key, params) =>
                format(MESSAGES[DEFAULT_LOCALE][key] ?? key, params),
        };
    }
    return ctx;
}

/** Convenience — `const t = useT(); t('key')`. */
export function useT(): I18nContextValue['t'] {
    return useI18n().t;
}
