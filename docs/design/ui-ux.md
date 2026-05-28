# Arche Admin UI — Design Considerations

## Tech Stack

- **Framework:** React + TypeScript
- **Component library:** shadcn/ui (Radix primitives + Tailwind CSS)
- **State / data fetching:** TanStack Query (server state), Zustand (client state)
- **Routing:** React Router (hash-based for simple deployment)
- **Forms:** React Hook Form + Zod (shared validation schemas with the API)
- **API client:** Generated TypeScript client from the JSON Schema endpoints

---

## Layout & Navigation

### A. Navigation Drawer (left side)
- shadcn/ui sidebar (`collapsible="icon"`) — collapses to icons on desktop, slides as overlay on mobile
- **Header:** Client switcher dropdown (super admin only) or read-only client name (scoped keys)
- **Main content:** Nav links: Dashboard, Blueprints, Affixes, Global Meta Attributes, Audit Log, Import, Export, API Keys (in that order, Import/Export hidden for non-super-admin)
- **Footer:** Current API key identity shown (key name or "Super Admin"), clicking opens dropdown with Logout option
- Dark/light mode toggle is in the header only (not in sidebar)

### B. Activity Panel (right side)
- Toggleable slide-out panel
- Default: floating overlay that hides on outside click
- Optional: user can **anchor/pin** it so it docks to the viewport and stays visible
- Content: activity log / audit log entries / change history

### C. Persistent Header
- Logo / app name
- Global Search Bar (Cmd+K)
- Dark/light mode toggle
- Navigation Drawer toggle button
- Activity Panel toggle button

### D. Responsive Design
- Mobile-first layout
- Tables collapse to stacked cards on narrow screens
- Drawers and panels use full screen width on mobile

---

## Pages / Views

### 1. Login
- Single input: enter the super admin API key
- On success, redirect to dashboard
- Key is stored in memory (not localStorage) — lost on tab close

### 2. Dashboard
- Stats cards: total blueprints, affixes, warnings
- "Quick Generate" playground:
  - Archetype dropdown (populated from distinct blueprint archetypes)
  - Constraint builder (dynamic form based on available meta attributes)
  - "Generate" button → preview result inline

### 3. Global Search Bar (persistent in header)
- Search across blueprints + affixes by name
- Results grouped by type, click navigates to detail view
- Keyboard shortcut: `Cmd+K` or `Ctrl+K`

### 4. Blueprints List
- Table with columns: checkbox (multi-select), Name, Archetype, Weight, Updated
- Multi-select: shift-click range select, checkbox toggle per row
- Filter bar: by archetype (dropdown), by name (text search)
- Pagination: offset-based with page numbers + page size selector
- Row actions: Edit (opens modal), Duplicate, Delete, "Test Generate"
- Batch toolbar appears when ≥1 rows selected: Batch Delete, Batch Assign, Batch Edit

### 5. Blueprint Detail
- Read-only page with shareable URL
- Tabbed layout (4 tabs):

**Tab 1 — General:**
- Name, Archetype, Weight, Description (display only)
- "Edit" button → opens Edit Blueprint modal

**Tab 2 — Attributes:**
- Table of attributes: Name, Value Type, Preview, Source (inline vs. global $ref_id)
- Sorted by attribute_order

**Tab 3 — Affixes:**
- Two sections: Prefixes and Suffixes
- Each section: min/max count, affix pool table with Name, Weight

**Tab 4 — History:**
- Filtered audit log for this blueprint
- Shows changes with diffs (before/after JSONs rendered as side-by-side or inline diff)

### 6. Blueprint Create / Edit (modal)
- Opened from Blueprints List (Create button) or Blueprint Detail (Edit button)
- Tabbed or stepped form:

**General:**
- Name (required), Archetype (required), Weight (required, number > 0), Description (optional)

**Attributes:**
- Table of attributes with drag-and-drop reorder (persisted as attribute_order)
- Add attribute: "Inline attribute" form or "From global library" picker
- Delete attribute per row with confirmation

**Affixes:**
- Min/max prefix and suffix counts
- Affix pool table (drag reorder, weight editable)
- "Add affix" button → opens affix picker (searchable, filtered by prefix/suffix type)

### 7. Affixes List
- Table: checkbox (multi-select), Name, Type (prefix/suffix), Attribute Name, Value Type, Updated
- Filter: by type, by name search
- Pagination: same as blueprints
- Row actions: Edit (opens modal), Delete
- Batch toolbar: Batch Delete, Batch Assign to Blueprints

### 8. Affix Detail
- Read-only page with shareable URL
- Name, Type, Description
- Attribute section (inline or global reference)
- History tab
- "Edit" button → opens Edit Affix modal

### 9. Affix Create / Edit (modal)
- Opened from Affixes List or Affix Detail
- Name (required), Type (prefix/suffix toggle), Description (optional)
- Attribute section: inline attribute form or "From global library" picker

