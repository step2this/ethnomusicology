#!/bin/bash
# Ralph Wiggum Loop — autonomous spec-driven iteration
# Usage: ./scripts/ralph-loop.sh [plan|build]
set -euo pipefail

MODE="${1:-build}"
MAX_ITERATIONS=20
PLAN_FILE="IMPLEMENTATION_PLAN.md"
PROMPT_FILE=".claude/prompts/ralph-prompt.md"
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

cd "$PROJECT_ROOT"

if [ ! -f "$PROMPT_FILE" ]; then
  echo "ERROR: $PROMPT_FILE not found"
  exit 1
fi

if [ "$MODE" = "plan" ]; then
  echo "Planning mode — generating IMPLEMENTATION_PLAN.md"
  cat "$PROMPT_FILE" | claude -p --allowedTools 'Read,Glob,Grep,WebFetch,WebSearch,Write'
  exit 0
fi

if [ ! -f "$PLAN_FILE" ]; then
  echo "ERROR: $PLAN_FILE not found. Run './scripts/ralph-loop.sh plan' first or create it manually."
  exit 1
fi

echo "Build mode — iterating until all tasks complete"
echo "Plan file: $PLAN_FILE"
echo "Max iterations: $MAX_ITERATIONS"
echo ""

for i in $(seq 1 $MAX_ITERATIONS); do
  echo "=== Iteration $i / $MAX_ITERATIONS ==="
  echo "$(date '+%Y-%m-%d %H:%M:%S')"

  # Check if any tasks remain
  if ! grep -q '^\- \[ \]' "$PLAN_FILE" 2>/dev/null; then
    echo ""
    echo "All tasks complete!"
    echo "Completed $(grep -c '^\- \[x\]' "$PLAN_FILE" 2>/dev/null || echo 0) tasks"
    exit 0
  fi

  # Show next task
  NEXT_TASK=$(grep -m1 '^\- \[ \]' "$PLAN_FILE" | head -c 120)
  echo "Next task: $NEXT_TASK"
  echo ""

  # Run fresh Claude instance
  OUTPUT=$(cat "$PROMPT_FILE" | claude -p 2>&1) || true
  echo "$OUTPUT" | tail -5

  # Check for completion signal
  if echo "$OUTPUT" | grep -q '<promise>DONE</promise>'; then
    echo ""
    echo "Task completed successfully"
  else
    echo ""
    echo "Task did not signal completion — continuing to next iteration (fresh context may help)"
  fi

  echo ""
  echo "---"
  echo ""
done

echo "Reached max iterations ($MAX_ITERATIONS)"
REMAINING=$(grep -c '^\- \[ \]' "$PLAN_FILE" 2>/dev/null || echo 0)
echo "$REMAINING tasks remaining"
exit 1
