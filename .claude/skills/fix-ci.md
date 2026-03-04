# fix-ci

Fix CI failures in a background worktree by pulling GitHub CI logs, fixing locally, pushing, and repeating until CI passes. Also available as `/ralph-loop` (alias).

## Usage
Invoke: `/fix-ci` or `/fix-ci <branch-name-or-pr-url>`
Alias: `/ralph-loop` (same behavior)

## Why Remote CI?
Playwright E2E tests cannot run locally on this EC2 instance (headless browser setup is fragile). GitHub Actions runs them for us. So the loop is: pull failure logs from GitHub → fix locally → push → wait for new CI run → check again.

## Behavior

When invoked, the lead agent should:

1. **Determine the branch or PR.** If an argument is provided, use it. Otherwise, use the current branch. Resolve PR URLs to branch names via `gh pr view`.

2. **Spawn a worktree-isolated background agent** using the Agent tool with `isolation: "worktree"`, `run_in_background: true`, and `model: "sonnet"`.

3. **Give it the prompt below** (fill in `{BRANCH}` with the actual branch name):

```
You are a CI-fix agent. Your job is to get all GitHub Actions CI checks passing on the branch `{BRANCH}`.

You are working in an isolated git worktree. The branch is already checked out for you.

## The Loop (max 10 iterations)

### Step 1: Find the latest CI run
Run: gh run list --branch {BRANCH} --limit 1 --json databaseId,status,conclusion

- If status is "in_progress" or "queued", wait for it:
  Run: gh run watch <run-id> --exit-status
  (This blocks until the run completes. It may take up to 20 minutes.)
- If conclusion is "success", you're done — report success and stop.
- If conclusion is "failure", proceed to Step 2.
- If no runs exist yet, push the branch and wait for one to appear.

### Step 2: Pull the failure logs
Run: gh run view <run-id> --log-failed 2>&1 | tail -200

Read the failure output carefully. Identify which step failed:
- "Backend quality gates" → cargo fmt/clippy/test issue
- "Frontend quality gates" → flutter analyze/test issue
- "Run Playwright tests" → E2E test failure
- "Build Flutter web" or "Build backend" → compilation error

### Step 3: Run local gates first (fast feedback)
Run: cd backend && cargo fmt --check && cargo clippy -- -D warnings && cargo test 2>&1
Run: cd frontend && flutter analyze && flutter test 2>&1

If local gates fail, fix those first — they're faster to iterate on than waiting for CI.

### Step 4: Diagnose and fix
- Read the failing file(s) using the Read tool
- Make minimal, targeted fixes
- Do NOT delete or skip tests
- Do NOT weaken assertions to make tests pass
- For Playwright failures: read the test file in `e2e/tests/`, understand what it expects, and fix the backend/frontend code to match (not the test, unless the test itself is wrong)

### Step 5: Verify local gates pass
Run: cd backend && cargo fmt --check && cargo clippy -- -D warnings && cargo test 2>&1
Run: cd frontend && flutter analyze && flutter test 2>&1

If local gates fail, go back to Step 4.

### Step 6: Commit and push
Run: git add -A && git commit -m "Fix CI: <brief description of what was fixed>" && git push

### Step 7: Wait for CI
Run: gh run list --branch {BRANCH} --limit 1 --json databaseId,status --jq '.[0]'

Wait for the new run to appear (the push triggers it), then:
Run: gh run watch <run-id> --exit-status

- If it passes → report success and stop
- If it fails → go back to Step 2

## Rules
- Fix the actual issue, don't delete or skip tests
- Each fix should be minimal and targeted
- Commit messages: "Fix CI: <brief description>"
- Maximum 10 iterations before stopping and reporting what's still broken
- If you're stuck after 3 iterations on the same failure, report the issue and stop
- Always run local gates before pushing to avoid wasting CI minutes
- Use `gh run view <id> --log-failed` not `--log` (full logs are too large)
```

4. **Leave it alone** — the lead agent continues with other work. The background agent will be reported as complete when it finishes (success or max iterations reached).

5. **When notified of completion**, check the result:
   - If the agent reports success, the fixes are already pushed to the branch
   - If the agent reports remaining failures, review what it found and decide next steps
   - The worktree can be cleaned up (it will be auto-cleaned if no uncommitted changes remain)

## When to Use
- After pushing a PR and getting CI failures
- After a merge that breaks tests
- Anytime CI fails and the lead wants to keep working on other things
- Especially useful for Playwright E2E failures that can't be reproduced locally

## Key Details
- Uses `isolation: "worktree"` so the fix agent works on a separate copy of the repo
- Uses `run_in_background: true` so the main session continues unblocked
- The fix agent pushes directly to the branch — fixes appear on the PR automatically
- `gh run watch` blocks until CI completes, so the agent doesn't need to poll
- Local gates run first for fast feedback before waiting on remote CI (~5-20 min per cycle)
