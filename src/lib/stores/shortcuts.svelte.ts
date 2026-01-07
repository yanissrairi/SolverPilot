import { untrack } from 'svelte';
import { matchesShortcut, type ShortcutConfig } from '../utils/keyboard';

export interface Shortcut extends ShortcutConfig {
  action: () => void;
  description: string;
}

const shortcuts = $state<Shortcut[]>([]);
let listenerAttached = false;

export function registerShortcut(shortcut: Shortcut) {
  // Use untrack to prevent effects from tracking this mutation as a dependency
  untrack(() => shortcuts.push(shortcut));
}

export function unregisterShortcut(key: string) {
  // Use untrack to prevent effects from tracking this mutation as a dependency
  untrack(() => {
    const k = key.toLowerCase();
    // Iterate backwards to remove safely
    for (let i = shortcuts.length - 1; i >= 0; i--) {
      if (shortcuts[i].key.toLowerCase() === k) {
        shortcuts.splice(i, 1);
      }
    }
  });
}

function handleKeydown(event: KeyboardEvent) {
  const target = event.target as HTMLElement;
  const isInput =
    target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;

  let handled = false;

  for (const s of shortcuts) {
    if (matchesShortcut(event, s)) {
      // Safety: Don't trigger single-letter shortcuts while typing in inputs
      // Allow if modifiers are used (Ctrl/Alt) or if it's a special key like Escape/Enter
      if (isInput) {
        const hasModifier = event.ctrlKey || event.metaKey || event.altKey;
        const isSpecial = event.key === 'Escape' || event.key === 'Enter';

        if (!hasModifier && !isSpecial) {
          continue;
        }
      }

      s.action();
      handled = true;
    }
  }

  if (handled) {
    event.preventDefault();
  }
}

export function setupGlobalShortcuts() {
  if (listenerAttached) return;
  window.addEventListener('keydown', handleKeydown);
  listenerAttached = true;
}

export function cleanupGlobalShortcuts() {
  if (!listenerAttached) return;
  window.removeEventListener('keydown', handleKeydown);
  listenerAttached = false;
}
