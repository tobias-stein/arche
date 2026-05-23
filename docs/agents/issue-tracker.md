# Issue Tracker

Issues are tracked as local markdown files under `.scratch/` in this repo.

## Conventions

- Each issue lives at `.scratch/<feature>/<issue-slug>.md`
- Frontmatter includes: `title`, `status` (using triage labels), and any other metadata
- New issues are created by writing a new markdown file in the appropriate `.scratch/` subdirectory
- There is no remote issue tracker — all work is tracked locally in this repo

## Creating an issue

1. Determine the feature or area of work
2. Create a directory `.scratch/<feature>/` if it doesn't exist
3. Write a markdown file with a descriptive slug name
4. Include frontmatter with `title` and `status` fields

## Status workflow

Issues move through the triage state machine. See `docs/agents/triage-labels.md` for the label vocabulary.
