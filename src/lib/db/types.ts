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

export const ActivityEventSchema = z.object({
  id: z.string(),
  event_type: z.string(),
  entity_type: z.string().nullable(),
  entity_id: z.string().nullable(),
  space_id: z.string().nullable(),
  metadata_json: z.string().nullable(),
  created_at: z.string(),
})
export type ActivityEvent = z.infer<typeof ActivityEventSchema>

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

export const BackupResultSchema = z.object({
  sizeBytes: z.number().nonnegative(),
  createdAt: z.string(),
})
export type BackupResult = z.infer<typeof BackupResultSchema>

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
  id: z.enum(['deepseek-v4-flash', 'deepseek-v4-pro']),
  provider: z.literal('deepseek'),
  supportsStreaming: z.boolean(),
  supportsThinking: z.boolean(),
})
export type AiModel = z.infer<typeof AiModelSchema>

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
])
export type AiMode = z.infer<typeof AiModeSchema>

export const KeyStatusSchema = z.object({
  configured: z.boolean(),
  status: z.enum(['configured', 'missing', 'unavailable']),
})
export type KeyStatus = z.infer<typeof KeyStatusSchema>

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
