import { invoke } from '@tauri-apps/api/core'
import type { AppSetting, UserProfile, Space, ActivityEvent } from './types'

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
  return invoke('update_profile', {
    id,
    displayName,
    onboardingCompleted,
  })
}

// ─── Spaces ──────────────────────────────────────────────

export async function createSpace(params: {
  name: string
  description?: string
  icon?: string
  accent?: string
  templateType?: string
}): Promise<Space> {
  return invoke('create_space', {
    name: params.name,
    description: params.description ?? null,
    icon: params.icon ?? null,
    accent: params.accent ?? null,
    templateType: params.templateType ?? null,
  })
}

export async function getSpace(id: string): Promise<Space | null> {
  return invoke('get_space', { id })
}

export async function listSpaces(
  includeArchived?: boolean,
): Promise<Space[]> {
  return invoke('list_spaces', { includeArchived: includeArchived ?? false })
}

export async function updateSpace(
  id: string,
  params: {
    name?: string
    description?: string | null
    icon?: string | null
    accent?: string | null
    settingsJson?: string | null
  },
): Promise<Space | null> {
  return invoke('update_space', {
    id,
    name: params.name ?? null,
    description: params.description ?? null,
    icon: params.icon ?? null,
    accent: params.accent ?? null,
    settingsJson: params.settingsJson ?? null,
  })
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

export async function favouriteSpace(
  id: string,
  fav: boolean,
): Promise<boolean> {
  return invoke('favourite_space', { id, fav })
}

export async function reorderSpaces(ids: string[]): Promise<void> {
  return invoke('reorder_spaces', { ids })
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
