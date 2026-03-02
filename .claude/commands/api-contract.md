---
description: Write or update OpenAPI section for a steel thread's endpoints
allowed-tools: Read, Glob, Write, Edit, Bash
---

# API Contract: $ARGUMENTS

You are an **API Contract Architect** writing OpenAPI specifications for the Ethnomusicology project. You translate the API Contract section from steel threads into formal OpenAPI 3.1 specs that both frontend and backend agents use as the source of truth. The steel thread reference is: **$ARGUMENTS**

## Step 1: Load the Steel Thread

Find and read the steel thread from `docs/steel-threads/`. The argument may be in any format:
- `ST-001` or `st-001` — scan for `st-001-*.md`
- `st-001-fetch-tracks` — exact filename match
- A descriptive phrase — search filenames for a match

Extract from the steel thread:
1. **API Contract Section** — the endpoint table (Method, Path, Description, Schema Ref, Status)
2. **Main Success Scenario** — for understanding request/response flow
3. **Integration Assertions** — for understanding expected behaviors
4. **Cross-Cutting References** — for understanding which UCs inform the data model

If the steel thread has no API Contract section, report an error and stop.

## Step 2: Load Existing OpenAPI Spec

Check if `docs/api/openapi.yaml` exists.

- **If it exists**: Read it. Note existing paths, components, and tags to avoid overwriting.
- **If it doesn't exist**: Prepare to create a new file with full boilerplate.

## Step 3: Define Endpoints

For each endpoint in the steel thread's API Contract table:

### 3a. Path and Method
Define the OpenAPI path item with the correct HTTP method. Follow these conventions:
- Paths start with `/api/`
- Resource names are plural (e.g., `/api/tracks`, `/api/setlists`)
- Use path parameters for specific resources (e.g., `/api/tracks/{trackId}`)
- Use camelCase for path parameters

### 3b. Request Body (POST/PUT/PATCH)
Define the request body schema:
- Use `application/json` content type
- Define inline schema or reference a component
- Mark required fields
- Include field descriptions and examples

### 3c. Response Schemas
Define responses for standard status codes:

**200 OK** (or 201 Created for POST):
- Response body schema matching the MSS happy path
- Include pagination wrapper for list endpoints:
  ```yaml
  allOf:
    - $ref: '#/components/schemas/PaginatedResponse'
    - properties:
        data:
          type: array
          items:
            $ref: '#/components/schemas/<Resource>'
  ```

**Error responses** — use the pre-built `components/responses` entries (do NOT inline the schema):
- `400`: `$ref: '#/components/responses/BadRequest'`
- `401`: `$ref: '#/components/responses/Unauthorized'`
- `404`: `$ref: '#/components/responses/NotFound'` (for single-resource endpoints)
- `500`: `$ref: '#/components/responses/InternalError'`

### 3d. Query Parameters (GET with filters/pagination)
Define query parameters following project conventions:
- `page` (integer, default 1) — page number
- `per_page` (integer, default 25, max 100) — items per page
- `sort` (string) — sort field
- `order` (string, enum: asc/desc) — sort direction
- Resource-specific filters as needed

### 3e. Tags
Add a tag matching the steel thread ID (e.g., `ST-001`) and a domain tag (e.g., `tracks`, `setlists`, `import`).

### 3f. Shared Components
Create or reference shared components:

**Always create these if they don't exist**:
```yaml
components:
  schemas:
    ErrorResponse:
      type: object
      required: [error]
      properties:
        error:
          type: object
          required: [code, message]
          properties:
            code:
              type: string
              description: Machine-readable error code
              example: VALIDATION_ERROR
            message:
              type: string
              description: Human-readable error message
              example: Invalid track ID format
            details:
              type: object
              description: Additional error context
    PaginatedResponse:
      type: object
      required: [page, per_page, total, total_pages]
      properties:
        page:
          type: integer
          example: 1
        per_page:
          type: integer
          example: 25
        total:
          type: integer
          example: 142
        total_pages:
          type: integer
          example: 6
  securitySchemes:
    userIdHeader:
      type: apiKey
      in: header
      name: X-User-Id
      description: Temporary user identification (replaced by JWT in UC-008)
```

## Step 4: Write or Merge into OpenAPI File

### If creating a new file (`docs/api/openapi.yaml`):

Create the directory if needed, then write the full spec:
```yaml
openapi: 3.1.0
info:
  title: Ethnomusicology API
  description: DJ-first music platform API — LLM-powered setlist generation with multi-source music import
  version: 0.1.0
  contact:
    name: Ethnomusicology Team
servers:
  - url: http://localhost:3001
    description: Local development server
security:
  - userIdHeader: []
tags: []
paths: {}
components:
  schemas: {}
  securitySchemes: {}
```

Then populate paths, components, and tags from Step 3.

### If merging into an existing file:

- **New paths**: Add to the `paths` section. Do NOT overwrite existing paths.
- **New components**: Add to `components/schemas`. Do NOT overwrite existing schemas.
- **New tags**: Append to the `tags` array. Do NOT duplicate existing tags.
- **Existing paths with new methods**: Add the new method to the existing path object.
- **Conflicts**: If an existing path/schema conflicts with the new definition, report the conflict and keep the existing version. Note the conflict in the summary.

## Step 5: Validate YAML Structure

Run a basic structural validation:

```bash
# Check that the file is valid YAML
python3 -c "import yaml; yaml.safe_load(open('docs/api/openapi.yaml'))"
```

Verify these required OpenAPI fields are present:
- `openapi` version string
- `info.title` and `info.version`
- `paths` object (may be empty)
- `components` object

If validation fails, fix the YAML and re-validate.

## Step 6: Update the Steel Thread

Edit the steel thread file to update the API Contract section:
- Add **OpenAPI Schema Ref** for each endpoint (e.g., `#/paths/~1api~1tracks/get`)
- Update **Contract Status** to "Draft" for all newly defined endpoints

Use the Edit tool to make targeted changes — do not rewrite the entire steel thread file.

## Step 7: Report Summary

Output a summary including:

1. **Endpoints Written**:
   - `GET /api/tracks` — List tracks with pagination and filters
   - `POST /api/setlists` — Create a new setlist
   - etc.

2. **Shared Components Created/Referenced**:
   - `ErrorResponse` — created / already existed
   - `PaginatedResponse` — created / already existed
   - `Track` — created
   - etc.

3. **Conflicts** (if any):
   - `GET /api/tracks` already existed with different schema — kept existing version

4. **API Contract Review Gate Reminder**:
   > Before implementation begins, both frontend and backend agents must review and confirm the API contract. Use `/uc-review` on the steel thread to trigger a contract review. The contract status should progress from Draft → Agreed before any agent writes implementation code.

5. **Next Steps**:
   - Run `/uc-review st-<NNN>` to review the steel thread with its new contract references
   - Run `/task-decompose ST-<NNN>` to break the thread into implementable tasks
   - Frontend and backend agents should confirm the contract before starting work
