# State Management Patterns - Frontend

## Overview

SolverPilot uses **Svelte 5 Runes** for state management, replacing the legacy stores system with a more modern, reactive approach.

---

## State Management Strategy

### 1. Local Component State

**Pattern**: `$state` runes for component-specific data

Components manage their own local state using the `$state` rune:

```typescript
let items = $state<Item[]>([]);
let selectedId = $state<number | null>(null);
let isLoading = $state(false);
```

**Use Cases**:

- Form input values
- UI toggle states (open/closed, expanded/collapsed)
- Component-specific loading states
- Temporary data

---

### 2. Derived Values

**Pattern**: `$derived` for computed values

Computed values are automatically recalculated when dependencies change:

```typescript
let items = $state<Item[]>([]);
let count = $derived(items.length);
let hasItems = $derived(items.length > 0);
let filteredItems = $derived(items.filter(i => i.active));
```

**Use Cases**:

- Counts and aggregations
- Filtered/sorted lists
- Conditional flags
- Formatted values

---

### 3. Side Effects

**Pattern**: `$effect` for reactions

Effects run when their dependencies change, with automatic cleanup:

```typescript
$effect(() => {
  console.log(`Count changed to: ${count}`);
  // Cleanup happens automatically when effect re-runs or component unmounts
});
```

**Use Cases**:

- Logging and debugging
- External API calls
- DOM manipulation
- Event listener setup

---

## Global State Stores

The application uses three global stores for cross-component state:

### 1. Panel Store

**Location**: `src/lib/stores/panels.svelte.ts`

**Purpose**: Manage resizable panel widths with localStorage persistence

**Implementation**:

```typescript
class PanelStore {
  leftWidth = $state(loadFromStorage('panel-left-width', 280));
  rightWidth = $state(loadFromStorage('panel-right-width', 300));

  setLeftWidth(width: number) {
    this.leftWidth = width;
    saveToStorage('panel-left-width', width);
  }

  setRightWidth(width: number) {
    this.rightWidth = width;
    saveToStorage('panel-right-width', width);
  }
}
```

**Features**:

- Reactive `$state` runes
- Automatic localStorage sync
- Default values
- Getter/setter methods

**Usage**:

```typescript
import { panelStore } from '$lib/stores/panels.svelte.ts';

// Read width
const leftWidth = panelStore.leftWidth;

// Update width (persists to localStorage)
panelStore.setLeftWidth(320);
```

---

### 2. Keyboard Shortcuts Store

**Location**: `src/lib/stores/shortcuts.svelte.ts`

**Purpose**: Global keyboard shortcut registration and handling

**Implementation**:

```typescript
interface Shortcut {
  key: string;
  ctrl?: boolean;
  alt?: boolean;
  shift?: boolean;
  action: () => void;
  description: string;
}

const shortcuts = $state<Shortcut[]>([]);

export function registerShortcut(shortcut: Shortcut) {
  untrack(() => shortcuts.push(shortcut));
}

export function unregisterShortcut(key: string) {
  untrack(() => {
    // Remove matching shortcuts
  });
}
```

**Features**:

- Component-scoped shortcut registration
- Automatic cleanup on component unmount
- Input/textarea safety (respects typing context)
- Modifier key support (Ctrl, Alt, Shift)
- Global keydown listener

**Usage**:

```typescript
import { registerShortcut, unregisterShortcut } from '$lib/stores/shortcuts.svelte.ts';

$effect(() => {
  registerShortcut({
    key: 'n',
    ctrl: true,
    action: () => createNew(),
    description: 'Create new item',
  });

  return () => unregisterShortcut('n');
});
```

---

### 3. Toast Store

**Location**: `src/lib/stores/toast.svelte.ts`

**Purpose**: Global notification system for user feedback

**Implementation**:

```typescript
export type ToastType = 'success' | 'error' | 'info' | 'warning';

interface Toast {
  id: string;
  type: ToastType;
  message: string;
  duration?: number;
}

class ToastStore {
  toasts = $state<Toast[]>([]);

  add(type: ToastType, message: string, duration = 5000) {
    const id = crypto.randomUUID();
    this.toasts.push({ id, type, message, duration });

    if (duration > 0) {
      setTimeout(() => this.remove(id), duration);
    }
  }

  remove(id: string) {
    this.toasts = this.toasts.filter(t => t.id !== id);
  }

  success(message: string) {
    this.add('success', message);
  }
  error(message: string) {
    this.add('error', message);
  }
  info(message: string) {
    this.add('info', message);
  }
  warning(message: string) {
    this.add('warning', message);
  }
}

export const toast = new ToastStore();
```

