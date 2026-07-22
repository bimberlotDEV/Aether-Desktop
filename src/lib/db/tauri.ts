import { invoke } from '@tauri-apps/api/core'
import type { AppSetting, UserProfile, Space, ModuleInstance, SpaceWithDetails, ActivityEvent } from './types'

// ─── Settings ────────────────────────────────────────────
export async function getSetting(key: string): Promise<AppSetting | null> {
  return invoke('get_setting', { key })
}
export async function setSetting(key: string, value: string, valueType?: string): Promise<void> {
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
export async function updateProfile(id: string, displayName?: string, onboardingCompleted?: boolean): Promise<UserProfile | null> {
  return invoke('update_profile', { id, displayName, onboardingCompleted })
}

// ─── Spaces ──────────────────────────────────────────────
export async function createSpace(params: {
  name: string; description?: string; icon?: string; accent?: string;
  templateType?: string; parentSpaceId?: string;
}): Promise<Space> {
  return invoke('create_space', {
    name: params.name, description: params.description ?? null,
    icon: params.icon ?? null, accent: params.accent ?? null,
    templateType: params.templateType ?? null, parentSpaceId: params.parentSpaceId ?? null,
  })
}

export async function createSpaceWithModules(params: {
  name: string; description?: string; icon?: string; accent?: string;
  templateType?: string; parentSpaceId?: string; moduleTypes: string[];
}): Promise<{ space: Space; modules: ModuleInstance[] }> {
  return invoke('create_space_with_modules', {
    name: params.name, description: params.description ?? null,
    icon: params.icon ?? null, accent: params.accent ?? null,
    templateType: params.templateType ?? null, parentSpaceId: params.parentSpaceId ?? null,
    moduleTypes: params.moduleTypes,
  })
}

export async function createSchoolSpace(params: {
  name: string; description?: string; icon?: string; accent?: string;
  moduleTypes: string[];
  subjects: { name: string; icon?: string; accent?: string }[];
}): Promise<{ school: Space; subjects: Space[] }> {
  return invoke('create_school_space', {
    name: params.name, description: params.description ?? null,
    icon: params.icon ?? null, accent: params.accent ?? null,
    moduleTypes: params.moduleTypes, subjects: params.subjects,
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

export async function updateSpace(id: string, params: {
  name?: string; description?: string | null; icon?: string | null;
  accent?: string | null; settingsJson?: string | null; parentSpaceId?: string | null;
}): Promise<Space | null> {
  return invoke('update_space', {
    id, name: params.name ?? null, description: params.description ?? null,
    icon: params.icon ?? null, accent: params.accent ?? null,
    settingsJson: params.settingsJson ?? null, parentSpaceId: params.parentSpaceId ?? null,
  })
}

export async function setSpaceModules(spaceId: string, moduleTypes: string[]): Promise<ModuleInstance[]> {
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

// ─── Activity ────────────────────────────────────────────
export async function recordActivity(params: {
  eventType: string; entityType?: string; entityId?: string;
  spaceId?: string; metadataJson?: string;
}): Promise<ActivityEvent> {
  return invoke('record_activity', {
    eventType: params.eventType, entityType: params.entityType ?? null,
    entityId: params.entityId ?? null, spaceId: params.spaceId ?? null,
    metadataJson: params.metadataJson ?? null,
  })
}

export async function listActivity(params?: { spaceId?: string; limit?: number }): Promise<ActivityEvent[]> {
  return invoke('list_activity', { spaceId: params?.spaceId ?? null, limit: params?.limit ?? null })
}
