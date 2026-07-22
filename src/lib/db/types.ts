import { z } from 'zod'

// ─── App Settings ────────────────────────────────────────

export const AppSettingSchema = z.object({
  key: z.string(),
  value: z.string(),
  value_type: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
})

export type AppSetting = z.infer<typeof AppSettingSchema>

// ─── User Profile ───────────────────────────────────────

export const UserProfileSchema = z.object({
  id: z.string(),
  display_name: z.string().nullable(),
  onboarding_completed: z.boolean(),
  created_at: z.string(),
  updated_at: z.string(),
})

export type UserProfile = z.infer<typeof UserProfileSchema>

// ─── Spaces ──────────────────────────────────────────────

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
  created_at: z.string(),
  updated_at: z.string(),
})

export type Space = z.infer<typeof SpaceSchema>

// ─── Activity Events ────────────────────────────────────

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
