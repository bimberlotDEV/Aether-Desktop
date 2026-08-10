import { Channel, invoke } from '@tauri-apps/api/core'
import type {
  AppSetting,
  UserProfile,
  Space,
  ModuleInstance,
  SpaceWithDetails,
  ActivityEvent,
  Note,
  NoteListItem,
  NoteSearchResult,
  Task,
  TaskFilter,
  TaskInput,
  VaultFilter,
  VaultImportInput,
  VaultItem,
  VaultUpdateInput,
  AiContextItem,
  AiConversation,
  AiMessage,
  AiMode,
  AiModel,
  AiResolvedContextItem,
  AiStreamEvent,
  KeyStatus,
  MemoryFilter,
  MemoryInput,
  MemoryItem,
  NativeStatus,
} from './types'

// ─── Native desktop ─────────────────────────────────────
export async function getNativeStatus(): Promise<NativeStatus> {
  return invoke('native_get_status')
}
export async function sendTestNotification(): Promise<void> {
  return invoke('native_test_notification')
}

// ─── Settings ────────────────────────────────────────────
export async function getSetting(key: string): Promise<AppSetting | null> {
  return invoke('get_setting', { key })
}
export async function setSetting(
  key: string,
  value: string,
  valueType?: string,
): Promise<void> {
  return invoke('set_setting', { key, value, valueType })
}
export async function deleteSetting(key: string): Promise<boolean> {
  return invoke('delete_setting', { key })
}
export async function listSettings(): Promise<AppSetting[]> {
  return invoke('list_settings')
}

// ─── User Profile ────────────────────────────────────────
export async function getProfile(): Promise<UserProfile | null> {
  return invoke('get_profile')
}
export async function createProfile(): Promise<UserProfile> {
  return invoke('create_profile')
}
export async function updateProfile(
  id: string,
  displayName?: string,
  onboardingCompleted?: boolean,
): Promise<UserProfile | null> {
  return invoke('update_profile', { id, displayName, onboardingCompleted })
}

// ─── Spaces ──────────────────────────────────────────────
export async function createSpace(params: {
  name: string
  description?: string
  icon?: string
  accent?: string
  templateType?: string
  parentSpaceId?: string
}): Promise<Space> {
  return invoke('create_space', {
    name: params.name,
    description: params.description ?? null,
    icon: params.icon ?? null,
    accent: params.accent ?? null,
    templateType: params.templateType ?? null,
    parentSpaceId: params.parentSpaceId ?? null,
  })
}

export async function createSpaceWithModules(params: {
  name: string
  description?: string
  icon?: string
  accent?: string
  templateType?: string
  parentSpaceId?: string
  moduleTypes: string[]
}): Promise<{ space: Space; modules: ModuleInstance[] }> {
  return invoke('create_space_with_modules', {
    name: params.name,
    description: params.description ?? null,
    icon: params.icon ?? null,
    accent: params.accent ?? null,
    templateType: params.templateType ?? null,
    parentSpaceId: params.parentSpaceId ?? null,
    moduleTypes: params.moduleTypes,
  })
}

export async function createSchoolSpace(params: {
  name: string
  description?: string
  icon?: string
  accent?: string
  moduleTypes: string[]
  subjects: { name: string; icon?: string; accent?: string }[]
}): Promise<{ school: Space; subjects: Space[] }> {
  return invoke('create_school_space', {
    name: params.name,
    description: params.description ?? null,
    icon: params.icon ?? null,
    accent: params.accent ?? null,
    moduleTypes: params.moduleTypes,
    subjects: params.subjects,
  })
}

export async function getSpace(id: string): Promise<SpaceWithDetails | null> {
  return invoke('get_space', { id })
}

export async function listSpaces(includeArchived?: boolean): Promise<Space[]> {
  return invoke('list_spaces', { includeArchived: includeArchived ?? false })
}

export async function listTopLevelSpaces(): Promise<Space[]> {
  return invoke('list_top_level_spaces')
}

export async function listChildSpaces(parentId: string): Promise<Space[]> {
  return invoke('list_child_spaces', { parentId })
}

export async function updateSpace(
  id: string,
  params: {
    name?: string
    description?: string | null
    icon?: string | null
    accent?: string | null
    settingsJson?: string | null
    parentSpaceId?: string | null
  },
): Promise<Space | null> {
  return invoke('update_space', {
    id,
    name: params.name ?? null,
    description: params.description === null ? '' : (params.description ?? null),
    icon: params.icon === null ? '' : (params.icon ?? null),
    accent: params.accent === null ? '' : (params.accent ?? null),
    settingsJson: params.settingsJson ?? null,
    parentSpaceId: params.parentSpaceId ?? null,
  })
}

export async function setSpaceModules(
  spaceId: string,
  moduleTypes: string[],
): Promise<ModuleInstance[]> {
  return invoke('set_space_modules', { spaceId, moduleTypes })
}

