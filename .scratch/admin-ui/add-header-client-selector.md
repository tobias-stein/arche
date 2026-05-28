---
title: Add header client selector and client-scoped UI refinements
status: completed
---

## What to build

Several frontend UI refinements for the client-scoped login experience:

**1. Header client selector**
When logged in as a **super admin**: Add a client dropdown in the header (between "Arche Admin" title and the search bar) showing the currently selected client name. This duplicates the client switcher that currently lives only in the navigation drawer, making it always visible.

When logged in as a **client-scoped key**: Show the client name as a static label (no dropdown). Change the header title from "Arche Admin" to the client's name.

**2. Hide Import/Export from navigation drawer for client-scoped keys**
In `NavigationDrawer.tsx`, conditionally render the Import and Export nav items only when `isSuperAdmin` is true.

**3. Remove theme toggle from navigation drawer**
The theme toggle already exists in the header. Remove the duplicate toggle from the drawer's bottom section for all users.

**4. Remove client switcher from drawer when in header**
When the client selector is in the header, it can be removed from the drawer's top section to avoid duplication.

## Acceptance criteria

- [ ] Super admin sees a client dropdown in the header (same functionality as drawer client switcher)
- [ ] Client-scoped login shows client name as static label in header, "Arche Admin" replaced with client name
- [ ] Import and Export nav items hidden in drawer for client-scoped keys
- [ ] Theme toggle removed from drawer bottom section (everyone uses the header toggle)
- [ ] Client switcher removed from drawer when header selector is present
- [ ] Responsive behavior preserved on mobile

## Blocked by

- [Implement client-scoped key login in frontend](./implement-client-scoped-login.md)
