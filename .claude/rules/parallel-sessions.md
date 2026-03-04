---
paths:
  - "docs/**"
  - "backend/**"
  - "frontend/**"
---

# Parallel Session Protocol

When multiple Claude Code sessions work on this repo simultaneously:

## Starting a Session
1. **Read `docs/session-handoff.md` FIRST** — it lists active work and file ownership
2. **Claim your files** by editing the "File Ownership" table in session-handoff.md
3. Create your feature branch: `feature/{st|uc}-NNN-{slug}-{stream}`
4. If using worktrees: `git worktree add .claude/worktrees/{name} -b {branch}`

## File Ownership Rules
- Each session MUST list files it owns in session-handoff.md
- **NEVER modify a file owned by another session**
- Shared files (`main.rs`, `mod.rs`, `pubspec.yaml`, `Cargo.toml`) → only ONE session touches them
- If you need a shared file: finish your work, commit, then coordinate via session-handoff.md

## Crash Recovery
- **Commit after every completed task** (not just at the end)
- If OOM/crash: `git log --oneline -10` + `docs/session-handoff.md` = full recovery
- Check for stale `.git/index.lock` files after crashes (`rm` if empty and no git process running)

## Merge Strategy
- Each stream creates its own PR
- Independent streams (backend vs frontend): merge in any order
- Overlapping streams: merge one first, rebase the other
- Shared files go in the LAST PR to merge (reduces conflicts)
