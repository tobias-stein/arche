# Domain Docs

This repo uses a single-context layout.

## Layout

- `CONTEXT.md` at the repo root — project domain language, architecture overview, and key terminology
- `docs/adr/` at the repo root — architectural decision records

## Consumer rules

- Skills that need domain context read `CONTEXT.md` first to learn the project's language
- Skills that need to understand past decisions read ADRs from `docs/adr/`
- If `CONTEXT.md` does not exist, skills should not assume any domain knowledge
- If `docs/adr/` does not exist, there are no recorded architectural decisions
