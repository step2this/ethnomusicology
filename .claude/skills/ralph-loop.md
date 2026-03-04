# ralph-loop

Automatically run the CI fix loop after a PR is opened. Named after Ralph Wiggum ("I'm in danger") — the feeling when you push and wait for Playwright.

## Usage
Invoke: `/ralph-loop` or `/ralph-loop <pr-number-or-branch>`

## What It Does

This is the final step after opening a PR. It spawns a background worktree agent that watches the CI run triggered by the PR push, and if it fails, iterates on fixes until Playwright and all other checks pass.

## Behavior

When invoked, the lead agent should:

1. **Determine the branch.** If an argument is provided (PR number or branch name), resolve it. Otherwise use the current branch. For PR numbers, resolve via `gh pr view <number> --json headRefName --jq .headRefName`.

2. **Spawn the fix agent** using the Agent tool with:
   - `isolation: "worktree"`
   - `run_in_background: true`
   - `model: "sonnet"` (coding agent, not haiku)

3. **Give it this prompt** (fill in `{BRANCH}`):

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

4. **Report to the user** that the loop is running in the background and they'll be notified when it finishes.

## When to Use
- As the final step after opening a PR (`gh pr create`)
- After pushing fixes to a PR branch
- Anytime CI fails and you want hands-off iteration
- Especially for Playwright E2E failures that can't run locally on this EC2 instance

## Key Details
- Uses `isolation: "worktree"` so fix work doesn't interfere with the main session
- Uses `run_in_background: true` so the main session is unblocked
- The agent pushes directly to the branch — fixes show up on the PR automatically
- `gh run watch` blocks until CI completes, no polling needed
- Local quality gates run first for fast feedback before the 5-20 min CI round-trip
