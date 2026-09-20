import { z } from 'zod'

export const AppSettingSchema = z.object({
  key: z.string(),
  value: z.string(),
  value_type: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
})
export type AppSetting = z.infer<typeof AppSettingSchema>

export const UserProfileSchema = z.object({
  id: z.string(),
  display_name: z.string().nullable(),
  onboarding_completed: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
})
export type UserProfile = z.infer<typeof UserProfileSchema>

export const SpaceSchema = z.object({
  id: z.string(),
  name: z.string(),
  description: z.string().nullable(),
  icon: z.string().nullable(),
  accent: z.string().nullable(),
  template_type: z.string().nullable(),
  favourite: z.boolean(),
  archived_at: z.string().nullable(),
  sort_order: z.number(),
  settings_json: z.string().nullable(),
  parent_space_id: z.string().nullable(),
  last_opened_at: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
})
export type Space = z.infer<typeof SpaceSchema>

export const ModuleInstanceSchema = z.object({
  id: z.string(),
  space_id: z.string(),
  module_type: z.string(),
  title: z.string().nullable(),
  config_json: z.string().nullable(),
  layout_json: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
})
export type ModuleInstance = z.infer<typeof ModuleInstanceSchema>

export const SpaceWithDetailsSchema = z.object({
  space: SpaceSchema,
  modules: z.array(ModuleInstanceSchema),
  children: z.array(SpaceSchema),
})
export type SpaceWithDetails = z.infer<typeof SpaceWithDetailsSchema>

export const ActivityEventTypeSchema = z.enum([
  'space_opened',
  'note_created',
  'note_edited',
  'task_created',
  'task_created_from_ai_proposal',
  'task_completed',
  'task_archived',
  'vault_imported',
  'vault_updated',
  'vault_removed',
  'memory_created',
  'memory_updated',
  'memory_deleted',
  'source_scanned',
  'ai_conversation_used',
  'action_executed',
])

export const ActivityItemSchema = z.object({
  id: z.string(),
  eventType: ActivityEventTypeSchema,
  title: z.string(),
  detail: z.string().nullable(),
  spaceId: z.string().nullable(),
  spaceName: z.string().nullable(),
  entityType: z.string().nullable(),
  entityId: z.string().nullable(),
  destination: z.string(),
  createdAt: z.string(),
})
export type ActivityItem = z.infer<typeof ActivityItemSchema>

export const ContinuityItemSchema = z.object({
  id: z.string(),
  title: z.string(),
  detail: z.string(),
  updatedAt: z.string(),
  destination: z.string(),
  provenance: z.string(),
})
export type ContinuityItem = z.infer<typeof ContinuityItemSchema>

export const ContinuitySuggestionSchema = z.object({
  title: z.string(),
  detail: z.string(),
  destination: z.string(),
  sourceType: z.enum(['task', 'note', 'conversation', 'file', 'empty']),
  sourceId: z.string().nullable(),
})

export const SpaceContinuitySchema = z.object({
  spaceId: z.string(),
  spaceName: z.string(),
  lastWorkedAt: z.string(),
  recentNotes: z.array(ContinuityItemSchema),
  openTasks: z.array(ContinuityItemSchema),
  recentFiles: z.array(ContinuityItemSchema),
  latestConversation: ContinuityItemSchema.nullable(),
  recentActivity: z.array(ActivityItemSchema),
  suggestedNextStep: ContinuitySuggestionSchema,
})
export type SpaceContinuity = z.infer<typeof SpaceContinuitySchema>

export const NoteSchema = z.object({
  id: z.string(),
  space_id: z.string(),
  title: z.string(),
  content: z.string(),
  content_format: z.string(),
  excerpt: z.string(),
  pinned: z.boolean(),
  revision: z.number(),
  archived_at: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
  last_opened_at: z.string().nullable(),
})
export type Note = z.infer<typeof NoteSchema>

export const NoteListItemSchema = z.object({
  id: z.string(),
  space_id: z.string(),
  title: z.string(),
  excerpt: z.string(),
  content_format: z.string(),
  pinned: z.boolean(),
  revision: z.number(),
  archived_at: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
  last_opened_at: z.string().nullable(),
})
export type NoteListItem = z.infer<typeof NoteListItemSchema>

export const NoteSearchResultSchema = z.object({
  id: z.string(),
  space_id: z.string(),
  title: z.string(),
  excerpt: z.string(),
  pinned: z.boolean(),
  archived_at: z.string().nullable(),
  updated_at: z.string(),
})
export type NoteSearchResult = z.infer<typeof NoteSearchResultSchema>

