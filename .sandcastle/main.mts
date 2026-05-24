import { readFileSync, writeFileSync } from "fs";
import * as sandcastle from "@ai-hero/sandcastle";
import { docker } from "@ai-hero/sandcastle/sandboxes/docker";
import { execSync } from "node:child_process";

const MAX_ITERATIONS = 3;
const MAX_PARALLEL = 4;
const IDLE_TIMEOUT = 30 * 60; // 30 minutes

// const PLAN_MODEL = "opencode/big-pickle";
const PLAN_MODEL = "opencode-go/deepseek-v4-pro";
// const IMPL_MODEL = "opencode/big-pickle";
const IMPL_MODEL = "opencode-go/deepseek-v4-pro";
// const REVI_MODEL = "opencode/big-pickle";
const REVI_MODEL = "opencode-go/deepseek-v4-flash";
// const MERG_MODEL = "opencode/big-pickle";
const MERG_MODEL = "opencode-go/qwen3.6-plus";



// Phase 0: Ticket Validation — handle unmerged branches from interrupted runs
const rawBranches = execSync(
  'git branch --no-merged main --list "sandcastle/*"',
  { encoding: "utf-8" },
)
  .split("\n")
  .map((b) => b.trim().replace(/^[*+]\s*/, ""))
  .filter((b) => b.length > 0);

if (rawBranches.length > 0) {
  console.log(
    `\n=== Phase 0: Ticket Validation — ${rawBranches.length} unmerged branch(es) found ===\n`,
  );
  for (const b of rawBranches) console.log(`  ${b}`);
  console.log();

  const validatedBranches: { branch: string; issuePath: string }[] = [];

  for (const branch of rawBranches) {
    const issuePath = branch.replace(/^sandcastle\//, "");
    const issueId = issuePath.replace(/\//g, "-");
    const issueFilePath = `.scratch/${issuePath}.md`;

    try {
      await using sandbox = await sandcastle.createSandbox({
        sandbox: docker(),
        branch,
        hooks: {
          sandbox: {
            onSandboxReady: [{ command: "cargo check --workspace 2>&1 || true" }],
          },
        },
      });

      await sandbox.run({
        name: "Reviewer (validation) " + issueId,
        agent: sandcastle.opencode(REVI_MODEL),
        promptFile: "./.sandcastle/review-prompt.md",
        promptArgs: {
          ISSUE_ID: issueId,
          ISSUE_TITLE: issuePath,
          ISSUE_PATH: issueFilePath,
          BRANCH: branch,
          IMPLEMENTER_SUMMARY: "(not applicable — validation of pre-existing branch)",
        },
        idleTimeoutSeconds: IDLE_TIMEOUT,
      });

      validatedBranches.push({ branch, issuePath });
    } catch (err) {
      console.error(`  ✗ Validation failed for ${branch}: ${err}`);
    }
  }

  if (validatedBranches.length > 0) {
    try {
      await sandcastle.run({
        sandbox: docker(),
        name: "Merger (validation)",
        maxIterations: 10,
        agent: sandcastle.opencode(MERG_MODEL),
        promptFile: "./.sandcastle/merge-prompt.md",
        promptArgs: {
          BRANCHES: validatedBranches.map((v) => `- ${v.branch}`).join("\n"),
          ISSUES: validatedBranches.map((v) => `- ${v.issuePath}`).join("\n"),
        },
        idleTimeoutSeconds: IDLE_TIMEOUT,
      });
      console.log("\nValidation merge complete.\n");
    } catch (err) {
      console.error(`Merge (validation) failed: ${err}`);
    }
  }

  console.log("=== Ticket Validation done, proceeding to planning ===\n");
}

for (let iteration = 1; iteration <= MAX_ITERATIONS; iteration++) {
  console.log(`\n=== Iteration ${iteration}/${MAX_ITERATIONS} ===\n`);

  // Phase 1: Plan — orchestrator agent analyzes issues and picks parallelizable work
  const plan = await sandcastle.run({
    sandbox: docker(),
    name: "Planner",
    agent: sandcastle.opencode(PLAN_MODEL),
    promptFile: "./.sandcastle/plan-prompt.md",
    idleTimeoutSeconds: IDLE_TIMEOUT
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
          agent: sandcastle.opencode(IMPL_MODEL),
          promptFile: "./.sandcastle/implement-prompt.md",
          promptArgs: {
            ISSUE_ID: issue.id,
            ISSUE_TITLE: issue.title,
            ISSUE_PATH: issue.path,
            BRANCH: issue.branch,
          },
          idleTimeoutSeconds: IDLE_TIMEOUT,
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
          agent: sandcastle.opencode(REVI_MODEL),
          promptFile: "./.sandcastle/review-prompt.md",
          promptArgs: {
            ISSUE_ID: issue.id,
            ISSUE_TITLE: issue.title,
            ISSUE_PATH: issue.path,
            BRANCH: issue.branch,
            IMPLEMENTER_SUMMARY: implementerSummary,
          },
          idleTimeoutSeconds: IDLE_TIMEOUT,
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
    agent: sandcastle.opencode(MERG_MODEL),
    promptFile: "./.sandcastle/merge-prompt.md",
    promptArgs: {
      BRANCHES: completedBranches.map((b) => `- ${b}`).join("\n"),
      ISSUES: completedIssues
        .map((i) => `- ${i.id}: ${i.title} (${i.path})`)
        .join("\n"),
    },
    idleTimeoutSeconds: IDLE_TIMEOUT
  });

  console.log("\nBranches merged.");
}

console.log("\nAll done.");