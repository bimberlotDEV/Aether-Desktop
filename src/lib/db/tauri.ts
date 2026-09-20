import { Channel, invoke } from '@tauri-apps/api/core'
import type {
  AppSetting,
  UserProfile,
  Space,
  ModuleInstance,
  SpaceWithDetails,
  ActivityItem,
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
  AiProvider,
  AiProviderStatus,
  AiResolvedContextItem,
  AiStreamEvent,
  MemoryFilter,
  MemoryInput,
  MemoryItem,
  NativeStatus,
  BetaDiagnosticReport,
  UpdateStatus,
  UpdatePreview,
  UpdateProgressEvent,
  BackupResult,
  BackupArchiveResult,
  RestorePreview,
  Source,
  IndexedFile,
  SourceScanResult,
  UniversalSearchResult,
  SpaceContinuity,
  PulseSnapshot,
  ActionPreview,
  ActionRequest,
  AiActionDraft,
  ActionResult,
} from './types'
import { BetaDiagnosticReportSchema } from './types'

// ─── Native desktop ─────────────────────────────────────
export async function getNativeStatus(): Promise<NativeStatus> {
  return invoke('native_get_status')
}
export async function getBetaDiagnostics(): Promise<BetaDiagnosticReport> {
  return BetaDiagnosticReportSchema.parse(await invoke('native_get_beta_diagnostics'))
}
export async function sendTestNotification(): Promise<void> {
  return invoke('native_test_notification')
}
export async function getUpdateStatus(): Promise<UpdateStatus> {
  return invoke('native_get_update_status')
}
export async function checkForUpdate(): Promise<UpdatePreview | null> {
  return invoke('native_check_for_update')
}
export async function cancelUpdate(token: string): Promise<boolean> {
  return invoke('native_cancel_update', { token })
}
export async function installUpdate(
  token: string,
  onEvent: (event: UpdateProgressEvent) => void,
): Promise<void> {
  const channel = new Channel<UpdateProgressEvent>()
  channel.onmessage = onEvent
  return invoke('native_install_update', { token, onEvent: channel })
}
export async function exportWorkspaceBackup(destination: string): Promise<BackupResult> {
  return invoke('export_workspace_backup', { destination })
}
export async function exportWorkspaceArchive(
  destination: string,
): Promise<BackupArchiveResult> {
  return invoke('export_workspace_archive', { destination })
}
export async function previewWorkspaceRestore(source: string): Promise<RestorePreview> {
  return invoke('preview_workspace_restore', { source })
}
export async function cancelWorkspaceRestore(token: string): Promise<boolean> {
  return invoke('cancel_workspace_restore', { token })
}
export async function approveWorkspaceRestore(token: string): Promise<void> {
  return invoke('approve_workspace_restore', { token })
}
export async function getPulse(): Promise<PulseSnapshot> {
  return invoke('get_pulse')
}
export async function previewAction(request: ActionRequest): Promise<ActionPreview> {
  return invoke('preview_action', { request })
}
export async function executeAction(token: string): Promise<ActionResult> {
  return invoke('execute_action', { token })
}
export async function cancelAction(token: string): Promise<boolean> {
  return invoke('cancel_action', { token })
}

// ─── Context Sources ────────────────────────────────────
export async function createSource(input: {
  rootPath: string
  displayName: string
  spaceId: string | null
}): Promise<Source> {
  return invoke('create_source', { input })
}
export async function listSources(): Promise<Source[]> {
  return invoke('list_sources')
}
export async function updateSourceSpace(
  id: string,
  spaceId: string | null,
): Promise<Source | null> {
  return invoke('update_source_space', { id, spaceId })
}
export async function revokeSource(id: string): Promise<boolean> {
  return invoke('revoke_source', { id })
}
export async function listIndexedFiles(
  sourceId: string,
  includeRemoved = false,
): Promise<IndexedFile[]> {
  return invoke('list_indexed_files', { sourceId, includeRemoved })
}
export async function scanSource(id: string): Promise<SourceScanResult> {
  return invoke('scan_source', { id })
}

export async function universalSearch(
  query: string,
  currentSpaceId: string | null = null,
  limit = 30,
): Promise<UniversalSearchResult[]> {
  return invoke('universal_search', { query, currentSpaceId, limit })
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
export async function initializeProfile(): Promise<UserProfile> {
  return invoke('initialize_profile')
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
export async function listAiModels(): Promise<AiModel[]> {
  return invoke('ai_list_models')
}
export async function parseAiActionProposals(
  conversationId: string,
  messageId: string,
): Promise<AiActionDraft[]> {
  return invoke('ai_parse_action_proposals', { conversationId, messageId })
}
export async function previewAiActionProposal(
  conversationId: string,
  messageId: string,
  index: number,
): Promise<ActionPreview> {
  return invoke('ai_preview_action_proposal', { conversationId, messageId, index })
}
export async function listAiProviders(): Promise<AiProvider[]> {
  return invoke('ai_list_providers')
}
export async function listAiProviderStatuses(): Promise<AiProviderStatus[]> {
  return invoke('ai_list_provider_statuses')
}
export async function setAiProviderApiKey(
  provider: AiProvider['id'],
  apiKey: string,
): Promise<void> {
  return invoke('ai_set_provider_api_key', { provider, apiKey })
}
export async function removeAiProviderApiKey(
  provider: AiProvider['id'],
): Promise<boolean> {
  return invoke('ai_remove_provider_api_key', { provider })
}
export async function testAiProviderConnection(
  provider: AiProvider['id'],
  model: string,
): Promise<string> {
  return invoke('ai_test_provider_connection', { provider, model })
}
export async function createAiConversation(params: {
  spaceId?: string
  title?: string
  provider?: 'auto' | 'deepseek' | 'openai'
  model?: string
}): Promise<AiConversation> {
  return invoke('ai_create_conversation', {
    spaceId: params.spaceId ?? null,
    title: params.title ?? null,
    provider: params.provider ?? null,
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
export async function listActivity(params?: {
  spaceId?: string
  limit?: number
}): Promise<ActivityItem[]> {
  return invoke('list_activity', {
    spaceId: params?.spaceId ?? null,
    limit: params?.limit ?? null,
  })
}

export async function getSpaceContinuity(spaceId: string): Promise<SpaceContinuity> {
  return invoke('get_space_continuity', { spaceId })
}
