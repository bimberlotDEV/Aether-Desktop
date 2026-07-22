# Aether Database Schema

## Storage

The SQLite database is stored at `%APPDATA%/Aether/aether.db` on Windows. This resolves to `C:\Users\<user>\AppData\Roaming\Aether\aether.db`.

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
| `onboarding_completed` | INTEGER | 0 or 1 |
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
| `template_type` | TEXT | blank, school, personal, project |
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

## Migrations

Versioned in `src-tauri/src/db/migrations.rs`. Each migration has a name and SQL. Applied migrations are tracked in `_migrations` table. Migrations run in a transaction — all or nothing.

To add a migration:
1. Add a new entry to `MIGRATIONS` array with unique name
2. Write the SQL
3. No migration may modify a previously-applied migration
