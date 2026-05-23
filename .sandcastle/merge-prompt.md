# TASK

Merge the following branches into the current branch:

{{BRANCHES}}

For each branch:

1. Run `git merge <branch> --no-edit`
2. If there are merge conflicts, resolve them intelligently by reading both sides and choosing the correct resolution
3. After resolving conflicts, run the appropriate checks:
   - For Rust code: `cargo check --workspace && cargo clippy --workspace && cargo test --workspace`
   - For Admin UI code: `cd admin-ui && npm run build && npm run lint`
   - For both: run all of the above
4. If checks fail, fix the issues before proceeding to the next branch

After all branches are merged, make a single commit summarizing the merge.

# CLOSE ISSUES

For each branch that was merged, update its issue file status to `completed`. Change the `status` field in the frontmatter from `ready-for-agent` to `completed` in each issue's `.md` file.

If there are any parent issues (such as PRDs) which completing this issue would finish, close those too by updating their status to `completed`.

Here are all the issues:

{{ISSUES}}

Once you've merged everything you can, output <promise>COMPLETE</promise>.
