# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Run Commands

```bash
cargo build              # Build the project
cargo run                # Run dev server (http://0.0.0.0:8000)
cargo test               # Run tests
cargo fmt                # Format code
cargo clippy             # Lint code
diesel migration run     # Run database migrations (also auto-runs on startup)
```

## Architecture

This is a Rocket 0.5 web application for tracking scout hiking groups through checkpoints via QR code scanning.

### Core Components

**Database (Diesel + SQLite)**
- Connection pool via `rocket_sync_db_pools` (configured in `Rocket.toml`)
- Migrations embedded in binary via `embed_migrations!` macro, auto-run on startup
- Three tables: `groups`, `posts` (checkpoints), `scans` (group arrivals/departures at posts)
- All IDs are UUIDs stored as TEXT

**Authentication**
- Cookie-based admin auth using Rocket's `FromRequest` guard trait
- `Admin` guard in `src/auth.rs` checks for `admin_session` private cookie
- Password validated against `ADMIN_PASSWORD` environment variable
- Public routes use `is_admin()` helper for conditional UI

**Routes Structure** (`src/routes/`)
- `auth.rs` - Login/logout
- `dashboard.rs` - Public dashboard showing all groups and progress
- `scan.rs` - QR code scanning workflow (create group, record timestamps)
- `admin/groups.rs` - Group CRUD, timer controls, QR code generation
- `admin/posts.rs` - Checkpoint CRUD

**Templates**
- Tera templates in `templates/` with `.html.tera` extension
- Base layout in `base.html.tera`

### Key Patterns

- All DB operations use `DbConn::run(|c| ...)` closures for async execution
- POST handlers redirect after success (POST-Redirect-GET pattern)
- Forms use `#[derive(FromForm)]` structs
- Routes registered via `routes![]` macro in `main.rs`

### Scanning Workflow

1. User scans QR code → `/scan/<group_id>`
2. If group doesn't exist, form to create new group
3. Records: group start → arrive at post → leave post → group finish
4. Calculates idle time (at post) and walking time (total - idle)

## Environment Variables

Required in `.env` or environment:
- `ADMIN_PASSWORD` - Admin login password
- `ROCKET_SECRET_KEY` - 256-bit base64 key for secure cookies
