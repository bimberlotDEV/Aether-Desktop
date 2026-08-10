import { describe, expect, it } from 'vitest'
import { parseTaskProposal } from '@/lib/aiProposal'

describe('AI Task proposal parser', () => {
  it('accepts a fenced JSON proposal and applies safe defaults', () => {
    expect(
      parseTaskProposal(`\`\`\`json
        {"tasks":[{"title":"Ship alpha"}]}
      \`\`\``),
    ).toEqual([
      {
        title: 'Ship alpha',
        description: '',
        priority: 'none',
        dueDate: null,
        tags: [],
      },
    ])
  })

  it('rejects malformed, excessive, and unsafe proposals', () => {
    expect(parseTaskProposal('not json')).toBeNull()
    expect(
      parseTaskProposal(
        JSON.stringify({
          tasks: Array.from({ length: 21 }, (_, index) => ({ title: `Task ${index}` })),
        }),
      ),
    ).toBeNull()
    expect(
      parseTaskProposal('{"tasks":[{"title":"Bad","priority":"urgent"}]}'),
    ).toBeNull()
  })
})
