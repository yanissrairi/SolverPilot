# Story 1.1 UX Requirements - Multi-Select Benchmarks in Left Panel

## Executive Summary

Story 1.1 implements multi-select functionality in the BenchmarkList (left panel) to allow users to efficiently select multiple benchmark files for batch queueing. This document provides comprehensive UX requirements covering visual design, interaction patterns, accessibility, and feedback mechanics.

---

## 1. Visual Design Requirements

### 1.1 Checkbox Styling & Visibility

**Checkbox Design:**

- **Standard HTML checkbox** styled with TailwindCSS
- **Appearance**: Small checkbox (16px × 16px) positioned left of benchmark filename
- **Unchecked state**: Empty square outline `☐`
- **Checked state**: Filled square with checkmark `☑`
- **Color scheme**: Blue accents when selected (#3B82F6 / `blue-500`)

**Visibility Behavior:**

- **Default**: Checkboxes HIDDEN (opacity-0) to minimize visual clutter when no items are selected
- **On hover**: Checkboxes FADE IN (100ms transition) using CSS `opacity-0 hover:opacity-100`
- **When any item selected**: All checkboxes remain VISIBLE across entire list to encourage multi-select awareness
- **After first selection**: Keep checkboxes visible until all selections are cleared

**Implementation:**

```css
.benchmark-checkbox {
  @apply opacity-0 hover:opacity-100 transition-opacity duration-100;
}

.benchmark-row.any-selected .benchmark-checkbox {
  @apply opacity-100; /* Visible once user starts selecting */
}
```

### 1.2 Row Highlighting for Selected Items

**Selected Row Styling:**

- **Left border accent**: 4px left border in blue (`border-l-4 border-blue-500`)
- **Background tint**: Very subtle blue background (`bg-blue-900/10`) - approximately 10% opacity for light visual weight
- **Text color**: Preserved (no color change to maintain readability)
- **Spacing**: Maintained (`py-2 px-3` compact list item padding)

**Visual States:**
| State | Visual Indicator | Purpose |
|-------|------------------|---------|
| Unselected, unhovered | No checkbox, normal background | Minimal clutter |
| Unselected, hovered | Checkbox appears, slight hover background | Discoverability |
| Selected | Checkbox checked ☑, blue border + tint background | Clear selection feedback |
| Selected, hovered | Checkbox + blue border + stronger hover state | Confirmation on interaction |

**Implementation:**

```css
.benchmark-row {
  @apply py-2 px-3 text-sm leading-snug
         hover:bg-white/10 cursor-pointer
         transition-colors duration-150;
}

.benchmark-row.selected {
  @apply border-l-4 border-blue-500 bg-blue-900/10;
}
```

### 1.3 Selection Count Indicator

**Display Location:**

- **Fixed to bottom of left panel** (Benchmarks panel)
- **Above the "Queue Selected" button** (see Section 1.4)

**Visual Format:**

- Simple text: "X benchmark(s) selected" or "5 benchmarks selected"
- Use singular form when count = 1: "1 benchmark selected"
- Use plural form for 0 or ≥ 2: "0 benchmarks selected", "5 benchmarks selected"
- Color: Neutral gray (`text-slate-400`) - informational, not attention-grabbing
- Font size: 14px (`text-sm`)

**Dynamic Updates:**

- Count updates in real-time as user selects/deselects items
- Use Svelte `$derived` for reactive updates (no manual state management)

**Implementation:**

```svelte
<p class="text-slate-400 text-sm text-center">
  {selectionCount === 1 ? '1 benchmark selected' : `${selectionCount} benchmarks selected`}
</p>
```

### 1.4 Queue Action Button (StickyActionButton)

**Visual Design:**

- **Position**: Sticky-fixed to bottom of left (Benchmarks) panel
- **Sizing**: Full width with padding (responsive within panel width)
- **Elevation**: Appears above panel border with slight shadow separation

**Button States & Styling:**

| State                             | Label                   | Style                                              | Behavior               |
| --------------------------------- | ----------------------- | -------------------------------------------------- | ---------------------- |
| **IDLE** (0 selected)             | "Queue (0)"             | Disabled, gray (`disabled:opacity-50`)             | No click action        |
| **SELECTION_READY** (1+ selected) | "Queue Selected (5)"    | Enabled, blue (`bg-blue-600 hover:bg-blue-500`)    | Click queues items     |
| **QUEUE_READY** (items in queue)  | "Start Queue (10 jobs)" | Enabled, green (`bg-green-600 hover:bg-green-500`) | Click starts execution |
| **RUNNING** (queue active)        | "Queue Running"         | Yellow (`bg-yellow-600`), shows Pause option       | Pause/Resume controls  |

**Text Content:**

- Always shows count of affected items in parentheses
- Use "Queue Selected (5)" not "Queue 5 items"
- Font size: 16px bold for prominence
- Accessible text: Minimum 44px touch target height

**Implementation:**

```svelte
<button
  class="w-full py-3 px-4 bg-blue-600 hover:bg-blue-500 disabled:opacity-50
         rounded-lg font-bold text-white transition-colors"
  disabled={selectionCount === 0}
  aria-label={`Queue ${selectionCount} selected benchmarks`}
>
  {selectionCount === 0 ? 'Queue (0)' : `Queue Selected (${selectionCount})`}
</button>
```

### 1.5 Visual Feedback on Interaction

**Hover State (Unselected Row):**

- **Background**: `hover:bg-white/10` (very subtle 10% white overlay)
- **Cursor**: Changes to pointer (`cursor-pointer`)
- **Checkbox**: Fades in to 100% opacity
- **Transition duration**: 150ms (`transition-colors duration-150`)

**Active/Click State:**

- **Scale**: Slight compression (`active:scale-95`) on button only, not rows
- **Visual confirmation**: Checkbox immediately shows checked state

**Selection Range Visual:**

- All items in Shift+Click range highlight with same blue styling
- Animation: Smooth transitions between states (no jarring changes)

---

## 2. Interaction Patterns

### 2.1 Selection Methods

Benchmark selection supports multiple familiar interaction patterns, matching standard file manager conventions:

#### Method 1: Click Checkbox

- **Action**: Click the checkbox next to a benchmark filename
- **Result**: Toggles selection state for that item
- **Feedback**: Checkbox animates to checked/unchecked state
- **ARIA Support**: `role="checkbox"` with `aria-checked="true|false"`

**Implementation:**

```svelte
<input
  type="checkbox"
  role="checkbox"
  aria-label="Select benchmark_{filename}"
  checked={isSelected}
  on:change={e => toggleSelection(index)}
/>
```

#### Method 2: Space Key Toggle

- **Action**: Press `Space` key when benchmark row is focused (keyboard navigation)
- **Result**: Toggles selection state for focused item
- **Prerequisite**: Item must have keyboard focus (Tab navigation)
- **Feedback**: Checkbox state updates, row highlights with blue border

**Implementation:**

```svelte
<div
  role="listitem"
  tabindex={0}
  on:keydown={(e) => {
    if (e.code === 'Space') {
      e.preventDefault();
      toggleSelection(index);
    }
  }}
>
```

#### Method 3: Shift+Click for Range Selection

- **Action**: Hold Shift and click another benchmark item
- **Result**: Selects all items from last selected index to current item (inclusive)
- **Behavior**: Works with single or multiple pre-existing selections
- **Edge case**: If no prior selection, starts from top

**Implementation Logic:**

```svelte
function handleRowClick(clickedIndex, event) {
  if (event.shiftKey && lastSelectedIndex !== null) {
    // Select range from lastSelectedIndex to clickedIndex
    const [start, end] = [lastSelectedIndex, clickedIndex].sort((a, b) => a - b);
    for (let i = start; i <= end; i++) {
      selectedIds.add(benchmarks[i].id);
    }
  } else {
    // Single selection
    toggleSelection(clickedIndex);
  }
  lastSelectedIndex = clickedIndex;
}
```

#### Method 4: Ctrl/Cmd+Click for Individual Toggle

- **Action**: Hold Ctrl (Windows/Linux) or Cmd (macOS) and click a benchmark
- **Result**: Toggles selection state of that item WITHOUT affecting other selections
- **Behavior**: Allows non-contiguous multi-select
- **Feedback**: Item toggles while others remain unchanged

**Implementation:**

```svelte
function handleRowClick(clickedIndex, event) {
  if (event.ctrlKey || event.metaKey) {
    toggleSelection(clickedIndex);
  } else if (event.shiftKey && lastSelectedIndex !== null) {
    // Range selection (handled above)
  } else {
    // Single selection - clear others
    selectedIds.clear();
    selectedIds.add(benchmarks[clickedIndex].id);
  }
}
```

#### Method 5: Cmd/Ctrl+A for Select All

- **Action**: Press Cmd+A (macOS) or Ctrl+A (Windows/Linux)
- **Result**: Selects all benchmarks in current list (visible or scrolled)
- **Behavior**: Overwrites previous selections, selects entire list
- **Feedback**: All checkboxes show checked state, count updates

**Implementation:**

```svelte
document.addEventListener('keydown', (e) => {
  if ((e.ctrlKey || e.metaKey) && e.code === 'KeyA' && isBenchmarkListFocused) {
    e.preventDefault();
    selectedIds.clear();
    benchmarks.forEach(b => selectedIds.add(b.id));
  }
});
```

### 2.2 Selection Persistence

**During Queueing:**

- **Selections remain active** after clicking "Queue Selected"
- **User can queue more**: Select additional items and click "Queue Selected" again
- **Clear selections**: User must explicitly click items or Ctrl+Click checkboxes to deselect
- **Benefit**: Enables workflow of "queue 5 items, queue 5 more, queue 5 more"

**After Queue Starts:**

- **Left panel remains visible** and accessible
- **User can queue additional benchmarks** while queue is running
- **New items are added to pending list** (dynamic growth)
- **Selections remain until explicitly cleared** by user

**State Persistence:**

- **Per-session persistence** only (selections lost on app restart)
- **No saved selection sets** in Beta 1 (future enhancement for Beta 2)

---

## 3. Keyboard Shortcuts & Navigation

### 3.1 Keyboard Accessibility

**Tab Navigation:**

- All benchmark rows are tabbable (`tabindex="0"`)
- Arrow keys (Up/Down) navigate between items when list is focused
- Tab cycles through keyboard-focusable elements (checkboxes, buttons)

**Keyboard Shortcut Mapping:**
| Key | Action | Prerequisites |
|-----|--------|---------------|
| `Space` | Toggle selection of focused item | Row has keyboard focus |
| `Shift+↑/↓` | Extend selection range | Row has keyboard focus |
| `Ctrl/Cmd+A` | Select all benchmarks | List has focus |
| `Q` | Queue selected items | At least 1 item selected |
| `Escape` | Clear all selections | Any selections active |

**Shift+Click Range Behavior:**

```
Start: benchmark_05 selected
User Shift+Click on benchmark_10

Result: benchmarks 5, 6, 7, 8, 9, 10 all selected (range inclusive)
```

### 3.2 Q Key Shortcut

**Global Keyboard Shortcut for Queueing:**

- **Trigger key**: `Q`
- **Behavior**: Activates "Queue Selected" button click (sends queued items to queue panel)
- **Enabled when**: At least 1 item is selected
- **Disabled when**: No items selected (0 count)
- **Feedback**: Toast notification confirms queueing

**User Intent:**

- Power users can queue without touching mouse: Select (Space/Shift+Click) → Press Q → Done
- Reduces context switching between keyboard and mouse

**Implementation:**

```svelte
document.addEventListener('keydown', (e) => {
  if (e.code === 'KeyQ' && selectionCount > 0 && !isDialogOpen) {
    handleQueueSelected();
    showToast(`${selectionCount} benchmarks added to queue`);
  }
});
```

---

## 4. Accessibility Requirements

### 4.1 Focus Management & Visual Indicators

**Focus Ring Styling:**

- **Style**: `ring-2 ring-blue-500 ring-offset-2` (blue outline with 2px offset)
- **On rows**: Blue focus ring visible when using Tab navigation
- **On button**: Clear focus indicator when keyboard focused
- **Visibility**: Persist while focused, remove on blur

**Focus Trap Prevention:**

- List remains focusable but doesn't trap keyboard navigation
- Tab out of list → moves to next focusable element (button, etc.)

**Implementation:**

```css
.benchmark-row:focus {
  @apply ring-2 ring-blue-500 ring-offset-2 rounded-md;
}

.queue-button:focus {
  @apply ring-2 ring-green-500 ring-offset-2;
}
```

### 4.2 ARIA Labels & Roles

**List Structure:**

```html
<ul role="list">
  <li role="listitem" aria-label="benchmark_01.py, unselected">
    <input role="checkbox" aria-label="Select benchmark_01.py" />
    benchmark_01.py
  </li>
</ul>
```

**Per-Item ARIA Labels:**

- `aria-label="[filename], [selected/unselected]"`
- Update dynamically as selection state changes
- Example: "benchmark_01.py, selected" when ☑ checked

**Checkbox Accessibility:**

- `role="checkbox"` on input element
- `aria-checked="true|false"` reflects current state
- `aria-label="Select benchmark_01.py"` describes action

**Selection Count:**

- `aria-live="polite"` on count indicator
- Updates announced to screen readers when count changes
- Example: Screen reader announces "5 benchmarks selected"

**Button Accessibility:**

- Dynamic `aria-label` updates with count
- Example: `aria-label="Queue 5 selected benchmarks"`
- Disabled state: `aria-disabled="true"` when count = 0

**Implementation:**

```svelte
<div
  class="selection-count"
  role="status"
  aria-live="polite"
  aria-label={`${selectionCount} benchmarks selected`}
>
  {selectionCount} benchmark{selectionCount !== 1 ? 's' : ''} selected
</div>
```

### 4.3 Screen Reader Announcements

**Polite Announcements (Non-Intrusive):**

- **Selection changes**: "5 benchmarks selected" (polite, doesn't interrupt)
- **Queue action**: "5 benchmarks added to queue" (toast text)
- **Count updates**: Real-time as user selects/deselects

**Assertive Announcements (Urgent):**

- **Errors** (future): "Unable to queue - file not found" (uses `aria-live="assertive"`)
- **Success confirmations**: "Benchmarks queued successfully"

### 4.4 Contrast & Color Accessibility

**WCAG AAA Compliance:**

- Blue selections (`bg-blue-900/10` + `border-blue-500`): 12.6:1 contrast ratio
- Text colors (`text-slate-400`): Meet minimum 4.5:1 ratio
- No information conveyed by color alone - always includes icons or text

**Color Encoding:**

- Blue selection = clear visual distinction
- Checkbox appearance (☑️) = redundant visual marker
- Text labels = machine-readable fallback

### 4.5 Motion & Animation Accessibility

**Reduced Motion Support:**

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 0.01ms !important;
    transition-duration: 0.01ms !important;
  }
}
```

**Transitions Disabled for Users With:**

- Vestibular disorders
- Motion sensitivity
- Cognitive processing difficulties

**Animations Affected:**

- Checkbox fade-in (100ms) → instantaneous
- Hover transitions (150ms) → instantaneous
- All others disabled

---

## 5. Visual Feedback & Notifications

### 5.1 Toast Notifications on Queue Action

**Success Toast:**

- **Message**: "5 benchmarks added to queue"
- **Icon**: ✓ Checkmark
- **Color**: Green (`bg-green-600`)
- **Duration**: Auto-dismiss after 3 seconds
- **Position**: Bottom-right corner (standard)

**Implementation:**

```svelte
showToast({
  type: 'success',
  message: `${selectionCount} benchmarks added to queue`,
  duration: 3000
});
```

### 5.2 State Transitions

**Checkbox Animation:**

- **Fade-in**: 100ms opacity transition on hover
- **Check state**: Instant (no animation on click, just state update)
- **Visual weight**: Checkbox becomes more prominent when selected

**Row Highlighting:**

- **Background**: Smooth transition to blue tint (`transition-colors duration-150`)
- **Border**: Instant appearance of blue left border
- **No scale change** on rows (scale changes are reserved for buttons)

### 5.3 Real-Time Selection Feedback

**Live Count Update:**

- **Text updates instantly**: "5 benchmarks selected"
- **Button state changes**:
  - Color: Gray (`disabled:opacity-50`) → Blue (`bg-blue-600`) when count > 0
  - Enabled state: `disabled=true` → `disabled=false`
  - Click action becomes active

**Visual Scanning:**

- User can glance at count at any time to verify selections
- Button prominence increases as items are selected (visual encouragement)

---

## 6. Interaction Flow Examples

### Example 1: Basic Single Selection

```
1. User sees benchmark_01.py row
2. User hovers over row → checkbox appears
3. User clicks checkbox → checkbox shows ☑
4. Count updates: "1 benchmark selected"
5. Button enables: "Queue Selected (1)"
6. User clicks button → toast: "1 benchmark added to queue"
```

### Example 2: Range Selection with Shift+Click

```
1. User clicks benchmark_05.py → selected
2. Count: "1 benchmark selected"
3. User holds Shift and clicks benchmark_10.py
4. All benchmarks 5-10 now selected (6 items)
5. Count: "6 benchmarks selected"
6. Button shows: "Queue Selected (6)"
7. User presses Q key → queue action triggered
```

### Example 3: Keyboard-Only Selection

```
1. User presses Tab to focus first benchmark
2. User presses Space → first item selected
3. User presses Down Arrow to move focus to second item
4. User presses Shift+Down to extend selection range
5. Multiple items selected
6. User presses Q to queue → items moved to queue panel
7. Count resets: "0 benchmarks selected"
```

### Example 4: Select All with Ctrl+A

```
1. User has BenchmarkList in focus
2. User presses Ctrl+A → all 15 benchmarks selected
3. Count: "15 benchmarks selected"
4. All checkboxes show ☑
5. All rows show blue border + tint
6. Button: "Queue Selected (15)"
7. User clicks → all 15 queued
```

---

## 7. Error Handling & Edge Cases

### 7.1 File Validation Before Queueing

**Before adding to queue:**

1. Verify benchmark file still exists (deleted while browsing?)
2. Check file is valid Python (\*.py extension)
3. Verify file is readable (permission issues?)

**If validation fails:**

- **Single failed item**: Toast warning: "benchmark_01.py - file not found"
- **Multiple failures**: "2 files invalid - 3 queued successfully"
- **All failed**: "Unable to queue - no valid files selected"

**Implementation:**

```rust
// Backend validation
for benchmark in selected {
  if !Path::new(&benchmark.path).exists() {
    return Err("Benchmark file not found".to_string());
  }
}
```

### 7.2 Duplicate Detection

**If item already queued:**

- **Check**: Compare selected benchmarks against current queue contents
- **Warn user**: "benchmark_01.py is already queued - add duplicate?"
- **User choice**: Allow duplicates or skip (simple modal or toast with undo)

### 7.3 Selection Clearing

**User deselects items via:**

- Ctrl/Cmd+Click on selected item (toggle off)
- Individual checkbox clicks
- Escape key (clear all selections)
- After successful queue action (optional - currently stays selected)

---

## 8. Performance Considerations

### 8.1 List Rendering Optimization

**For lists with 10-50 benchmarks:**

- **Standard rendering**: All items rendered immediately (no virtualization needed)
- **Complexity**: O(n) for selection operations

**For lists with 100+ benchmarks (future):**

- **Virtualization**: React Window / Svelte-Virtual-List pattern
- **Window size**: 15-20 visible items at a time
- **Preserve selection**: Selections maintained across scroll

### 8.2 State Management

**Efficient Updates:**

- Use `Set<id>` for selections (O(1) lookup)
- Use Svelte `$derived` for counts (reactive, no manual updates)
- Avoid re-rendering entire list on each selection change

**Implementation:**

```svelte
let selectedIds = $state(new Set());