// ─── Task Types ──────────────────────────────────────────

export const TaskStatusSchema = z.enum(['inbox', 'planned', 'in_progress', 'done'])
export type TaskStatus = z.infer<typeof TaskStatusSchema>

export const TaskPrioritySchema = z.enum(['none', 'low', 'medium', 'high'])
export type TaskPriority = z.infer<typeof TaskPrioritySchema>

export const LocalDateSchema = z.string().regex(/^\d{4}-\d{2}-\d{2}$/)

export const TaskSchema = z.object({
  id: z.string(),
  space_id: z.string().nullable(),
  parent_task_id: z.string().nullable(),
  title: z.string(),
  description: z.string(),
  status: TaskStatusSchema,
  priority: TaskPrioritySchema,
  due_date: LocalDateSchema.nullable(),
  tags: z.array(z.string()),
  completed_at: z.string().nullable(),
  archived_at: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
})
export type Task = z.infer<typeof TaskSchema>

export const TaskInputSchema = z.object({
  spaceId: z.string().nullable(),
  parentTaskId: z.string().nullable(),
  title: z.string().trim().min(1).max(200),
  description: z.string().max(10_000),
  status: TaskStatusSchema,
  priority: TaskPrioritySchema,
  dueDate: LocalDateSchema.nullable(),
  tags: z.array(z.string().trim().min(1).max(40)).max(20),
})
export type TaskInput = z.infer<typeof TaskInputSchema>

export const TaskFilterSchema = z.object({
  spaceId: z.string().optional(),
  unassignedOnly: z.boolean().optional(),
  status: TaskStatusSchema.optional(),
  priority: TaskPrioritySchema.optional(),
  search: z.string().optional(),
  includeArchived: z.boolean().optional(),
  limit: z.number().int().min(1).max(500).optional(),
})
export type TaskFilter = z.infer<typeof TaskFilterSchema>

export const PulseTaskSchema = z.object({
  id: z.string(),
  title: z.string(),
  spaceId: z.string().nullable(),
  spaceName: z.string().nullable(),
  dueDate: LocalDateSchema,
  priority: TaskPrioritySchema,
  destination: z.string(),
})
export type PulseTask = z.infer<typeof PulseTaskSchema>

export const PulseSpaceSchema = z.object({
  id: z.string(),
  name: z.string(),
  reason: z.string(),
  lastWorkedAt: z.string(),
  destination: z.string(),
})
export type PulseSpace = z.infer<typeof PulseSpaceSchema>

export const PulseFileSchema = z.object({
  id: z.string(),
  title: z.string(),
  detail: z.string(),
  spaceId: z.string().nullable(),
  spaceName: z.string().nullable(),
  detectedAt: z.string(),
  destination: z.string(),
})
export type PulseFile = z.infer<typeof PulseFileSchema>

export const PulseSuggestionSchema = z.object({
  title: z.string(),
  detail: z.string(),
  destination: z.string(),
  sourceType: z.enum(['task', 'space', 'file', 'empty']),
  sourceId: z.string().nullable(),
})

export const PulseSnapshotSchema = z.object({
  today: LocalDateSchema,
  overdue: z.array(PulseTaskSchema),
  dueToday: z.array(PulseTaskSchema),
  upcoming: z.array(PulseTaskSchema),
  continueSpaces: z.array(PulseSpaceSchema),
  newFiles: z.array(PulseFileSchema),
  recentActivity: z.array(ActivityItemSchema),
  suggestedNextStep: PulseSuggestionSchema,
})
export type PulseSnapshot = z.infer<typeof PulseSnapshotSchema>

export const ActionRequestSchema = z.discriminatedUnion('type', [
  z.object({
    type: z.literal('createTask'),
    title: z.string().trim().min(1).max(200),
    description: z.string().max(10_000),
    dueDate: LocalDateSchema.nullable(),
    spaceId: z.string().nullable(),
  }),
  z.object({
    type: z.literal('createNote'),
    title: z.string().trim().min(1).max(200),
    content: z.string().max(20_000),
    spaceId: z.string().min(1),
  }),
  z.object({
    type: z.literal('createFolder'),
    sourceId: z.string().min(1),
    relativePath: z.string().trim().min(1).max(500),
  }),
  z.object({
    type: z.literal('copyFile'),
    sourceId: z.string().min(1),
    fromRelativePath: z.string().trim().min(1).max(500),
    toRelativePath: z.string().trim().min(1).max(500),
  }),
  z.object({
    type: z.literal('moveFile'),
    sourceId: z.string().min(1),
    fromRelativePath: z.string().trim().min(1).max(500),
    toRelativePath: z.string().trim().min(1).max(500),
  }),
  z.object({
    type: z.literal('renameFile'),
    sourceId: z.string().min(1),
    fromRelativePath: z.string().trim().min(1).max(500),
    newName: z.string().trim().min(1).max(255),
  }),
  z.object({
    type: z.literal('openFile'),
    sourceId: z.string().min(1),
    relativePath: z.string().trim().min(1).max(500),
  }),
  z.object({ type: z.literal('openFolder'), sourceId: z.string().min(1) }),
])
export type ActionRequest = z.infer<typeof ActionRequestSchema>
export const AiActionDraftSchema = z.object({
  index: z.number().int().nonnegative(),
  actionType: z.enum(['createTask', 'createNote']),
  title: z.string(),
  detail: z.string(),
})
export type AiActionDraft = z.infer<typeof AiActionDraftSchema>