### 10. Global Meta Attributes List
- Table: Name, Value Type, Preview, Usage Count (# of blueprints + affixes referencing it)
- Filter: by name, by value type
- Row actions: Edit (opens modal), Delete

### 11. Global Meta Attribute Detail
- Read-only page with shareable URL
- Name, Description, Value type with type-specific payload
- "Referenced by" section: lists blueprints and affixes that use this attribute
- History tab
- "Edit" button → opens Edit modal

### 12. Global Meta Attribute Create / Edit (modal)
- Name (required), Description (optional)
- Value type selector: single / enum / range / string / boolean → dynamic form fields based on type

### 13. Clients Management
- Table: Name, Created, Key Count
- "Create Client" button → modal with name input
- Row actions: View Keys, Delete (danger zone)

### 14. Client Detail
- Name (editable)
- Keys section: table of keys with Name, Permissions, Created, Last Used
- "Create Key" button → modal with name + permission checkboxes → shows raw key once
- "Revoke Key" button per row → confirmation dialog
- Danger zone: "Delete Client" button → confirmation + information about cascading actions

### 15. Audit Log
- Full-page table: Timestamp, Actor, Client, Resource Type, Action, Resource Name
- Filters: by client, by resource type, by action (`created`, `updated`, `deleted`, `force_deleted`, `adjusted`), by date range
- Click row → expand to show before/after diff

### 16. Export
- Table of clients with per-row checkboxes
- Per-client toggles: "Export API keys", "Export audit log" (core tables always exported)
- Global toggle: "Inline global references" — resolves `$ref_id` to inline values in export
- "Export" button → browser downloads ZIP with folder-per-client structure

### 17. Import
- File picker (drag-and-drop zone for ZIP file)
- Parses archive and shows preview: clients found, resource counts per client
- "New client or existing?" prompt for each client name collision
- "Inline or $ref_id?" prompt for global references (if export did not inline)
- "Begin Import" button → conflict resolution phase

### 18. Conflict Resolution (Import modal)
- Progress bar, table of conflicting resources, per-attribute resolution dropdowns
- Per-resource "Apply to all attributes" dropdown
- Top bar: "Apply to all remaining" button
- Completes → summary toast

---

## UI Patterns

### Resource Views Pattern
Every entity (blueprints, affixes, global meta attributes, clients) follows:
- **List page** — table with search/filter, row actions, batch toolbar
- **Detail page** — read-only, shareable URL, tabbed views, "Edit" button
- **Create/Edit modal** — overlay opened from list or detail page

### Delete Protection
- Any delete triggers a modal showing:
  - What is being deleted
  - What references it (if any) with links to each referencing resource
  - A "Force Delete" button (styled red/danger) with additional warning text
  - A "Cancel" button

### Attribute Inline vs. Reference
- Inline attributes: shown with a small "inline" badge
- `$ref_id` attributes: shown with a "global" badge + link to the global definition
- When viewing a blueprint, resolved values are previewed (e.g., referenced rarity shows its enum values inline in a lighter color)

### Diff View
- For audit log history: side-by-side JSON diff using a library like `react-diff-viewer-continued`
- Color-coded: green for additions, red for removals, yellow for changes

### Toast Notifications
- Success: "Blueprint updated"
- Error: "Failed to delete affix — it is referenced by 2 blueprints"
- Force delete: "Affix deleted. Removed from 2 blueprints."

### Dark / Light Mode
- shadcn/ui supports dark mode out of the box
- Toggle in the header (also available in Navigation Drawer)

### Duplicate / Use as Template
- "Use as template" button in blueprint detail view toolbar
- Also available as row action in Blueprints List: "Duplicate"
- Creates a new blueprint with all attributes, affixes, and settings copied (new ID)
- Opens the new blueprint in edit mode with "(Copy)" suffix on name
- User can immediately modify before saving

### Batch Operations
- **Batch Delete (blueprints/affixes):** Select multiple rows → click "Delete" → confirmation dialog listing resources to delete → confirm → toast with count of deleted resources
- **Batch Assign (affixes to blueprints):** Two approaches:
  1. **From affix list:** Select N affixes → click "Assign to blueprints" → opens blueprint picker (checkboxes) → confirm → each affix added with weight 1.0. Duplicate assignments silently skipped.
  2. **Drag-and-drop column layout:** Three-column view (affixes | globals | blueprints). Select/multiselect items in left two columns, drag onto selected blueprints in right column.
- **Batch Edit (blueprint attributes):** Select multiple blueprints → click "Batch Edit" → merged attribute view showing only attributes with matching value types across all selections. Blank fields for differing values.

### Animations
- Page transitions: smooth fade/slide between routes
- Micro-interactions: hover effects, button feedback
- Loading states: animated skeleton loaders during data fetches
- Panel transitions: slide + fade for drawers, modals, activity panel

---

## Future Considerations (not in scope yet)
- Generation preview history (recently generated things)
- Real-time updates via WebSocket when another admin changes data
- Multi-user auth with user accounts / SSO
