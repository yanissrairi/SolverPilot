# UI Component Inventory - Frontend

## Component Organization

The frontend uses a feature-based organization with reusable UI components.

### Layout Components

Located in `src/lib/layout/`

| Component          | Purpose                       | Key Features                             |
| ------------------ | ----------------------------- | ---------------------------------------- |
| **MainLayout**     | 3-panel application layout    | Resizable panels, persistent sizing      |
| **Header**         | Application header/title bar  | Branding, navigation                     |
| **ResizablePanel** | Resizable container component | Drag-to-resize, localStorage persistence |

---

## Feature Components

Located in `src/lib/features/`

### Benchmarks Feature

**Location**: `src/lib/features/benchmarks/`

| Component         | Purpose                            |
| ----------------- | ---------------------------------- |
| **BenchmarkList** | Display and manage benchmark files |

**Functionality**:

- List benchmarks in active project
- Add benchmarks via file picker
- Remove benchmarks
- Queue benchmarks for execution

### Jobs Feature

**Location**: `src/lib/features/jobs/`

| Component      | Purpose                            |
| -------------- | ---------------------------------- |
| **JobMonitor** | Real-time job execution monitoring |

**Functionality**:

- Display running job status
- Show progress (current/total)
- Stream logs in real-time
- Display elapsed time
- Start/stop/kill job controls

### History Feature

**Location**: `src/lib/features/history/`

| Component        | Purpose               |
| ---------------- | --------------------- |
| **HistoryPanel** | Job execution history |

**Functionality**:

- Display completed jobs
- Show job results and logs
- Delete historical jobs
- Filter by status

### Projects Feature

**Location**: `src/lib/features/projects/`

| Component           | Purpose                |
| ------------------- | ---------------------- |
| **ProjectSelector** | Manage Python projects |

**Functionality**:

- List all projects
- Create new projects with Python version selection
- Switch active project
- Delete projects

### Dependencies Feature

**Location**: `src/lib/features/dependencies/`

| Component           | Purpose                      |
| ------------------- | ---------------------------- |
| **DependencyPanel** | Python dependency management |

**Functionality**:

- Analyze benchmark dependencies (tree-sitter)
- Display local vs external dependencies
- Add/remove packages via `uv`
- Sync environment

### SSH Feature

**Location**: `src/lib/features/ssh/`

| Component              | Purpose                  |
| ---------------------- | ------------------------ |
| **SshPassphraseModal** | SSH key passphrase input |

**Functionality**:

- Prompt for SSH key passphrase
- Secure input handling
- Add key to SSH agent

### Setup Feature

**Location**: `src/lib/features/setup/`

| Component       | Purpose                      |
| --------------- | ---------------------------- |
| **SetupWizard** | First-time application setup |

**Functionality**:

- Configure SSH connection
- Set remote paths
- Configure Gurobi settings
- Test connections

---

## Reusable UI Components

Located in `src/lib/ui/`

### Interactive Components

| Component      | Purpose               | Props/Features                       |
| -------------- | --------------------- | ------------------------------------ |
| **Button**     | Primary action button | Variants: primary, secondary, danger |
| **IconButton** | Icon-only button      | Compact, tooltip support             |
| **Modal**      | Dialog overlay        | Backdrop, focus trap, ESC to close   |
| **Select**     | Dropdown selection    | Options array, onChange handler      |
| **Tooltip**    | Contextual help       | Hover trigger, positioning           |

### Display Components

| Component      | Purpose                     | Features                      |
| -------------- | --------------------------- | ----------------------------- |
| **Badge**      | Status indicator            | Color variants, sizes         |
| **EmptyState** | Empty list placeholder      | Icon, message, call-to-action |
| **Spinner**    | Loading indicator           | Animated, size variants       |
| **Skeleton**   | Content loading placeholder | Shimmer effect                |

### Feedback Components

| Component          | Purpose                    | Features                          |
| ------------------ | -------------------------- | --------------------------------- |
| **Toast**          | Notification message       | Auto-dismiss, type variants       |
| **ToastContainer** | Toast notification manager | Stacked notifications, animations |

---

## Component Patterns

### Svelte 5 Runes Usage

All components use modern Svelte 5 patterns:

```typescript
// State
let items = $state<Item[]>([]);

// Computed values
let count = $derived(items.length);

// Effects
$effect(() => {
  // Side effect with automatic cleanup
});

// Props with Snippet support
interface Props {
  title: string;
  children?: Snippet;
}
const { title, children }: Props = $props();
```

### Component Composition

- Feature components compose reusable UI components
- Snippet-based children for flexible content
- Props destructuring with TypeScript interfaces

### Event Handling

- Direct function binding (no custom events)
- Callback props for parent communication
- Keyboard shortcut integration

---

## Styling Approach

**TailwindCSS 4** utility classes throughout:

- Responsive design with breakpoint prefixes
- Dark mode support (where applicable)
- Custom color palette for brand consistency
- Spacing and typography scales

**No Component-Scoped CSS**:

- All styling via Tailwind utilities
- Consistent design system
- Easy theming and customization

---

## Accessibility Features

1. **Keyboard Navigation**:
   - Focus trap in modals
   - Keyboard shortcuts system
   - Tab order management

2. **ARIA Labels**:
   - Descriptive labels on interactive elements
   - Screen reader support
   - Role attributes where needed

3. **Visual Feedback**:
   - Loading states with spinners/skeletons
   - Error states with clear messaging
   - Success feedback via toasts

---

## Summary

**Total Components**: 28+

- **Layout**: 3 components
- **Features**: 7 feature modules with 8 main components
- **UI Library**: 11 reusable components
- **Utilities**: 2 utility modules

**Design Philosophy**:

- Feature-first organization
- Reusable, composable components
- Type-safe props with TypeScript
- Modern Svelte 5 runes-based reactivity
- TailwindCSS for consistent styling
- Keyboard-first UX
