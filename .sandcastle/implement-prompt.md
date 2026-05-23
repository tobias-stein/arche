# TASK

Fix issue {{ISSUE_ID}}: {{ISSUE_TITLE}}

Read the issue file for details:

!`cat {{ISSUE_PATH}}`

Only work on the issue specified.

Work on branch {{BRANCH}}. Make commits, run checks, and complete the issue.

# CONTEXT

Here are the last 10 commits:

<recent-commits>

!`git log -n 10 --format="%H%n%ad%n%B---" --date=short`

</recent-commits>

# EXPLORATION

Explore the repo and fill your context window with relevant information that will allow you to complete the task.

Pay extra attention to test files that touch the relevant parts of the code.

# EXECUTION

If applicable, use RGR to complete the task.

1. RED: write one test
2. GREEN: write the implementation to pass that test
3. REPEAT until done
4. REFACTOR the code

# FEEDBACK LOOPS

Before committing, run the appropriate checks for what you changed:

- For Rust code: `cargo check --workspace && cargo clippy --workspace && cargo test --workspace`
- For Admin UI code: `cd admin-ui && npm run build && npm run lint`
- For both: run all of the above

# COMMIT

Make a git commit. The commit message must:

1. Start with `RALPH:` prefix
2. Include task completed + PRD reference
3. Key decisions made
4. Files changed
5. Blockers or notes for next iteration

Keep it concise.

# THE ISSUE

If the task is not complete, leave a note for the next iteration.

Do not change the issue status — this will be done during merge.

Once complete, output <promise>COMPLETE</promise>.

# FINAL RULES

ONLY WORK ON A SINGLE TASK.
