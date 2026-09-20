import { beforeEach, describe, expect, it, vi } from 'vitest'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))

vi.mock('@tauri-apps/api/core', () => ({
  invoke,
  Channel: class {
    onmessage?: (message: unknown) => void
  },
}))

import {
  createSpaceWithModules,
  createTask,
  listTaskAttention,
  listTasks,
  updateNote,
  updateSpace,
  updateTask,
  importVaultItem,
  listVaultItems,
  updateVaultItem,
  removeVaultItem,
  openVaultItem,
  revealVaultItem,
  createAiConversation,
  streamAiMessage,
  addAiContext,
  cancelAiRequest,
  createMemory,
  listMemory,
  updateMemory,
  deleteMemory,
  getNativeStatus,
  getBetaDiagnostics,
  sendTestNotification,
  getUpdateStatus,
  checkForUpdate,
  cancelUpdate,
  installUpdate,
  exportWorkspaceBackup,
  exportWorkspaceArchive,
  previewWorkspaceRestore,
  cancelWorkspaceRestore,
  approveWorkspaceRestore,
  createSource,
  scanSource,
  revokeSource,
  universalSearch,
  listActivity,
  getSpaceContinuity,
  getPulse,
  previewAction,
  executeAction,
  cancelAction,
  listAiProviderStatuses,
  setAiProviderApiKey,
  testAiProviderConnection,
  parseAiActionProposals,
  previewAiActionProposal,
  initializeProfile,
} from '@/lib/db/tauri'

