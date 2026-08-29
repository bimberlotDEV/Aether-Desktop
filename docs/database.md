# Aether Database Schema

## Storage

The SQLite database is stored at `%APPDATA%/com.aether.desktop/aether.db` on Windows. Tauri derives this runtime app-data directory from the configured application identifier, resolving to `C:\Users\<user>\AppData\Roaming\com.aether.desktop\aether.db`.

- **Journal mode:** WAL (Write-Ahead Logging) for concurrent reads
- **Foreign keys:** Enabled
- **Synchronization:** NORMAL (safe with WAL)

## Tables

### app_settings

Key-value store for application preferences.

| Column | Type | Notes |
|---|---|---|
| `key` | TEXT PK | Setting identifier |
| `value` | TEXT | Setting value (string-encoded) |
| `value_type` | TEXT | Type hint (string, bool, number) |
| `created_at` | TEXT | ISO 8601 timestamp |
| `updated_at` | TEXT | ISO 8601 timestamp |

### user_profile

Local user profile (single row). Not a cloud account.

| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUIDv7 |
| `display_name` | TEXT | User's name |
| `onboarding_completed` | INTEGER | 0 or 1; authoritative local first-run state |

Profile initialization is idempotent. If no profile exists in an otherwise empty database, Aether inserts one with `onboarding_completed = 0`. If meaningful domain rows already exist, the missing profile is treated as an upgrade from a release that did not create profiles and is inserted completed. Existing domain rows are only detected and are never rewritten by initialization.
| `created_at` | TEXT | ISO 8601 |
| `updated_at` | TEXT | ISO 8601 |

### spaces

User-created workspaces.

| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUIDv7 |
| `name` | TEXT | Required |
| `description` | TEXT | Optional |
| `icon` | TEXT | Lucide icon name |
| `accent` | TEXT | Hex color |
| `template_type` | TEXT | blank, school, developer, professional, personal, project, or subject |
| `favourite` | INTEGER | 0 or 1 |
| `archived_at` | TEXT | NULL if active |
| `sort_order` | INTEGER | Manual ordering |
| `settings_json` | TEXT | JSON blob |
| `created_at` | TEXT | ISO 8601 |
| `updated_at` | TEXT | ISO 8601 |

Indexes: `favourite`, `archived_at`, `sort_order`

### module_instances

Modules enabled within a Space.

| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUIDv7 |
| `space_id` | TEXT FK | References spaces(id) ON DELETE CASCADE |
| `module_type` | TEXT | Module identifier |
| `title` | TEXT | Display title |
| `config_json` | TEXT | Module configuration |
| `layout_json` | TEXT | Layout settings |
| `created_at` | TEXT | ISO 8601 |
| `updated_at` | TEXT | ISO 8601 |

Index: `space_id`

### activity_events

Meaningful action history.

| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUIDv7 |
| `event_type` | TEXT | Action category |
| `entity_type` | TEXT | e.g., space, note, task |
| `entity_id` | TEXT | Affected entity |
| `space_id` | TEXT FK | References spaces(id) ON DELETE SET NULL |
| `metadata_json` | TEXT | Additional context |
| `created_at` | TEXT | ISO 8601 |

Indexes: `created_at`, `space_id`, `event_type`

### tasks

Local-first Tasks and self-referencing subtasks. The boundary model is defined by ADR-009.

| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUIDv7 |
| `space_id` | TEXT FK | Optional; references spaces(id) ON DELETE SET NULL |
| `parent_task_id` | TEXT FK | Optional subtask parent; references tasks(id) ON DELETE CASCADE |
| `title` | TEXT | Required, trimmed, maximum 200 characters |
| `description` | TEXT | Plain text, maximum 10,000 characters |
| `status` | TEXT | inbox, planned, in_progress, done |
| `priority` | TEXT | none, low, medium, high |
| `due_date` | TEXT | Optional local calendar date (`YYYY-MM-DD`) |
| `tags_json` | TEXT | Validated JSON array; exposed as `tags: string[]` |
| `completed_at` | TEXT | Set exactly when status is done |
| `archived_at` | TEXT | NULL if active |
| `created_at` | TEXT | ISO 8601 timestamp |
| `updated_at` | TEXT | ISO 8601 timestamp |

