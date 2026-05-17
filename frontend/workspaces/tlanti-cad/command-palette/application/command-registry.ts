/**
 * Global CommandRegistry singleton + a tiny observer so the palette can
 * re-render when features register/unregister actions on mount/unmount.
 */

import type { CommandAction } from '../domain/command-action';

type Listener = () => void;

class CommandRegistryImpl {
    private actions = new Map<string, CommandAction>();
    private listeners = new Set<Listener>();
    private recents: string[] = [];

    register(action: CommandAction): () => void {
        this.actions.set(action.id, action);
        this.notify();
        return () => {
            this.actions.delete(action.id);
            this.notify();
        };
    }

    registerAll(actions: readonly CommandAction[]): () => void {
        const disposers = actions.map((a) => this.register(a));
        return () => {
            for (const dispose of disposers) dispose();
        };
    }

    list(): CommandAction[] {
        return Array.from(this.actions.values());
    }

    recent(limit = 8): CommandAction[] {
        const out: CommandAction[] = [];
        for (const id of this.recents) {
            const action = this.actions.get(id);
            if (action) out.push(action);
            if (out.length >= limit) break;
        }
        return out;
    }

    markUsed(actionId: string): void {
        this.recents = [actionId, ...this.recents.filter((id) => id !== actionId)].slice(0, 20);
    }

    subscribe(listener: Listener): () => void {
        this.listeners.add(listener);
        return () => {
            this.listeners.delete(listener);
        };
    }

    private notify(): void {
        for (const listener of this.listeners) listener();
    }
}

/** Module-scoped singleton — all features share the same registry. */
export const commandRegistry = new CommandRegistryImpl();