export async function getSpaceModules(spaceId: string): Promise<ModuleInstance[]> {
  return invoke('get_space_modules', { spaceId })
}

export async function archiveSpace(id: string): Promise<boolean> {
  return invoke('archive_space', { id })
}

export async function restoreSpace(id: string): Promise<boolean> {
  return invoke('restore_space', { id })
}

export async function deleteSpace(id: string): Promise<boolean> {
  return invoke('delete_space', { id })
}

export async function favouriteSpace(id: string, fav: boolean): Promise<boolean> {
  return invoke('favourite_space', { id, fav })
}

export async function reorderSpaces(ids: string[]): Promise<void> {
  return invoke('reorder_spaces', { ids })
}

export async function duplicateSpace(id: string): Promise<Space> {
  return invoke('duplicate_space', { id })
}

// ─── Notes ───────────────────────────────────────────────
export async function createNote(spaceId: string): Promise<Note> {
  return invoke('create_note', { spaceId })
}
export async function getNote(id: string): Promise<Note | null> {
  return invoke('get_note', { id })
}
export async function updateNote(
  id: string,
  params: {
    title?: string
    content?: string
    excerpt?: string
    expectedRevision?: number
  },
): Promise<Note | null> {
  return invoke('update_note', {
    id,
    title: params.title ?? null,
    content: params.content ?? null,
    excerpt: params.excerpt ?? null,
    expectedRevision: params.expectedRevision ?? null,
  })
}
export async function listNotesBySpace(spaceId: string): Promise<NoteListItem[]> {
  return invoke('list_notes_by_space', { spaceId })
}
export async function listRecentNotes(
  spaceId?: string,
  limit?: number,
): Promise<NoteListItem[]> {
  return invoke('list_recent_notes', { spaceId: spaceId ?? null, limit: limit ?? null })
}
export async function listPinnedNotes(spaceId?: string): Promise<NoteListItem[]> {
  return invoke('list_pinned_notes', { spaceId: spaceId ?? null })
}
export async function listArchivedNotes(spaceId?: string): Promise<NoteListItem[]> {
  return invoke('list_archived_notes', { spaceId: spaceId ?? null })
}
export async function searchNotes(
  query: string,
  spaceId?: string,
  limit?: number,
): Promise<NoteSearchResult[]> {
  return invoke('search_notes', { query, spaceId: spaceId ?? null, limit: limit ?? null })
}
export async function pinNote(id: string, pinned: boolean): Promise<boolean> {
  return invoke('pin_note', { id, pinned })
}
export async function archiveNote(id: string): Promise<boolean> {
  return invoke('archive_note', { id })
}
export async function restoreNote(id: string): Promise<boolean> {
  return invoke('restore_note', { id })
}
export async function deleteNote(id: string): Promise<boolean> {
  return invoke('delete_note', { id })
}
export async function moveNote(id: string, newSpaceId: string): Promise<boolean> {
  return invoke('move_note', { id, newSpaceId })
}
export async function duplicateNote(id: string): Promise<Note> {
  return invoke('duplicate_note', { id })
}

// ─── Tasks ───────────────────────────────────────────────
export async function createTask(input: TaskInput): Promise<Task> {
  return invoke('create_task', { input })
}
export async function createTasksBatch(inputs: TaskInput[]): Promise<Task[]> {
  return invoke('create_tasks_batch', { inputs })
}
export async function getTask(id: string): Promise<Task | null> {
  return invoke('get_task', { id })
}
export async function listTasks(filter?: TaskFilter): Promise<Task[]> {
  return invoke('list_tasks', { filter: filter ?? null })
}
export async function updateTask(id: string, input: TaskInput): Promise<Task | null> {
  return invoke('update_task', { id, input })
}
export async function listTaskAttention(
  today: string,
  horizon: string,
  limit?: number,
): Promise<Task[]> {
  return invoke('list_task_attention', { today, horizon, limit: limit ?? null })
}
export async function archiveTask(id: string): Promise<boolean> {
  return invoke('archive_task', { id })
}
export async function restoreTask(id: string): Promise<boolean> {
  return invoke('restore_task', { id })
}
export async function deleteTask(id: string): Promise<boolean> {
  return invoke('delete_task', { id })
}

// ─── Vault ───────────────────────────────────────────────
export async function importVaultItem(input: VaultImportInput): Promise<VaultItem> {
  return invoke('import_vault_item', {
    path: input.path,
    storageMode: input.storageMode,
    spaceId: input.spaceId,
    displayTitle: input.displayTitle ?? null,
    tags: input.tags,
  })
}
export async function getVaultItem(id: string): Promise<VaultItem | null> {
  return invoke('get_vault_item', { id })
}
export async function listVaultItems(filter?: VaultFilter): Promise<VaultItem[]> {
  return invoke('list_vault_items', { filter: filter ?? null })
}
export async function updateVaultItem(
  id: string,
  input: VaultUpdateInput,
): Promise<VaultItem | null> {
  return invoke('update_vault_item', { id, input })
}
export async function removeVaultItem(id: string): Promise<boolean> {
  return invoke('remove_vault_item', { id })
}
export async function openVaultItem(id: string): Promise<void> {
  return invoke('open_vault_item', { id })
}
export async function revealVaultItem(id: string): Promise<void> {
  return invoke('reveal_vault_item', { id })
}

