# ralph-loop

Alias for `/fix-ci`. See `.claude/skills/fix-ci.md` for the full implementation.

## Usage
Invoke: `/ralph-loop` or `/ralph-loop <branch-name-or-pr-url>`

## Behavior
Identical to `/fix-ci` — spawns a background worktree agent that iterates on CI failures until all checks pass.
