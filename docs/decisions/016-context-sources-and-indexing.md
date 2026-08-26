# ADR-016 — Explicit Sources and bounded metadata indexing

- **Status:** Accepted
- **Date:** 2026-08-25

## Context

Aether needs local file awareness to support later universal search and continuity, but silently scanning a computer would violate its local-first, user-controlled identity. Directory access must be explicit, visible, revocable, resource-conscious, and separated from AI context. The first slice must also avoid turning file indexing into file ownership.

## Decision

A Source is a user-authorized canonical directory stored in SQLite with an optional Space association and scan status. A manual scan traverses only that directory in a blocking worker, collects bounded regular-file metadata, and applies one transactional snapshot to `indexed_files`.

The scanner:

- never follows symbolic links or Windows reparse points;
- skips known dependency, VCS, cache, and build-output directories;
- enforces fixed depth and file-count limits;
- stores normalized relative paths, names, extensions, byte size, and filesystem timestamps;
- never reads or stores file content in this milestone;
- marks unseen records removed only after a complete, non-truncated traversal;
- infers a rename only from a unique old/new size and modified-time match;
- reports skipped, failed, and truncated work instead of hiding it.

Removing a Source deletes only its SQLite records through cascading foreign keys. It never mutates the authorized directory. The Source root is returned to the UI because transparent authorization and revocation require it; indexed-file APIs expose only relative paths and metadata. Source data is not available to AI until a separate explicit-context design is accepted.

## Options considered

1. **Watch the filesystem immediately.** Rejected for the foundation because lifecycle recovery, event coalescing, rename semantics, and resource throttling need evidence from the snapshot model first.
2. **Index all user folders automatically.** Rejected because authorization would be implicit and scan scope difficult to understand.
3. **Store full child paths.** Rejected because the canonical Source plus validated relative path is sufficient and reduces duplicated sensitive path exposure.
4. **Extract text during the first scan.** Deferred to keep resource limits, format support, and later search indexing independently reviewable.

## Consequences

- Context Foundation becomes useful and testable without cloud access or file mutation.
- Manual rescans are required until a later background-indexing slice is accepted.
- Removed rows remain as local history for later continuity work but are excluded from normal current-file views.
- A truncated scan may add/update observed rows but cannot safely mark unobserved rows removed.
- Universal search, content extraction, automatic associations, and watchers remain later milestones.
