# Repeater Atlas: Agent Notes

## Quick Context

- Purpose: Public repeater directory and map with club-managed data.
- Users: Public readers; club admins/editors manage repeater entries.
- Deployment: Single-instance service (no horizontal scaling in MVP).
- Data access: Public read; admin editing only on main site domain.
- Club views: CNAME-based read-only views; embeddable minimal layout.

## Stack

- Rust + Axum (SSR) + Askama templates.
- HTMX for progressive enhancement.
- PostgreSQL + Diesel (including migrations).
- Auth: signed/encrypted session cookies; in-memory session store.
- Map: Leaflet + OSM tiles (provider swappable).

## Conventions

- Generate migrations via `diesel migration generate`, not by hand.

## Documentation

These must be read before doing any work.

- `docs/CODE.md`
- `docs/DATA_MODEL.md`
- `docs/DESIGN.md`
- `docs/IMPLEMENTATION.md`
- `docs/TESTING.md`

## CLI commands used while developing and refactoring

- Build the entire codebase: `make all`.
- Run all unit and integration tests with: `make test`
- Read the entire Makefile, look at all targets and comments. Learn the
  top-level targets as commands to run.
- Generate migrations via `diesel migration generate`, not by hand.
  - Requires `diesel_cli` installed
    (`cargo install diesel_cli --no-default-features --features postgres`).
  - Apply the migration with `diesel migration run`, it will automatically
    update `src/schema.rs`.
- Local Postgres pointed to by `DATABASE_URL` in `.env`.
- _NEVER_ commit any code without explicit instructions. Ask if a commit should
  be made, only if everything builds and all tests pass.

## Testing Notes

- Access the database via `bin/psql`. It will connect to the correct database
  automatically.
- Integration tests use a dedicated local Postgres via `DATABASE_URL`.
- Fixture loader can drop/recreate the test DB and run migrations.
- Tests commit changes (no transaction rollback).

## Landing the Plane (Session Completion)

**When ending a work session**, you MUST complete ALL steps below. Work is NOT
complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs
   follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd sync
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**

- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds

<!-- BEGIN BEADS INTEGRATION -->

## Issue Tracking with bd (beads)

**IMPORTANT**: This project uses **bd (beads)** for ALL issue tracking. Do NOT
use markdown TODOs, task lists, or other tracking methods.

### Why bd?

- Dependency-aware: Track blockers and relationships between issues
- Git-friendly: Auto-syncs to JSONL for version control
- Agent-optimized: JSON output, ready work detection, discovered-from links
- Prevents duplicate tracking systems and confusion

### Quick Start

**Check for ready work:**

```bash
bd ready --json
```

**Create new issues:**

```bash
bd create "Issue title" --description="Detailed context" -t bug|feature|task -p 0-4 --json
bd create "Issue title" --description="What this issue is about" -p 1 --deps discovered-from:bd-123 --json
```

**Claim and update:**

```bash
bd update bd-42 --status in_progress --json
bd update bd-42 --priority 1 --json
```

**Complete work:**

```bash
bd close bd-42 --reason "Completed" --json
```

### Issue Types

- `bug` - Something broken
- `feature` - New functionality
- `task` - Work item (tests, docs, refactoring)
- `epic` - Large feature with subtasks
- `chore` - Maintenance (dependencies, tooling)

### Priorities

- `0` - Critical (security, data loss, broken builds)
- `1` - High (major features, important bugs)
- `2` - Medium (default, nice-to-have)
- `3` - Low (polish, optimization)
- `4` - Backlog (future ideas)

### Workflow for AI Agents

1. **Check ready work**: `bd ready` shows unblocked issues
2. **Claim your task**: `bd update <id> --status in_progress`
3. **Work on it**: Implement, test, document
4. **Discover new work?** Create linked issue:
   - `bd create "Found bug" --description="Details about what was found" -p 1 --deps discovered-from:<parent-id>`
5. **Complete**: `bd close <id> --reason "Done"`

### Auto-Sync

bd automatically syncs with git:

- Exports to `.beads/issues.jsonl` after changes (5s debounce)
- Imports from JSONL when newer (e.g., after `git pull`)
- No manual export/import needed!

### Important Rules

- ✅ Use bd for ALL task tracking
- ✅ Always use `--json` flag for programmatic use
- ✅ Link discovered work with `discovered-from` dependencies
- ✅ Check `bd ready` before asking "what should I work on?"
- ❌ Do NOT create markdown TODO lists
- ❌ Do NOT use external issue trackers
- ❌ Do NOT duplicate tracking systems

For more details, see README.md and docs/QUICKSTART.md.

<!-- END BEADS INTEGRATION -->