describe('Tauri database boundary', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockResolvedValue(null)
  })

  it('maps Space form values to the command argument contract', async () => {
    await createSpaceWithModules({
      name: 'Research',
      moduleTypes: ['notes', 'tasks'],
    })

    expect(invoke).toHaveBeenCalledWith('create_space_with_modules', {
      name: 'Research',
      description: null,
      icon: null,
      accent: null,
      templateType: null,
      parentSpaceId: null,
      moduleTypes: ['notes', 'tasks'],
    })
  })

  it('keeps native readiness and test notifications behind narrow commands', async () => {
    await getNativeStatus()
    await sendTestNotification()

    expect(invoke).toHaveBeenNthCalledWith(1, 'native_get_status')
    expect(invoke).toHaveBeenNthCalledWith(2, 'native_test_notification')
  })

  it('validates the closed beta diagnostic report returned by Rust', async () => {
    invoke.mockResolvedValueOnce({
      appVersion: '0.5.0',
      databaseSchema: '010_ai_route_provenance',
      databaseIntegrity: 'ok',
      platform: 'Windows x86_64',
      updaterConfigured: false,
      trayAvailable: true,
      shortcutRegistered: true,
      notificationsAvailable: true,
    })

    await expect(getBetaDiagnostics()).resolves.toMatchObject({ appVersion: '0.5.0' })
    expect(invoke).toHaveBeenCalledWith('native_get_beta_diagnostics')

    invoke.mockResolvedValueOnce({
      appVersion: '0.5.0',
      databaseSchema: '010_ai_route_provenance',
      databaseIntegrity: 'ok',
      platform: 'Windows x86_64',
      updaterConfigured: false,
      trayAvailable: true,
      shortcutRegistered: true,
      notificationsAvailable: true,
      localPath: 'C:\\private',
    })
    await expect(getBetaDiagnostics()).rejects.toThrow()
  })

  it('initializes first-run profile state entirely in Rust', async () => {
    await initializeProfile()

    expect(invoke).toHaveBeenCalledWith('initialize_profile')
  })

  it('keeps updater endpoints and installers out of the frontend command contract', async () => {
    await getUpdateStatus()
    await checkForUpdate()
    await cancelUpdate('pending-token')
    await installUpdate('approved-token', vi.fn())

    expect(invoke).toHaveBeenNthCalledWith(1, 'native_get_update_status')
    expect(invoke).toHaveBeenNthCalledWith(2, 'native_check_for_update')
    expect(invoke).toHaveBeenNthCalledWith(3, 'native_cancel_update', {
      token: 'pending-token',
    })
    expect(invoke).toHaveBeenNthCalledWith(4, 'native_install_update', {
      token: 'approved-token',
      onEvent: expect.any(Object),
    })
  })

  it('loads Pulse through one read-only command without frontend scope input', async () => {
    await getPulse()
    expect(invoke).toHaveBeenCalledWith('get_pulse')
  })

  it('keeps Action preview, approval, and cancellation behind opaque tokens', async () => {
    const request = {
      type: 'copyFile' as const,
      sourceId: 'source-1',
      fromRelativePath: 'in/report.pdf',
      toRelativePath: 'out/report.pdf',
    }
    await previewAction(request)
    await executeAction('proposal-token')
    await cancelAction('cancel-token')
    expect(invoke).toHaveBeenNthCalledWith(1, 'preview_action', { request })
    expect(invoke).toHaveBeenNthCalledWith(2, 'execute_action', {
      token: 'proposal-token',
    })
    expect(invoke).toHaveBeenNthCalledWith(3, 'cancel_action', {
      token: 'cancel-token',
    })
  })

  it('passes only the chosen destination to the backup command', async () => {
    await exportWorkspaceBackup('C:\\Backups\\workspace.aether-backup.db')

    expect(invoke).toHaveBeenCalledWith('export_workspace_backup', {
      destination: 'C:\\Backups\\workspace.aether-backup.db',
    })
  })

  it('keeps complete backup and restore behind paths plus opaque approval tokens', async () => {
    await exportWorkspaceArchive('C:\\Backups\\workspace.aether-backup')
    await previewWorkspaceRestore('C:\\Backups\\workspace.aether-backup')
    await cancelWorkspaceRestore('preview-token')
    await approveWorkspaceRestore('approval-token')

    expect(invoke).toHaveBeenNthCalledWith(1, 'export_workspace_archive', {
      destination: 'C:\\Backups\\workspace.aether-backup',
    })
    expect(invoke).toHaveBeenNthCalledWith(2, 'preview_workspace_restore', {
      source: 'C:\\Backups\\workspace.aether-backup',
    })
    expect(invoke).toHaveBeenNthCalledWith(3, 'cancel_workspace_restore', {
      token: 'preview-token',
    })
    expect(invoke).toHaveBeenNthCalledWith(4, 'approve_workspace_restore', {
      token: 'approval-token',
    })
  })

  it('keeps Source authorization and scanning behind narrow commands', async () => {
    await createSource({ rootPath: 'C:\\Work', displayName: 'Work', spaceId: null })
    await scanSource('source-1')
    await revokeSource('source-1')

    expect(invoke).toHaveBeenNthCalledWith(1, 'create_source', {
      input: { rootPath: 'C:\\Work', displayName: 'Work', spaceId: null },
    })
    expect(invoke).toHaveBeenNthCalledWith(2, 'scan_source', { id: 'source-1' })
    expect(invoke).toHaveBeenNthCalledWith(3, 'revoke_source', { id: 'source-1' })
  })

  it('reads curated Activity and Space continuity through narrow commands', async () => {
    await listActivity({ spaceId: 'space-1', limit: 20 })
    await getSpaceContinuity('space-1')

    expect(invoke).toHaveBeenNthCalledWith(1, 'list_activity', {
      spaceId: 'space-1',
      limit: 20,
    })
    expect(invoke).toHaveBeenNthCalledWith(2, 'get_space_continuity', {
      spaceId: 'space-1',
    })
  })

  it('passes Universal Search scope and limits through one narrow invoke', async () => {
    invoke.mockResolvedValueOnce([])
    await universalSearch('gradient descent', 'space-1', 25)
    expect(invoke).toHaveBeenCalledWith('universal_search', {
      query: 'gradient descent',
      currentSpaceId: 'space-1',
      limit: 25,
    })
  })

  it('maps cleared optional Space fields to empty persisted values', async () => {
    await updateSpace('space-1', {
      name: 'Renamed',
      description: null,
      icon: null,
      accent: '#6366f1',
    })

    expect(invoke).toHaveBeenCalledWith('update_space', {
      id: 'space-1',
      name: 'Renamed',
      description: '',
      icon: '',
      accent: '#6366f1',
      settingsJson: null,
      parentSpaceId: null,
    })
  })

  it('passes the optimistic revision used to protect Note autosave', async () => {
    await updateNote('note-1', {
      title: 'Title',
      content: 'Body',
      excerpt: 'Body',
      expectedRevision: 7,
    })

    expect(invoke).toHaveBeenCalledWith('update_note', {
      id: 'note-1',
      title: 'Title',
      content: 'Body',
      excerpt: 'Body',
      expectedRevision: 7,
    })
  })

  it('passes full-state Task inputs through the command boundary', async () => {
    const input = {
      spaceId: 'space-1',
      parentTaskId: null,
      title: 'Ship Phase 5',
      description: 'Complete the persistence slice',
      status: 'in_progress' as const,
      priority: 'high' as const,
      dueDate: '2026-08-12',
      tags: ['release'],
    }

    await createTask(input)
    await updateTask('task-1', input)

    expect(invoke).toHaveBeenNthCalledWith(1, 'create_task', { input })
    expect(invoke).toHaveBeenNthCalledWith(2, 'update_task', { id: 'task-1', input })
  })

  it('maps Task filters and attention dates without changing their semantics', async () => {
    await listTasks({
      spaceId: 'space-1',
      status: 'planned',
      priority: 'medium',
      search: 'release',
      includeArchived: false,
      limit: 50,
    })
    await listTaskAttention('2026-08-10', '2026-08-17', 12)

    expect(invoke).toHaveBeenNthCalledWith(1, 'list_tasks', {
      filter: {
        spaceId: 'space-1',
        status: 'planned',
        priority: 'medium',
        search: 'release',
        includeArchived: false,
        limit: 50,
      },
    })
    expect(invoke).toHaveBeenNthCalledWith(2, 'list_task_attention', {
      today: '2026-08-10',
      horizon: '2026-08-17',
      limit: 12,
    })
  })

  it('passes Vault ownership and ID-only native operations at the invoke boundary', async () => {
    const input = {
      path: 'C:\\Users\\Aether\\document.md',
      storageMode: 'managed' as const,
      spaceId: 'space-1',
      displayTitle: 'Research document',
      tags: ['research'],
    }
    await importVaultItem(input)
    await listVaultItems({ storageMode: 'linked', search: 'brief', limit: 20 })
    await updateVaultItem('vault-1', {
      spaceId: null,
      displayTitle: 'Updated title',
      tags: ['updated'],
    })
    await openVaultItem('vault-1')
    await revealVaultItem('vault-1')
    await removeVaultItem('vault-1')

    expect(invoke).toHaveBeenNthCalledWith(1, 'import_vault_item', {
      path: input.path,
      storageMode: 'managed',
      spaceId: 'space-1',
      displayTitle: 'Research document',
      tags: ['research'],
    })
    expect(invoke).toHaveBeenNthCalledWith(2, 'list_vault_items', {
      filter: { storageMode: 'linked', search: 'brief', limit: 20 },
    })
    expect(invoke).toHaveBeenNthCalledWith(3, 'update_vault_item', {
      id: 'vault-1',
      input: { spaceId: null, displayTitle: 'Updated title', tags: ['updated'] },
    })
    expect(invoke).toHaveBeenNthCalledWith(4, 'open_vault_item', { id: 'vault-1' })
    expect(invoke).toHaveBeenNthCalledWith(5, 'reveal_vault_item', { id: 'vault-1' })
    expect(invoke).toHaveBeenNthCalledWith(6, 'remove_vault_item', { id: 'vault-1' })
  })

  it('passes AI model, channel, cancellation, and explicit context boundaries', async () => {
    const onEvent = vi.fn()
    await createAiConversation({ spaceId: 'space-1', model: 'deepseek-v4-pro' })
    await streamAiMessage('request-1', 'conversation-1', 'Summarise this', onEvent)
    await cancelAiRequest('request-1')
    await addAiContext('conversation-1', 'note', 'note-1')

    expect(invoke).toHaveBeenNthCalledWith(1, 'ai_create_conversation', {
      spaceId: 'space-1',
      title: null,
      provider: null,
      model: 'deepseek-v4-pro',
    })
    expect(invoke).toHaveBeenNthCalledWith(
      2,
      'ai_stream_message',
      expect.objectContaining({
        requestId: 'request-1',
        conversationId: 'conversation-1',
        content: 'Summarise this',
        retryUserMessageId: null,
        mode: 'ask',
        onEvent: expect.objectContaining({ onmessage: onEvent }),
      }),
    )
    expect(invoke).toHaveBeenNthCalledWith(3, 'ai_cancel_request', {
      requestId: 'request-1',
    })
    expect(invoke).toHaveBeenNthCalledWith(4, 'ai_add_context', {
      conversationId: 'conversation-1',
      entityType: 'note',
      entityId: 'note-1',
    })
  })

  it('keeps provider credentials and AI drafts behind narrow commands', async () => {
    await listAiProviderStatuses()
    await setAiProviderApiKey('openai', 'sk-secret')
    await testAiProviderConnection('openai', 'gpt-5-mini')
    await parseAiActionProposals('conversation-1', 'message-1')
    await previewAiActionProposal('conversation-1', 'message-1', 0)

    expect(invoke).toHaveBeenNthCalledWith(1, 'ai_list_provider_statuses')
    expect(invoke).toHaveBeenNthCalledWith(2, 'ai_set_provider_api_key', {
      provider: 'openai',
      apiKey: 'sk-secret',
    })
    expect(invoke).toHaveBeenNthCalledWith(3, 'ai_test_provider_connection', {
      provider: 'openai',
      model: 'gpt-5-mini',
    })
    expect(invoke).toHaveBeenNthCalledWith(4, 'ai_parse_action_proposals', {
      conversationId: 'conversation-1',
      messageId: 'message-1',
    })
    expect(invoke).toHaveBeenNthCalledWith(5, 'ai_preview_action_proposal', {
      conversationId: 'conversation-1',
      messageId: 'message-1',
      index: 0,
    })
  })

  it('passes explicit Memory scope and content through typed boundaries', async () => {
    const input = {
      spaceId: 'space-1',
      title: 'Release constraint',
      content: 'Windows is the first supported platform.',
      reason: 'Keeps planning aligned.',
      category: 'constraint' as const,
    }
    await createMemory(input)
    await listMemory({ spaceId: 'space-1', category: 'constraint', search: 'Windows' })
    await updateMemory('memory-1', input)
    await deleteMemory('memory-1')

    expect(invoke).toHaveBeenNthCalledWith(1, 'create_memory', { input })
    expect(invoke).toHaveBeenNthCalledWith(2, 'list_memory', {
      filter: { spaceId: 'space-1', category: 'constraint', search: 'Windows' },
    })
    expect(invoke).toHaveBeenNthCalledWith(3, 'update_memory', {
      id: 'memory-1',
      input,
    })
    expect(invoke).toHaveBeenNthCalledWith(4, 'delete_memory', { id: 'memory-1' })
  })
})