const ActionTypeSchema = z.enum([
  'createTask',
  'createNote',
  'createFolder',
  'copyFile',
  'moveFile',
  'renameFile',
  'openFile',
  'openFolder',
])

export const ActionPreviewSchema = z.object({
  token: z.string(),
  actionType: ActionTypeSchema,
  title: z.string(),
  summary: z.string(),
  consequence: z.string(),
  expiresAt: z.string(),
})
export type ActionPreview = z.infer<typeof ActionPreviewSchema>

export const ActionResultSchema = z.object({
  actionType: ActionTypeSchema,
  title: z.string(),
  detail: z.string(),
  destination: z.string(),
  executedAt: z.string(),
})
export type ActionResult = z.infer<typeof ActionResultSchema>

// ─── Vault Types ─────────────────────────────────────────

export const VaultStorageModeSchema = z.enum(['linked', 'managed'])
export type VaultStorageMode = z.infer<typeof VaultStorageModeSchema>

export const VaultItemSchema = z.object({
  id: z.string(),
  space_id: z.string().nullable(),
  storage_mode: VaultStorageModeSchema,
  display_title: z.string(),
  original_name: z.string(),
  media_type: z.string(),
  size_bytes: z.number().int().nonnegative(),
  tags: z.array(z.string()),
  created_at: z.string(),
  updated_at: z.string(),
})
export type VaultItem = z.infer<typeof VaultItemSchema>

export const VaultImportInputSchema = z.object({
  path: z.string().min(1),
  storageMode: VaultStorageModeSchema,
  spaceId: z.string().nullable(),
  displayTitle: z.string().trim().min(1).max(200).optional(),
  tags: z.array(z.string().trim().min(1).max(40)).max(20),
})
export type VaultImportInput = z.infer<typeof VaultImportInputSchema>

export const VaultUpdateInputSchema = z.object({
  spaceId: z.string().nullable(),
  displayTitle: z.string().trim().min(1).max(200),
  tags: z.array(z.string().trim().min(1).max(40)).max(20),
})
export type VaultUpdateInput = z.infer<typeof VaultUpdateInputSchema>

export const VaultFilterSchema = z.object({
  spaceId: z.string().optional(),
  unassignedOnly: z.boolean().optional(),
  storageMode: VaultStorageModeSchema.optional(),
  search: z.string().optional(),
  limit: z.number().int().min(1).max(500).optional(),
})
export type VaultFilter = z.infer<typeof VaultFilterSchema>

// ─── Memory Types ───────────────────────────────────────

export const MemoryCategorySchema = z.enum([
  'preference',
  'decision',
  'recurring_context',
  'terminology',
  'goal',
  'constraint',
])
export type MemoryCategory = z.infer<typeof MemoryCategorySchema>

export const MemoryItemSchema = z.object({
  id: z.string(),
  space_id: z.string().nullable(),
  title: z.string(),
  content: z.string(),
  reason: z.string(),
  category: MemoryCategorySchema,
  source: z.literal('user'),
  created_at: z.string(),
  updated_at: z.string(),
})
export type MemoryItem = z.infer<typeof MemoryItemSchema>

export const MemoryInputSchema = z.object({
  spaceId: z.string().nullable(),
  title: z.string().trim().min(1).max(200),
  content: z.string().trim().min(1).max(20_000),
  reason: z.string().trim().min(1).max(500),
  category: MemoryCategorySchema,
})
export type MemoryInput = z.infer<typeof MemoryInputSchema>

export const MemoryFilterSchema = z.object({
  spaceId: z.string().optional(),
  globalOnly: z.boolean().optional(),
  category: MemoryCategorySchema.optional(),
  search: z.string().optional(),
  limit: z.number().int().min(1).max(500).optional(),
})
export type MemoryFilter = z.infer<typeof MemoryFilterSchema>