**Features**:

- Auto-dismiss with configurable duration
- Type variants (success, error, info, warning)
- Unique IDs via crypto.randomUUID()
- Convenience methods for each type
- Stacked notifications

**Usage**:

```typescript
import { toast } from '$lib/stores/toast.svelte.ts';

// Show success message
toast.success('Project created successfully!');

// Show error (stays longer)
toast.error('Failed to connect to server', 8000);

// Show info
toast.info('Sync in progress...');
```

---

## Backend State Synchronization

### Polling Pattern

For real-time updates, the frontend polls backend state:

```typescript
let status = $state<JobStatusResponse | null>(null);

$effect(() => {
  const interval = setInterval(async () => {
    status = await getJobStatus();
  }, 2000);

  return () => clearInterval(interval);
});
```

**Use Cases**:

- Job execution monitoring
- SSH connection status
- Sync status checking

### Request-Response Pattern

For user-initiated actions:

```typescript
async function handleSubmit() {
  isLoading = true;
  try {
    const result = await saveConfig(config);
    toast.success('Configuration saved!');
  } catch (error) {
    toast.error(`Failed: ${error}`);
  } finally {
    isLoading = false;
  }
}
```

---

## State Flow Architecture

```
┌─────────────────────────────────────────┐
│        Component Local State            │
│    ($state, $derived, $effect)          │
└────────────┬────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────┐
│         Global Stores (3)               │
│  - Panel sizes (panelStore)             │
│  - Keyboard shortcuts (shortcuts)       │
│  - Notifications (toast)                │
└────────────┬────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────┐
│         API Layer (api.ts)              │
│    Tauri invoke() wrapper functions     │
└────────────┬────────────────────────────┘
             │
             ↓
┌─────────────────────────────────────────┐
│      Backend State (Rust)               │
│   AppState with Arc<Mutex<T>>           │
└─────────────────────────────────────────┘
```

---

## Best Practices

### 1. Use `untrack()` for Store Mutations

Prevent effects from tracking store mutations:

```typescript
untrack(() => shortcuts.push(newShortcut));
```

### 2. Return Cleanup Functions from Effects

Always clean up side effects:

```typescript
$effect(() => {
  const interval = setInterval(poll, 1000);
  return () => clearInterval(interval);
});
```

### 3. Keep Local State Local

Don't lift state to global stores unless multiple components need it.

### 4. Use Derived for Computations

Don't duplicate computed logic; use `$derived`:

```typescript
// Good
let count = $derived(items.length);

// Bad
let count = $state(0);
$effect(() => {
  count = items.length;
});
```

### 5. Batch Backend Calls

Avoid excessive polling; batch related requests:

```typescript
// Good - single call with all data
const status = await getJobStatus();

// Bad - multiple calls for related data
const job = await getJob();
const logs = await getLogs();
const progress = await getProgress();
```

---

## Migration from Legacy Stores

**Svelte 5 Runes replace**:

- `writable()` → `$state()`
- `readable()` → `$derived()`
- `derived()` → `$derived()`
- `$:` reactive statements → `$derived()` or `$effect()`
- `onMount()/onDestroy()` → `$effect()` with cleanup

**Benefits**:

- Better TypeScript integration
- Simpler syntax
- Automatic cleanup
- More predictable reactivity
- Smaller bundle size

---

## Summary

**State Management Layers**:

1. **Component State**: `$state` runes for local data
2. **Global Stores**: 3 stores for cross-component concerns
3. **Backend Sync**: Polling + request-response patterns
4. **Persistence**: localStorage for UI preferences

**Key Patterns**:

- Runes-based reactivity (`$state`, `$derived`, `$effect`)
- Class-based stores for global state
- `untrack()` for non-reactive mutations
- Cleanup functions in `$effect()`
- Toast notifications for user feedback
- Keyboard shortcuts with auto-cleanup
