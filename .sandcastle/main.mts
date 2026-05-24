import { readFileSync, writeFileSync } from "fs";
import * as sandcastle from "@ai-hero/sandcastle";
import { docker } from "@ai-hero/sandcastle/sandboxes/docker";

const MAX_ITERATIONS = 3;
const MAX_PARALLEL = 4;

for (let iteration = 1; iteration <= MAX_ITERATIONS; iteration++) {
  console.log(`\n=== Iteration ${iteration}/${MAX_ITERATIONS} ===\n`);

  // Phase 1: Plan — orchestrator agent analyzes issues and picks parallelizable work
  const plan = await sandcastle.run({
    sandbox: docker(),
    name: "Planner",
    agent: sandcastle.opencode("opencode/big-pickle"),
    promptFile: "./.sandcastle/plan-prompt.md",
  });

  const planMatch = plan.stdout.match(/<plan>([\s\S]*?)<\/plan>/);
  if (!planMatch) {
    throw new Error(
      "Orchestrator did not produce a <plan> tag.\n\n" + plan.stdout,
    );
  }

  const { issues } = JSON.parse(planMatch[1]) as {
    issues: { id: string; title: string; branch: string; path: string }[];
  };

  if (issues.length === 0) {
    console.log("No issues to work on. Exiting.");
    break;
  }

  console.log(
    `Planning complete. ${issues.length} issue(s) to work in parallel:`,
  );
  for (const issue of issues) {
    console.log(`  ${issue.id}: ${issue.title} → ${issue.branch}`);
  }

  // Phase 2: Execute + Review — implement then review each branch, max 4 in parallel
  let running = 0;
  const queue: (() => void)[] = [];
  const acquire = () =>
    running < MAX_PARALLEL
      ? (running++, Promise.resolve())
      : new Promise<void>((resolve) => queue.push(resolve));
  const release = () => {
    running--;
    const next = queue.shift();
    if (next) {
      running++;
      next();
    }
  };

  const settled = await Promise.allSettled(
    issues.map(async (issue) => {
      await acquire();
      try {
        await using sandbox = await sandcastle.createSandbox({
          sandbox: docker(),
          branch: issue.branch,
          hooks: {
            sandbox: {
              onSandboxReady: [{ command: "cargo check --workspace 2>&1 || true" }],
            },
          },
        });

        const result = await sandbox.run({
          name: "Implementer " + issue.id,
          agent: sandcastle.opencode("opencode/big-pickle"),
          promptFile: "./.sandcastle/implement-prompt.md",
          promptArgs: {
            ISSUE_ID: issue.id,
            ISSUE_TITLE: issue.title,
            ISSUE_PATH: issue.path,
            BRANCH: issue.branch,
          },
        });

        const summaryMatch = result.stdout.match(
          /<summary>([\s\S]*?)<\/summary>/,
        );
        const implementerSummary = summaryMatch
          ? summaryMatch[1].trim()
          : "(no summary provided)";

        const commitCount = result.commits.length;
        if (commitCount > 0) {
          console.log(
            `  - ${issue.id}: ${commitCount} commit(s), summary: ${implementerSummary}`,
          );
        } else {
          console.log(
            `  - ${issue.id}: no commits, summary: ${implementerSummary}`,
          );
        }

        const reviewResult = await sandbox.run({
          name: "Reviewer " + issue.id,
          agent: sandcastle.opencode("opencode/big-pickle"),
          promptFile: "./.sandcastle/review-prompt.md",
          promptArgs: {
            ISSUE_ID: issue.id,
            ISSUE_TITLE: issue.title,
            ISSUE_PATH: issue.path,
            BRANCH: issue.branch,
            IMPLEMENTER_SUMMARY: implementerSummary,
          },
        });

        const closeMatch = reviewResult.stdout.match(
          /<close-issue>\s*true\s*<\/close-issue>/,
        );
        if (closeMatch) {
          const issuePath = issue.path;
          let content = readFileSync(issuePath, "utf-8");
          content = content.replace(
            /^status: ready-for-agent$/m,
            "status: closed\nresolution: no-work-needed",
          );
          writeFileSync(issuePath, content);
          console.log(
            `  ✓ ${issue.id}: closed (no work needed per reviewer)`,
          );
        }

        return result;
      } finally {
        release();
      }
    }),
  );

  for (const [i, outcome] of settled.entries()) {
      if (outcome.status === "rejected") {
      console.error(
        `  ✗ ${issues[i].id} (${issues[i].branch}) failed: ${outcome.reason}`,
      );
    }
  }

  const completedIssues = settled
    .map((outcome, i) => ({ outcome, issue: issues[i] }))
    .filter(
      (
        entry,
      ): entry is {
        outcome: PromiseFulfilledResult<
          Awaited<ReturnType<typeof sandcastle.run>>
        >;
        issue: (typeof issues)[number];
      } =>
        entry.outcome.status === "fulfilled" &&
        entry.outcome.value.commits.length > 0,
    )
    .map((entry) => entry.issue);

  const completedBranches = completedIssues.map((i) => i.branch);

  console.log(
    `\nExecution complete. ${completedBranches.length} branch(es) with commits:`,
  );
  for (const branch of completedBranches) {
    console.log(`  ${branch}`);
  }

  if (completedBranches.length === 0) {
    console.log("No commits produced. Nothing to merge.");
    continue;
  }

  // Phase 3: Merge — one agent merges all branches together
  await sandcastle.run({
    sandbox: docker(),
    name: "Merger",
    maxIterations: 10,
    agent: sandcastle.opencode("opencode/big-pickle"),
    promptFile: "./.sandcastle/merge-prompt.md",
    promptArgs: {
      BRANCHES: completedBranches.map((b) => `- ${b}`).join("\n"),
      ISSUES: completedIssues
        .map((i) => `- ${i.id}: ${i.title} (${i.path})`)
        .join("\n"),
    },
  });

  console.log("\nBranches merged.");
}

console.log("\nAll done.");