// ─── Native Desktop Types ───────────────────────────────

export const NativeStatusSchema = z.object({
  trayAvailable: z.boolean(),
  shortcut: z.string(),
  shortcutRegistered: z.boolean(),
  notificationsAvailable: z.boolean(),
  updaterConfigured: z.boolean(),
})
export type NativeStatus = z.infer<typeof NativeStatusSchema>

export const UpdatePhaseSchema = z.enum(['idle', 'checking', 'ready', 'installing'])
export const UpdateStatusSchema = z.object({
  configured: z.boolean(),
  channel: z.literal('Stable'),
  currentVersion: z.string(),
  phase: UpdatePhaseSchema,
})
export type UpdateStatus = z.infer<typeof UpdateStatusSchema>

export const UpdatePreviewSchema = z.object({
  token: z.string().min(1),
  currentVersion: z.string(),
  version: z.string(),
  notes: z.string().nullable(),
  publishedAt: z.string().nullable(),
  expiresAt: z.string(),
})
export type UpdatePreview = z.infer<typeof UpdatePreviewSchema>

export const UpdateProgressEventSchema = z.discriminatedUnion('event', [
  z.object({
    event: z.literal('started'),
    data: z.object({ contentLength: z.number().int().nonnegative().nullable() }),
  }),
  z.object({
    event: z.literal('progress'),
    data: z.object({
      downloaded: z.number().int().nonnegative(),
      chunkLength: z.number().int().nonnegative(),
    }),
  }),
  z.object({ event: z.literal('downloaded') }),
  z.object({ event: z.literal('verified') }),
  z.object({ event: z.literal('installing') }),
])
export type UpdateProgressEvent = z.infer<typeof UpdateProgressEventSchema>

export const BackupResultSchema = z.object({
  sizeBytes: z.number().nonnegative(),
  createdAt: z.string(),
})
export type BackupResult = z.infer<typeof BackupResultSchema>

export const BackupArchiveResultSchema = z.object({
  sizeBytes: z.number().nonnegative(),
  createdAt: z.string(),
  managedFileCount: z.number().int().nonnegative(),
  linkedFileCount: z.number().int().nonnegative(),
})
export type BackupArchiveResult = z.infer<typeof BackupArchiveResultSchema>

export const RestoreCountsSchema = z.object({
  spaces: z.number().int().nonnegative(),
  notes: z.number().int().nonnegative(),
  tasks: z.number().int().nonnegative(),
  memories: z.number().int().nonnegative(),
  conversations: z.number().int().nonnegative(),
})
export const RestorePreviewSchema = z.object({
  token: z.string(),
  createdAt: z.string(),
  appVersion: z.string(),
  archiveSizeBytes: z.number().nonnegative(),
  managedFileCount: z.number().int().nonnegative(),
  linkedFileCount: z.number().int().nonnegative(),
  expiresAt: z.string(),
  counts: RestoreCountsSchema,
})
export type RestorePreview = z.infer<typeof RestorePreviewSchema>

// ─── Context Sources ────────────────────────────────────

export const SourceSchema = z.object({
  id: z.string(),
  rootPath: z.string(),
  displayName: z.string(),
  spaceId: z.string().nullable(),
  scanStatus: z.enum(['never', 'scanning', 'complete', 'error']),
  lastScanAt: z.string().nullable(),
  lastError: z.string().nullable(),
  createdAt: z.string(),
  updatedAt: z.string(),
})
export type Source = z.infer<typeof SourceSchema>

export const IndexedFileSchema = z.object({
  id: z.string(),
  sourceId: z.string(),
  relativePath: z.string(),
  filename: z.string(),
  extension: z.string().nullable(),
  sizeBytes: z.number().int().nonnegative(),
  createdAtFs: z.number().int().nullable(),
  modifiedAtFs: z.number().int().nullable(),
  state: z.enum(['present', 'removed']),
  firstSeenAt: z.string(),
  lastSeenAt: z.string(),
  updatedAt: z.string(),
})
export type IndexedFile = z.infer<typeof IndexedFileSchema>

export const SourceScanResultSchema = z.object({
  sourceId: z.string(),
  scanned: z.number().int().nonnegative(),
  added: z.number().int().nonnegative(),
  changed: z.number().int().nonnegative(),
  renamed: z.number().int().nonnegative(),
  removed: z.number().int().nonnegative(),
  unchanged: z.number().int().nonnegative(),
  skipped: z.number().int().nonnegative(),
  errors: z.number().int().nonnegative(),
  truncated: z.boolean(),
  completedAt: z.string(),
})
export type SourceScanResult = z.infer<typeof SourceScanResultSchema>

