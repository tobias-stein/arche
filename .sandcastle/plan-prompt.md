# ISSUES LIST

Here are the open issues in the local file-based tracker:

!`find .scratch -name '*.md' -exec grep -l 'status: ready-for-agent' {} + | while read f; do title=$(head -6 "$f" | grep '^title:' | sed 's/^title: *//'); blocked=$(grep -c '^- ' "$f" 2>/dev/null || echo 0); echo "$f|$title"; done`

# TASK

Read each issue file listed above using the `read` tool. For each issue, understand what it requires and whether it **blocks** or **is blocked by** any other open issue (look at the "Blocked by" section in each issue).

An issue B is **blocked by** issue A if:
- B lists A (by its `.scratch/<feature>/<slug>.md` path) in its "Blocked by" section
- B requires code or infrastructure that A introduces
- B and A modify overlapping files or modules, making concurrent work likely to produce merge conflicts
- B's requirements depend on a decision or API shape that A will establish

An issue is **unblocked** if it has zero blocking dependencies on other open issues.

For each unblocked issue, assign a branch name using the format `sandcastle/<feature>/<issue-slug>` (derived from its file path under `.scratch/`).

If the issue appears to be a PRD and it has implementation issues which link to it, the PRD cannot be worked on.

# OUTPUT

Output your plan as a JSON object wrapped in `<plan>` tags:

<plan>
{"issues": [{"id": "admin-ui/blueprints-list-page", "title": "Blueprints — list page", "branch": "sandcastle/admin-ui/blueprints-list-page", "path": ".scratch/admin-ui/blueprints-list-page.md"}]}
</plan>

Include only unblocked issues. If every issue is blocked, include the single highest-priority candidate (the one with the fewest or weakest dependencies).