// ─── Memory ─────────────────────────────────────────────
export async function createMemory(input: MemoryInput): Promise<MemoryItem> {
  return invoke('create_memory', { input })
}
export async function getMemory(id: string): Promise<MemoryItem | null> {
  return invoke('get_memory', { id })
}
export async function listMemory(filter?: MemoryFilter): Promise<MemoryItem[]> {
  return invoke('list_memory', { filter: filter ?? null })
}
export async function updateMemory(
  id: string,
  input: MemoryInput,
): Promise<MemoryItem | null> {
  return invoke('update_memory', { id, input })
}
export async function deleteMemory(id: string): Promise<boolean> {
  return invoke('delete_memory', { id })
}

// ─── AI ─────────────────────────────────────────────────
export async function getAiKeyStatus(): Promise<KeyStatus> {
  return invoke('ai_get_key_status')
}
export async function setAiApiKey(apiKey: string): Promise<void> {
  return invoke('ai_set_api_key', { apiKey })
}
export async function removeAiApiKey(): Promise<boolean> {
  return invoke('ai_remove_api_key')
}
export async function testAiConnection(): Promise<string> {
  return invoke('ai_test_connection')
}
export async function listAiModels(): Promise<AiModel[]> {
  return invoke('ai_list_models')
}
export async function createAiConversation(params: {
  spaceId?: string
  title?: string
  model?: string
}): Promise<AiConversation> {
  return invoke('ai_create_conversation', {
    spaceId: params.spaceId ?? null,
    title: params.title ?? null,
    model: params.model ?? null,
  })
}
export async function getAiConversation(id: string): Promise<AiConversation | null> {
  return invoke('ai_get_conversation', { id })
}
export async function listAiConversations(
  spaceId?: string,
  includeArchived = false,
): Promise<AiConversation[]> {
  return invoke('ai_list_conversations', {
    spaceId: spaceId ?? null,
    includeArchived,
  })
}
export async function updateAiConversation(
  id: string,
  params: { title?: string; archived?: boolean },
): Promise<AiConversation | null> {
  return invoke('ai_update_conversation', {
    id,
    title: params.title ?? null,
    archived: params.archived ?? null,
  })
}
export async function deleteAiConversation(id: string): Promise<boolean> {
  return invoke('ai_delete_conversation', { id })
}
export async function listAiMessages(
  conversationId: string,
  limit?: number,
): Promise<AiMessage[]> {
  return invoke('ai_list_messages', { conversationId, limit: limit ?? null })
}
export async function streamAiMessage(
  requestId: string,
  conversationId: string,
  content: string,
  onEvent: (event: AiStreamEvent) => void,
  retryUserMessageId?: string,
  mode: AiMode = 'ask',
): Promise<void> {
  const channel = new Channel<AiStreamEvent>()
  channel.onmessage = onEvent
  return invoke('ai_stream_message', {
    requestId,
    conversationId,
    content,
    retryUserMessageId: retryUserMessageId ?? null,
    mode,
    onEvent: channel,
  })
}
export async function cancelAiRequest(requestId: string): Promise<boolean> {
  return invoke('ai_cancel_request', { requestId })
}
export async function addAiContext(
  conversationId: string,
  entityType: 'note' | 'task' | 'vault' | 'memory',
  entityId: string,
): Promise<AiContextItem> {
  return invoke('ai_add_context', { conversationId, entityType, entityId })
}
export async function listAiContext(conversationId: string): Promise<AiContextItem[]> {
  return invoke('ai_list_context', { conversationId })
}
export async function resolveAiContext(
  conversationId: string,
): Promise<AiResolvedContextItem[]> {
  return invoke('ai_resolve_context', { conversationId })
}
export async function removeAiContext(id: string): Promise<boolean> {
  return invoke('ai_remove_context', { id })
}
export async function clearAiContext(conversationId: string): Promise<number> {
  return invoke('ai_clear_context', { conversationId })
}

// ─── Activity ────────────────────────────────────────────
export async function recordActivity(params: {
  eventType: string
  entityType?: string
  entityId?: string
  spaceId?: string
  metadataJson?: string
}): Promise<ActivityEvent> {
  return invoke('record_activity', {
    eventType: params.eventType,
    entityType: params.entityType ?? null,
    entityId: params.entityId ?? null,
    spaceId: params.spaceId ?? null,
    metadataJson: params.metadataJson ?? null,
  })
}

export async function listActivity(params?: {
  spaceId?: string
  limit?: number
}): Promise<ActivityEvent[]> {
  return invoke('list_activity', {
    spaceId: params?.spaceId ?? null,
    limit: params?.limit ?? null,
  })
}