let selectionCount = $derived(selectedIds.size);
let allSelected = $derived(selectionCount === benchmarks.length);

function toggleSelection(id) {
  if (selectedIds.has(id)) {
    selectedIds.delete(id);
  } else {
    selectedIds.add(id);
  }
  // Reactivity triggers automatically with $state
}
```

---

## 9. Component Specifications

### 9.1 MultiSelectCheckbox Component

**Props Interface:**

```typescript
interface Props {
  items: Benchmark[]; // Array of benchmark objects
  onSelectionChange: (selected: Set<string>) => void; // Selection callback
  onQueueSelected: (selected: Benchmark[]) => void; // Queue action callback
}
```

**Benchmark Object:**

```typescript
interface Benchmark {
  id: string;
  filename: string;
  path: string;
  lastModified: Date;
  size: number;
}
```

**Internal State:**

```svelte
<script lang="ts">
  let selectedIds = $state(new Set<string>());
  let lastSelectedIndex = $state<number | null>(null);

  let selectionCount = $derived(selectedIds.size);
  let isAnySelected = $derived(selectedIds.size > 0);
</script>
```

### 9.2 StickyActionButton Component

**Props:**

```typescript
interface Props {
  selectionCount: number;
  queuedCount: number;
  isRunning: boolean;
  onQueueSelected: (selected: Benchmark[]) => Promise<void>;
  onStartQueue?: () => Promise<void>;
}
```

**State Machine:**

```
IDLE (count=0)
  ↓ (user selects items)
