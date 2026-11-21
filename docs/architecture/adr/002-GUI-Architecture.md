# ADR-002 – GUI architecture

**Status:** Accepted  
**Date:** 2026-11-21

## Context

A single timeline-based screen with a waveform, bookmarks, transport and tuning controls grows quickly into monolithic widgets and controllers. The prototype's reusable waveform control showed the value of keeping view logic apart from the rendering.

## Decision

The GUI follows Recursive Separation of Concerns with feature-oriented component composition, as defined in `docs/development/GUI-Architecture.md`. The document is binding for all Flutter code. Complex geometry and interaction logic (time-to-pixel conversion, snapping, region editing rules) live in the Rust client core or in pure Dart units without Flutter dependencies, not in widgets. State management is not prescribed by a framework; its boundaries follow the rules of the document.

## Consequences

Each feature owns its screens, components, presentation state and styles. A design system holds only genuinely shared tokens and primitives. Non-trivial GUI changes start with a short description of the intended component decomposition and end with the architectural quality gate. The choice of a Flutter state-management package is a later, separate decision.
