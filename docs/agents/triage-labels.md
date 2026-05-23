# Triage Labels

This repo uses the default label vocabulary for the triage state machine.

## Label mapping

| Canonical role    | Label string     | Meaning                                      |
| ----------------- | ---------------- | -------------------------------------------- |
| needs-triage      | `needs-triage`   | Maintainer needs to evaluate the issue       |
| needs-info        | `needs-info`     | Waiting on reporter for more information     |
| ready-for-agent   | `ready-for-agent`| Fully specified, AFK-ready                   |
| ready-for-human   | `ready-for-human`| Needs human implementation                   |
| wontfix           | `wontfix`        | Will not be actioned                         |

## Usage

- When triaging an issue, apply exactly one of these labels
- The `triage` skill reads and writes these strings to move issues through the state machine
- For local markdown, labels are recorded in the issue's frontmatter as `status`