// ─── Universal Search ──────────────────────────────────

export const UniversalSearchKindSchema = z.enum([
  'space',
  'note',
  'task',
  'vault',
  'memory',
  'conversation',
  'activity',
  'file',
])
export type UniversalSearchKind = z.infer<typeof UniversalSearchKindSchema>

export const UniversalSearchResultSchema = z.object({
  kind: UniversalSearchKindSchema,
  entityId: z.string(),
  spaceId: z.string().nullable(),
  title: z.string(),
  subtitle: z.string(),
  provenance: z.string(),
  score: z.number().int(),
  updatedAt: z.string(),
  sourceId: z.string().nullable(),
  relativePath: z.string().nullable(),
})
export type UniversalSearchResult = z.infer<typeof UniversalSearchResultSchema>

// ─── AI Types ────────────────────────────────────────────

export const AiConversationSchema = z.object({
  id: z.string(),
  space_id: z.string().nullable(),
  title: z.string(),
  provider: z.string(),
  model: z.string(),
  system_context_version: z.number(),
  archived_at: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
  last_opened_at: z.string().nullable(),
})
export type AiConversation = z.infer<typeof AiConversationSchema>

export const AiMessageSchema = z.object({
  id: z.string(),
  conversation_id: z.string(),
  role: z.enum(['user', 'assistant', 'system']),
  content: z.string(),
  status: z.enum(['pending', 'streaming', 'complete', 'error', 'cancelled']),
  provider_message_id: z.string().nullable(),
  error_code: z.string().nullable(),
  metadata_json: z.string().nullable(),
  provider: z.enum(['deepseek', 'openai']).nullable(),
  model: z.string().nullable(),
  routing_mode: z.enum(['auto', 'manual']).nullable(),
  route_reason: z.string().nullable(),
  created_at: z.string(),
  updated_at: z.string(),
})
export type AiMessage = z.infer<typeof AiMessageSchema>

export const AiContextItemSchema = z.object({
  id: z.string(),
  conversation_id: z.string(),
  entity_type: z.string(),
  entity_id: z.string(),
  context_mode: z.string(),
  added_at: z.string(),
})
export type AiContextItem = z.infer<typeof AiContextItemSchema>

export const AiModelSchema = z.object({
  id: z.string().min(1),
  displayName: z.string(),
  provider: z.enum(['deepseek', 'openai']),
  supportsStreaming: z.boolean(),
  supportsThinking: z.boolean(),
  supportsStructuredOutput: z.boolean(),
})
export type AiModel = z.infer<typeof AiModelSchema>

export const AiProviderSchema = z.object({
  id: z.enum(['deepseek', 'openai']),
  displayName: z.string(),
  remote: z.boolean(),
})
export type AiProvider = z.infer<typeof AiProviderSchema>

export const AiProviderStatusSchema = z.object({
  provider: z.enum(['deepseek', 'openai']),
  configured: z.boolean(),
  status: z.enum(['configured', 'missing', 'unavailable']),
})
export type AiProviderStatus = z.infer<typeof AiProviderStatusSchema>

export const AiResolvedContextItemSchema = z.object({
  attachmentId: z.string(),
  entityType: z.enum(['note', 'task', 'vault', 'memory']),
  entityId: z.string(),
  title: z.string(),
  detail: z.string(),
})
export type AiResolvedContextItem = z.infer<typeof AiResolvedContextItemSchema>

export const AiModeSchema = z.enum([
  'ask',
  'summarize',
  'explain',
  'plan',
  'rewrite',
  'create_tasks',
  'propose_actions',
])
export type AiMode = z.infer<typeof AiModeSchema>

export const AiStreamEventSchema = z.discriminatedUnion('event', [
  z.object({
    event: z.literal('started'),
    data: z.object({
      requestId: z.string(),
      userMessage: AiMessageSchema,
      assistantMessage: AiMessageSchema,
    }),
  }),
  z.object({ event: z.literal('delta'), data: z.object({ content: z.string() }) }),
  z.object({
    event: z.literal('complete'),
    data: z.object({ message: AiMessageSchema }),
  }),
  z.object({
    event: z.literal('cancelled'),
    data: z.object({ message: AiMessageSchema }),
  }),
  z.object({
    event: z.literal('failed'),
    data: z.object({
      code: z.string(),
      message: z.string(),
      assistantMessage: AiMessageSchema,
    }),
  }),
])
export type AiStreamEvent = z.infer<typeof AiStreamEventSchema>
