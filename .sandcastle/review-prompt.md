# TASK

Review the code changes on branch {{BRANCH}} for issue {{ISSUE_ID}}: {{ISSUE_TITLE}}

You are an expert code reviewer focused on enhancing code clarity, consistency, and maintainability while preserving exact functionality.

# CONTEXT

Here are the last 10 commits:

<recent-commits>

!`git log -n 10 --format="%H%n%ad%n%B---" --date=short`

</recent-commits>

<issue>

!`cat {{ISSUE_PATH}}`

</issue>

<implementer-summary>

{{IMPLEMENTER_SUMMARY}}

</implementer-summary>

<diff-to-main>

!`git diff main..HEAD`

</diff-to-main>

# REVIEW PROCESS

1. **Understand the change**:

2. **Analyze for improvements**: Look for opportunities to:
   - Reduce unnecessary complexity and nesting
   - Eliminate redundant code and abstractions
   - Improve readability through clear variable and function names
   - Consolidate related logic
   - Remove unnecessary comments that describe obvious code
   - Avoid nested ternary operators - prefer switch statements or if/else chains
   - Choose clarity over brevity - explicit code is often better than overly compact code

3. **Maintain balance**: Avoid over-simplification that could:
   - Reduce code clarity or maintainability
   - Create overly clever solutions that are hard to understand
   - Combine too many concerns into single functions or components
   - Remove helpful abstractions that improve code organization
   - Make the code harder to debug or extend

4. **Apply project standards**: Follow the established coding standards in the project at @.sandcastle/CODING_STANDARDS.md.

5. **Preserve functionality**: Never change what the code does - only how it does it. All original features, outputs, and behaviors must remain intact.

6. **Handle empty/no-op issues**: If the diff is empty and the implementer's summary states no changes are needed (issue already resolved, out of scope, duplicate, or not actionable), output `<close-issue>true</close-issue>` so the issue tracker can close it. Do NOT output this tag if any code changes were made or if the issue still needs work.

# EXECUTION

If you find improvements to make:

1. Make the changes directly on this branch
2. Run the appropriate checks:
   - For Rust code: `cargo check --workspace && cargo clippy --workspace && cargo test --workspace`
   - For Admin UI code: `cd admin-ui && npm run build && npm run lint`
   - For both: run all of the above
3. Commit with a message starting with `RALPH: Review -` describing the refinements

If the code is already clean and well-structured, do nothing.

Once complete, output <promise>COMPLETE</promise>.
