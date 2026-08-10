import { z } from 'zod'

const ProposalSchema = z.object({
  tasks: z
    .array(
      z.object({
        title: z.string().trim().min(1).max(200),
        description: z.string().max(10_000).optional().default(''),
        priority: z.enum(['none', 'low', 'medium', 'high']).optional().default('none'),
        dueDate: z
          .string()
          .regex(/^\d{4}-\d{2}-\d{2}$/)
          .nullable()
          .optional()
          .default(null),
        tags: z.array(z.string().trim().min(1).max(40)).max(20).optional().default([]),
      }),
    )
    .min(1)
    .max(20),
})

export type TaskProposal = z.infer<typeof ProposalSchema>['tasks']

export function parseTaskProposal(content: string): TaskProposal | null {
  const cleaned = content
    .trim()
    .replace(/^```(?:json)?\s*/i, '')
    .replace(/\s*```$/, '')
  try {
    const result = ProposalSchema.safeParse(JSON.parse(cleaned))
    return result.success ? result.data.tasks : null
  } catch {
    return null
  }
}
