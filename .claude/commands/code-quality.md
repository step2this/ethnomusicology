# Grade Work

Run the full quality pipeline against the current codebase and produce
a graded report.

## Steps

### Backend (Rust)
1. Run `cd backend && cargo clippy --message-format=json 2>&1` and capture output
2. Run `cd backend && cargo test 2>&1` and capture output
3. Run `cd backend && cargo fmt --check 2>&1` and capture output

### Frontend (Flutter)
4. Run `cd frontend && flutter analyze 2>&1` and capture output
5. Run `cd frontend && flutter test 2>&1` and capture output

## Analysis

Using the outputs above, grade the codebase against these criteria:

### Architectural Quality (A-F)
- Does the module structure follow clean architecture?
  (domain has zero deps on infra, services are separated from routes)
- Are module boundaries clean?

### Code Quality (A-F)
- Any clippy warnings?
- Any Flutter analyzer warnings?
- Functions with high complexity?
- Any unwrap/expect usage in Rust?

### Test Quality (A-F)
- Test coverage across backend and frontend
- Integration tests present?
- Edge cases covered?

## Output Format
Produce a markdown report with:
- Overall grade (weighted: Architecture 30%, Code 40%, Tests 30%)
- Per-module breakdown table
- Top 5 action items ranked by impact