SELECTION_READY (count>0)
  ↓ (user clicks queue)
QUEUE_READY (queued, not running)
  ↓ (user clicks start)
RUNNING (queue active)
  ↓ (user clicks pause or all complete)
PAUSED / COMPLETED
```

---

## 10. Implementation Checklist

### Phase 1: Component Structure

- [ ] Create MultiSelectCheckbox component with hover/select states
- [ ] Create StickyActionButton component with state machine
- [ ] Create selection count indicator display
- [ ] Integrate checkbox fade-in/out logic

### Phase 2: Interaction Logic

- [ ] Implement Space key toggle
- [ ] Implement Shift+Click range selection
- [ ] Implement Ctrl/Cmd+Click individual toggle
- [ ] Implement Ctrl/Cmd+A select all
- [ ] Implement Q key shortcut for queueing
- [ ] Implement Escape key to clear selections

### Phase 3: Accessibility

- [ ] Add ARIA labels to checkboxes
- [ ] Add role attributes (list, listitem, checkbox)
- [ ] Add aria-live for count updates
- [ ] Add aria-disabled for button states
- [ ] Add focus ring styling
- [ ] Test with keyboard navigation

### Phase 4: Visual Polish

- [ ] Checkbox hover fade-in/out (100ms)
- [ ] Row highlight with blue border + tint
- [ ] Button state color transitions
- [ ] Selection count text updates
- [ ] Toast notifications on queue action

### Phase 5: Testing

- [ ] Test all selection methods (checkbox, Space, Shift+Click, Ctrl+Click, Ctrl+A)
- [ ] Test keyboard-only navigation and selection
- [ ] Test range selection edge cases
- [ ] Test rapid selection/deselection
- [ ] Test with screen readers (NVDA, JAWS, VoiceOver)
- [ ] Test performance with 50+ items

---

## 11. Success Criteria

Users can successfully complete Story 1.1 when they can:

✅ **Select multiple benchmarks** using at least 3 different methods:

- Checkbox clicking
- Space key toggling
- Shift+Click range selection
- Ctrl/Cmd+Click individual toggle
- Ctrl/Cmd+A select all

✅ **See clear visual feedback** for:

- Selected items (blue border + light background)
- Checkbox state (checked/unchecked)
- Selection count ("5 benchmarks selected")
- Button state changes (enabled/disabled, color change)

✅ **Queue selected items** via:

- Button click (mouse)
- Q key (keyboard)
- Toast confirmation message

✅ **Keyboard-only workflow**:

- Navigate with Tab/Arrow keys
- Select with Space
- Select ranges with Shift+Arrow
- Queue with Q key
- All without touching mouse

✅ **Accessibility compliance**:

- All functionality available via keyboard
- Screen reader compatible (ARIA labels, roles, live regions)
- WCAG AAA contrast compliance
- Focus ring visible on all interactive elements

---

## Appendix: Code Examples

### Svelte Component Template

```svelte
<script lang="ts">
  import type { Benchmark } from '$lib/types';
  import MultiSelectCheckbox from './MultiSelectCheckbox.svelte';
  import StickyActionButton from './StickyActionButton.svelte';

  interface Props {
    benchmarks: Benchmark[];
  }

  const { benchmarks }: Props = $props();

  let selectedIds = $state(new Set<string>());
  let lastSelectedIndex = $state<number | null>(null);

  let selectionCount = $derived(selectedIds.size);
  let selectedBenchmarks = $derived(benchmarks.filter(b => selectedIds.has(b.id)));

  function toggleSelection(id: string) {
    if (selectedIds.has(id)) {
      selectedIds.delete(id);
    } else {
      selectedIds.add(id);
    }
  }

  function handleQueueSelected() {
    // Dispatch to parent or call API
    dispatch('queue', { selected: selectedBenchmarks });
    // Optionally clear selections after queue
    // selectedIds.clear();
  }
