# Meta-PRD: auditor-tools

## 1. Project Overview

**Project Name:** auditor-tools  
**Platform:** Desktop application (Tauri 2.x)  
**Backend:** Rust  
**Frontend:** Vanilla JavaScript / TypeScript  
**Styling:** Tailwind CSS

`auditor-tools` is a modular, card-driven desktop application intended to host a suite of stand-alone and semi-related audit support tools. Each tool is accessed via a dashboard-style home screen composed of task cards. Tools are built incrementally, one card at a time, under strict phase control.

This Meta-PRD is the governing authority for all phase PRDs and must accompany each phase document.

---

## 2. Architectural Intent

- The application consists of a **single shared shell** (Tauri + frontend) and multiple **task modules**.
- Each dashboard card represents exactly one task module.
- Task modules may be:
  - Fully isolated, or
  - Consumers of shared internal libraries.
- Shared logic must live in clearly named Rust crates or frontend utility modules intended for reuse.
- Navigation model is strictly:
  - Dashboard → Task Home → Dashboard
  - No nested dashboards or cross-task navigation.

---

## 3. Canonical UI Reference

The dashboard layout, card structure, and visual language demonstrated in `layout1-dashboard-cards.html` are declared the **canonical baseline UI** for the application start screen.

- Spacing, hierarchy, and card affordances should match this reference.
- Visual changes are permitted only when explicitly authorized by a phase PRD.

---

## 4. Phase Index

- **PRD1:** Application shell, dashboard, and File Conversion task integration
- **PRD2:** DOCX/XLSX Unlocker task
- **PRD3:** DOCX Checkbox Fixer task
- **PRD4:** PCI DSS QA Tool
- Additional PRDs to be defined as development proceeds

Only one phase PRD may be active at a time.

---

## 5. Global Rules

### 5.1 Chain of Authority

1. This Meta-PRD overrides ambiguities in all phase PRDs.
2. Phase PRDs define scoped implementation steps.
3. No feature outside the active phase may be implemented.

### 5.2 Naming Rule

- Task identifiers, Rust commands, shared libraries, and UI components must use **stable names** across PRDs.
- Placeholders introduced in one phase must be referenced by the same name in later phases.

### 5.3 Replacement Rule

- Placeholders must be **fully replaced** in later phases.
- No placeholder logic may be partially retained or wrapped.
- Deprecated placeholders must be deleted when replaced.

### 5.4 File Conversion Authority Rule

The existing File Conversion application (from the provided repository copy) is authoritative for conversion logic.

- Logic must not be rewritten.
- Integration must adapt, not duplicate.
- UI behavior must be preserved; styling may be updated to match the auditor-tools visual system.

---

## 6. Transcript Fidelity Notes

- The card-based dashboard is a core product identity and not a temporary scaffold.
- Cards without functional backends must appear visually disabled (greyed out) until implemented.
- Tasks are expected to be built incrementally, one at a time, via future PRDs.

---

# PRD1: Dashboard Shell and File Conversion Integration

## 1. Purpose

This phase establishes the core application shell, renders the canonical dashboard, and integrates a single fully functional task: **File Conversion**.

This phase proves the end-to-end architecture without introducing additional task complexity.

---

## 2. In-Scope Features

1. Tauri 2.x application shell initialization
2. Frontend bootstrapping using Vanilla JS/TS and Tailwind CSS
3. Dashboard page matching the canonical card layout
4. Card state handling:
   - Enabled: File Conversion
   - Disabled: All other cards
5. Navigation from dashboard to File Conversion task home
6. Integration of existing File Conversion backend logic
7. Visual restyling of File Conversion UI to match auditor-tools styling

---

## 3. Out-of-Scope Features

- Any functionality for non-File Conversion cards
- New file conversion formats or logic changes
- Refactoring or optimizing File Conversion internals
- Persistent settings or preferences
- Plugin or extension systems

---

## 4. Placeholders

The following placeholders must be created and clearly marked for replacement:

- `DisabledTaskCard` – UI representation for unavailable tasks
  - Must be visually greyed out
  - Must block navigation
  - **Deprecated:** Replace with real task cards in future PRDs

- `TaskRouterStub` – Minimal routing mechanism
  - Routes only to File Conversion
  - **Deprecated:** Replace with full router as tasks are added

- `SharedLibPlaceholder` – Namespace for future shared logic
  - Contains no real functionality in this phase
  - **Deprecated:** Replace when shared libraries are introduced

All placeholders must include explicit deprecation comments.

---

## 5. User Flow

1. User launches auditor-tools
2. Dashboard loads with all task cards visible
3. Only File Conversion card appears enabled
4. User clicks File Conversion
5. Application navigates to File Conversion task home
6. User performs file conversion using existing functionality
7. User returns to dashboard

No other flows are permitted in this phase.

---

## 6. Deliverables

By the end of PRD1, the following must exist:

- A runnable Tauri desktop application
- A dashboard screen matching the canonical layout
- Correct enabled/disabled card behavior
- Fully functional File Conversion task
- Unified visual styling across dashboard and File Conversion UI

---

## 7. Handoff Contract

For the next phase (PRD2):

- `DisabledTaskCard` placeholders for specific tasks may be replaced
- `TaskRouterStub` must be expanded or replaced, not wrapped
- `SharedLibPlaceholder` may be populated or replaced as needed

All replacements must comply with the Meta-PRD Replacement Rule. No deprecated code may persist.

---

## 8. User Verification Steps

The user should verify:

1. The application launches without errors
2. The dashboard visually matches the provided layout reference
3. Disabled cards cannot be interacted with
4. File Conversion card opens correctly
5. File Conversion functionality behaves identically to the original app
6. Styling is consistent across the application

Known limitations:
- Only one task is functional
- Navigation is minimal and temporary
- Shared libraries are placeholders only

