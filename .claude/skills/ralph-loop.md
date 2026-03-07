# ralph-loop

Autonomous spec-driven iteration loop. Reads a plan file, picks the next pending task, implements it, runs quality gates, commits, and exits for fresh context.

## Usage
Invoke: `/ralph-loop`

## Behavior
1. Read `IMPLEMENTATION_PLAN.md` from project root
2. Find the FIRST task marked `- [ ]` (pending)
3. Read any files listed in the task's "Files" section
4. Implement the task completely
5. Run quality gates:
   - Backend: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`
   - Frontend: `flutter analyze`, `flutter test`
6. If gates pass: commit with descriptive message, mark task `[x]` in IMPLEMENTATION_PLAN.md
7. If gates fail: fix the issue and retry (max 3 attempts)
8. Output `<promise>DONE</promise>` when the task is complete and committed

## Rules
- ONE task per iteration — do not start the next task
- Commit after every completed task (crash recovery)
- Do not modify files outside the task's scope
- If stuck after 3 attempts, mark the task with `[!]` and a note, then output `<promise>DONE</promise>` to move on

## External Loop
Run via `./scripts/ralph-loop.sh build` — bash loop that invokes fresh Claude instances until all tasks are `[x]`.