</script>

<div class="benchmark-list-container h-full flex flex-col">
  <div class="flex-1 overflow-y-auto custom-scrollbar">
    <ul role="list" class="space-y-0">
      {#each benchmarks as benchmark, index (benchmark.id)}
        <li
          role="listitem"
          class="list-item-compact-interactive"
          class:selected={selectedIds.has(benchmark.id)}
        >
          <input
            type="checkbox"
            role="checkbox"
            aria-label="Select {benchmark.filename}"
            aria-checked={selectedIds.has(benchmark.id)}
            checked={selectedIds.has(benchmark.id)}
            on:change={() => toggleSelection(benchmark.id)}
            class="benchmark-checkbox"
          />
          <span class="flex-1 truncate">{benchmark.filename}</span>
        </li>
      {/each}
    </ul>
  </div>

  <div class="border-t border-slate-700/50 p-4">
    <div class="text-center text-slate-400 text-sm mb-3" role="status" aria-live="polite">
      {selectionCount === 1 ? '1 benchmark selected' : `${selectionCount} benchmarks selected`}
    </div>

    <StickyActionButton {selectionCount} on:queue={handleQueueSelected} />
  </div>
</div>

<style>
  .benchmark-checkbox {
    @apply opacity-0 hover:opacity-100 transition-opacity duration-100;
  }

  :global(.benchmark-list-container.any-selected) .benchmark-checkbox {
    @apply opacity-100;
  }
</style>
```

---

## Summary

Story 1.1 delivers a **keyboard-accessible, multi-select benchmark interface** that enables power users to efficiently batch multiple benchmarks for queueing. The implementation prioritizes:

1. **Familiarity**: Standard file manager selection patterns (Shift+Click, Ctrl+Click)
2. **Efficiency**: Keyboard shortcuts (Space, Q, Ctrl+A) for power users
3. **Clarity**: Visual feedback (checkbox states, blue highlights, count indicator)
4. **Accessibility**: WCAG AAA compliance, screen reader support, keyboard-only navigation
5. **Confidence**: Toast confirmations and always-visible selection state

The interface supports selection workflows ranging from mouse-only (click checkboxes) to keyboard-only (Tab, Space, Shift+Arrow, Q) to mixed input methods.