Indexes: `space_id`, `parent_task_id`, `status`, `priority`, `due_date`, `archived_at`

Repository filters own Task search and enum filtering. Pulse uses a bounded due-attention query for overdue and upcoming top-level Tasks.

### vault_items

Metadata and ownership records for local files. Filesystem behavior is governed by ADR-010.

| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUIDv7; also owns the managed item directory |
| `space_id` | TEXT FK | Optional; references spaces(id) ON DELETE SET NULL |
| `storage_mode` | TEXT | linked or managed |
| `display_title` | TEXT | User-editable title, 1 to 200 characters |
| `original_name` | TEXT | Filename captured at import |
| `stored_path` | TEXT | Canonical absolute path for linked items; Vault-relative path for managed items |
| `media_type` | TEXT | Extension-derived MIME classification; unsupported formats remain octet-stream |
| `size_bytes` | INTEGER | Non-negative source size at import |
| `tags_json` | TEXT | Validated JSON array; exposed as `tags: string[]` |
| `created_at` | TEXT | ISO 8601 timestamp |
| `updated_at` | TEXT | ISO 8601 timestamp |

Indexes: `space_id`, `storage_mode`, `display_title`, `updated_at`. `stored_path` is unique.

Managed files live below `%APPDATA%/com.aether.desktop/vault/items/<item-id>/`. Linked originals are never mutated or deleted by Vault removal. `stored_path` remains internal to Rust; native open and reveal operations resolve a database item rather than accepting or returning arbitrary frontend paths.

### memory_items

Explicit user-authored durable context, governed by ADR-012.

| Column | Type | Notes |
|---|---|---|
| `id` | TEXT PK | UUIDv7 |
| `space_id` | TEXT FK | Optional global scope; Space rows use ON DELETE CASCADE |
| `title` | TEXT | Required, 1 to 200 characters |
| `content` | TEXT | Required, maximum 20,000 characters |
| `reason` | TEXT | Required user-visible attribution, maximum 500 characters |
| `category` | TEXT | preference, decision, recurring_context, terminology, goal, constraint |
| `source` | TEXT | MVP is strictly user-authored |
| `created_at` | TEXT | ISO 8601 timestamp |
| `updated_at` | TEXT | ISO 8601 timestamp |

Indexes: `space_id`, `category`, `updated_at`. A cleanup trigger removes polymorphic AI context attachments when an item is deleted. Content is sent externally only after explicit attachment and Rust scope validation.

## Migrations

Versioned in `src-tauri/src/db/migrations.rs`. Each migration has a name and SQL. Applied migrations are tracked in `_migrations` table. Migrations run in a transaction — all or nothing.

To add a migration:
1. Add a new entry to `MIGRATIONS` array with unique name
2. Write the SQL
3. No migration may modify a previously-applied migration

## Workspace export

Settings creates a versioned `.aether-backup` archive containing a consistent SQLite online-backup snapshot and the exact bytes of every managed Vault item. The snapshot removes `secrets`; linked external files are never read or copied. A strict manifest binds every payload to its size and SHA-256 digest.

Restore first verifies archive paths, limits, hashes, migration compatibility, SQLite integrity, relationships, and managed-file ownership. A separate one-time approval stages and migrates the replacement, preserves only current device-local credential rows, creates a complete pre-restore recovery archive, and restarts Aether. Before SQLite opens, startup swaps the staged database and managed Vault tree together and rolls back or resumes safely after an interrupted swap. Restore replaces the workspace; it does not merge records. The legacy `.aether-backup.db` native export remains for compatibility. See ADR-014 and ADR-022.
