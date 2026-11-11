import { expect, mock, test } from 'claude-code/testing'

import type { Engine, Mounted } from 'claude-code/testing'
import type { On, ProcessSpawnChunk } from 'claude-code'
import type { PaneReport } from '../types'

const PANE = {
  plugin: 'multi-link-pane', component: 'Pane', requestId: 'multi-link',
  props: { title: 'multi-link', isFocused: false, bodyColumns: 40,
    placement: 'dock', scroll: { offset: 0, bodyRows: 24 }, view: {} },
} as const

for (const surface of ['terminal', 'desktop'] as const) {
  test('reading before the first line on ' + surface, async ($, on) => {
    mock.clock(on)
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 40, rows: 24 } })
    expect((await ui.find({ type: 'Text', text: 'reading sessions…' }))?.text)
      .toBe('reading sessions…')
    await ui.unmount()
  })
}

// Verbatim Step 16.0 answer keys; the mod sandbox cannot import JSON files.
const keys: Record<string, PaneReport> = {
  "ambiguous": {
    "v": 1,
    "repo": "/home/dev/repo-r",
    "now": "2026-11-10T22:20:00.000Z",
    "this": "141a0000-0000-4000-8000-00000000141a",
    "seen": true,
    "sections": [
      {
        "name": "working",
        "rows": []
      },
      {
        "name": "waiting",
        "rows": []
      },
      {
        "name": "recent",
        "rows": [
          {
            "id": "141a0000-0000-4000-8000-00000000141a",
            "harness": "claude",
            "label": "step 5.1",
            "step": "5.1",
            "state": "unknown",
            "phrase": "state unknown",
            "is_this": true,
            "children": [],
            "handoffs": [
              {
                "step": null,
                "outcome": "2 possible runs",
                "candidates": [
                  "142d0000-0000-4000-8000-00000000142d",
                  "142e0000-0000-4000-8000-00000000142e"
                ]
              }
            ]
          },
          {
            "id": "142e0000-0000-4000-8000-00000000142e",
            "harness": "codex",
            "label": "“check the bridge script”",
            "step": null,
            "state": "unknown",
            "phrase": "done 10:02pm",
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "142d0000-0000-4000-8000-00000000142d",
            "harness": "codex",
            "label": "“try the bridge check again”",
            "step": null,
            "state": "unknown",
            "phrase": "done 10:01pm",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      }
    ]
  },
  "busy": {
    "v": 1,
    "repo": "/home/dev/repo-p",
    "now": "2026-11-10T22:20:00.000Z",
    "this": "14150000-0000-4000-8000-000000001415",
    "seen": true,
    "sections": [
      {
        "name": "working",
        "rows": [
          {
            "id": "14150000-0000-4000-8000-000000001415",
            "harness": "claude",
            "label": "step 3.2",
            "step": "3.2",
            "state": "busy",
            "phrase": "working now · for 22 min",
            "is_this": true,
            "children": [
              {
                "id": "14260000-0000-4000-8000-000000001426",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 59 min",
                "match": "exact"
              },
              {
                "id": "14250000-0000-4000-8000-000000001425",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 120 min",
                "match": "exact"
              }
            ],
            "handoffs": []
          }
        ]
      },
      {
        "name": "waiting",
        "rows": [
          {
            "id": "14240000-0000-4000-8000-000000001424",
            "harness": "codex",
            "label": "“run the roads checks by hand”",
            "step": null,
            "state": "stale",
            "phrase": "stale since 9:50pm",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      },
      {
        "name": "recent",
        "rows": []
      }
    ]
  },
  "child-other-repo": {
    "v": 1,
    "repo": "/home/dev/repo-q",
    "now": "2026-11-10T22:20:00.000Z",
    "this": "14150000-0000-4000-8000-000000001415",
    "seen": false,
    "sections": [
      {
        "name": "working",
        "rows": [
          {
            "id": "14270000-0000-4000-8000-000000001427",
            "harness": "codex",
            "label": "step 3.2",
            "step": "3.2",
            "state": "busy",
            "phrase": "working now · for 54 min",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      },
      {
        "name": "waiting",
        "rows": []
      },
      {
        "name": "recent",
        "rows": []
      }
    ]
  },
  "children-exact-likely": {
    "v": 1,
    "repo": "/home/dev/repo-m",
    "now": "2026-11-10T22:20:00.000Z",
    "this": "14120000-0000-4000-8000-000000001412",
    "seen": true,
    "sections": [
      {
        "name": "working",
        "rows": [
          {
            "id": "14120000-0000-4000-8000-000000001412",
            "harness": "claude",
            "label": "step 2.4",
            "step": "2.4",
            "state": "busy",
            "phrase": "working now · for 28 min",
            "is_this": true,
            "children": [
              {
                "id": "14220000-0000-4000-8000-000000001422",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 12 min",
                "match": "likely"
              }
            ],
            "handoffs": []
          },
          {
            "id": "14110000-0000-4000-8000-000000001411",
            "harness": "claude",
            "label": "step 2.3",
            "step": "2.3",
            "state": "busy",
            "phrase": "working now · for 35 min",
            "is_this": false,
            "children": [
              {
                "id": "14210000-0000-4000-8000-000000001421",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 9 min",
                "match": "exact"
              }
            ],
            "handoffs": []
          },
          {
            "id": "14160000-0000-4000-8000-000000001416",
            "harness": "claude",
            "label": "step 2.5",
            "step": "2.5",
            "state": "busy",
            "phrase": "working now · for 20 min",
            "is_this": false,
            "children": [
              {
                "id": "14280000-0000-4000-8000-000000001428",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 49 min",
                "match": "exact"
              },
              {
                "id": "14290000-0000-4000-8000-000000001429",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 47 min",
                "match": "exact"
              }
            ],
            "handoffs": []
          }
        ]
      },
      {
        "name": "waiting",
        "rows": []
      },
      {
        "name": "recent",
        "rows": [
          {
            "id": "14170000-0000-4000-8000-000000001417",
            "harness": "claude",
            "label": "step 2.3",
            "step": "2.3",
            "state": "unknown",
            "phrase": "state unknown",
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "14180000-0000-4000-8000-000000001418",
            "harness": "claude",
            "label": "step 2.5",
            "step": "2.5",
            "state": "idle",
            "phrase": "idle · last active 47 min ago",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      }
    ]
  },
  "done-2h": {
    "v": 1,
    "repo": "/home/dev/repo-p9l",
    "now": "2026-11-09T12:00:00.000Z",
    "this": "95000004-0000-4000-8000-000095000004",
    "seen": true,
    "sections": [
      {
        "name": "working",
        "rows": []
      },
      {
        "name": "waiting",
        "rows": []
      },
      {
        "name": "recent",
        "rows": [
          {
            "id": "95000004-0000-4000-8000-000095000004",
            "harness": "claude",
            "label": "“Synthetic lane parent 4.”",
            "step": null,
            "state": "unknown",
            "phrase": "state unknown",
            "is_this": true,
            "children": [
              {
                "id": "95c00004-0000-4000-8000-000095c00004",
                "harness": "codex",
                "state": "unknown",
                "phrase": "done 11:01am",
                "match": "exact"
              }
            ],
            "handoffs": []
          },
          {
            "id": "95c00001-0000-4000-8000-000095c00001",
            "harness": "codex",
            "label": "step 2.1",
            "step": "2.1",
            "state": "unknown",
            "phrase": "done 11:30am",
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "95c00002-0000-4000-8000-000095c00002",
            "harness": "codex",
            "label": "step 7.1",
            "step": "7.1",
            "state": "unknown",
            "phrase": "done 10:31am",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      }
    ]
  },
  "empty": {
    "v": 1,
    "repo": "/home/dev/repo-none",
    "now": "2026-11-10T22:20:00.000Z",
    "this": "aaaaaaaa-0000-4000-8000-000000000000",
    "seen": false,
    "sections": [
      {
        "name": "working",
        "rows": []
      },
      {
        "name": "waiting",
        "rows": []
      },
      {
        "name": "recent",
        "rows": []
      }
    ]
  },
  "idle-recent": {
    "v": 1,
    "repo": "/home/dev/repo-m",
    "now": "2026-11-10T22:20:00.000Z",
    "this": "14180000-0000-4000-8000-000000001418",
    "seen": true,
    "sections": [
      {
        "name": "working",
        "rows": [
          {
            "id": "14120000-0000-4000-8000-000000001412",
            "harness": "claude",
            "label": "step 2.4",
            "step": "2.4",
            "state": "busy",
            "phrase": "working now · for 28 min",
            "is_this": false,
            "children": [
              {
                "id": "14220000-0000-4000-8000-000000001422",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 12 min",
                "match": "likely"
              }
            ],
            "handoffs": []
          },
          {
            "id": "14110000-0000-4000-8000-000000001411",
            "harness": "claude",
            "label": "step 2.3",
            "step": "2.3",
            "state": "busy",
            "phrase": "working now · for 35 min",
            "is_this": false,
            "children": [
              {
                "id": "14210000-0000-4000-8000-000000001421",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 9 min",
                "match": "exact"
              }
            ],
            "handoffs": []
          },
          {
            "id": "14160000-0000-4000-8000-000000001416",
            "harness": "claude",
            "label": "step 2.5",
            "step": "2.5",
            "state": "busy",
            "phrase": "working now · for 20 min",
            "is_this": false,
            "children": [
              {
                "id": "14280000-0000-4000-8000-000000001428",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 49 min",
                "match": "exact"
              },
              {
                "id": "14290000-0000-4000-8000-000000001429",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 47 min",
                "match": "exact"
              }
            ],
            "handoffs": []
          }
        ]
      },
      {
        "name": "waiting",
        "rows": []
      },
      {
        "name": "recent",
        "rows": [
          {
            "id": "14180000-0000-4000-8000-000000001418",
            "harness": "claude",
            "label": "step 2.5",
            "step": "2.5",
            "state": "idle",
            "phrase": "idle · last active 47 min ago",
            "is_this": true,
            "children": [],
            "handoffs": []
          },
          {
            "id": "14170000-0000-4000-8000-000000001417",
            "harness": "claude",
            "label": "step 2.3",
            "step": "2.3",
            "state": "unknown",
            "phrase": "state unknown",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      }
    ]
  },
  "unseen": {
    "v": 1,
    "repo": "/home/dev/repo-m",
    "now": "2026-11-10T22:20:00.000Z",
    "this": "aaaaaaaa-0000-4000-8000-000000000000",
    "seen": false,
    "sections": [
      {
        "name": "working",
        "rows": [
          {
            "id": "14120000-0000-4000-8000-000000001412",
            "harness": "claude",
            "label": "step 2.4",
            "step": "2.4",
            "state": "busy",
            "phrase": "working now · for 28 min",
            "is_this": false,
            "children": [
              {
                "id": "14220000-0000-4000-8000-000000001422",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 12 min",
                "match": "likely"
              }
            ],
            "handoffs": []
          },
          {
            "id": "14110000-0000-4000-8000-000000001411",
            "harness": "claude",
            "label": "step 2.3",
            "step": "2.3",
            "state": "busy",
            "phrase": "working now · for 35 min",
            "is_this": false,
            "children": [
              {
                "id": "14210000-0000-4000-8000-000000001421",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 9 min",
                "match": "exact"
              }
            ],
            "handoffs": []
          },
          {
            "id": "14160000-0000-4000-8000-000000001416",
            "harness": "claude",
            "label": "step 2.5",
            "step": "2.5",
            "state": "busy",
            "phrase": "working now · for 20 min",
            "is_this": false,
            "children": [
              {
                "id": "14280000-0000-4000-8000-000000001428",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 49 min",
                "match": "exact"
              },
              {
                "id": "14290000-0000-4000-8000-000000001429",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 47 min",
                "match": "exact"
              }
            ],
            "handoffs": []
          }
        ]
      },
      {
        "name": "waiting",
        "rows": []
      },
      {
        "name": "recent",
        "rows": [
          {
            "id": "14170000-0000-4000-8000-000000001417",
            "harness": "claude",
            "label": "step 2.3",
            "step": "2.3",
            "state": "unknown",
            "phrase": "state unknown",
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "14180000-0000-4000-8000-000000001418",
            "harness": "claude",
            "label": "step 2.5",
            "step": "2.5",
            "state": "idle",
            "phrase": "idle · last active 47 min ago",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      }
    ]
  },
  "waiting-quiet": {
    "v": 1,
    "repo": "/home/dev/repo-p9c",
    "now": "2026-11-09T12:00:00.000Z",
    "this": "92000012-0000-4000-8000-00009200000c",
    "seen": true,
    "sections": [
      {
        "name": "working",
        "rows": []
      },
      {
        "name": "waiting",
        "rows": [
          {
            "id": "92000012-0000-4000-8000-00009200000c",
            "harness": "claude",
            "label": "“Synthetic exit case 12.”",
            "step": null,
            "state": "waiting",
            "phrase": "waiting on you · since 6:02am",
            "is_this": true,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000009-0000-4000-8000-000092000009",
            "harness": "claude",
            "label": "“Synthetic exit case 9.”",
            "step": null,
            "state": "stale",
            "phrase": "stale since 5:30am",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      },
      {
        "name": "recent",
        "rows": []
      }
    ]
  },
  "waiting-stale": {
    "v": 1,
    "repo": "/home/dev/repo-p9c",
    "now": "2026-11-09T07:30:00.000Z",
    "this": "92000012-0000-4000-8000-00009200000c",
    "seen": true,
    "sections": [
      {
        "name": "working",
        "rows": []
      },
      {
        "name": "waiting",
        "rows": [
          {
            "id": "92000012-0000-4000-8000-00009200000c",
            "harness": "claude",
            "label": "“Synthetic exit case 12.”",
            "step": null,
            "state": "waiting",
            "phrase": "waiting on you · since 6:02am",
            "is_this": true,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000009-0000-4000-8000-000092000009",
            "harness": "claude",
            "label": "“Synthetic exit case 9.”",
            "step": null,
            "state": "stale",
            "phrase": "stale since 5:30am",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      },
      {
        "name": "recent",
        "rows": [
          {
            "id": "92000014-0000-4000-8000-00009200000e",
            "harness": "claude",
            "label": "“Synthetic exit case 14.”",
            "step": null,
            "state": "unknown",
            "phrase": "state unknown",
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000013-0000-4000-8000-00009200000d",
            "harness": "claude",
            "label": "“Synthetic resumed prompt, no reply yet.”",
            "step": null,
            "state": "unknown",
            "phrase": "state unknown",
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000011-0000-4000-8000-00009200000b",
            "harness": "claude",
            "label": "“Synthetic exit case 11.”",
            "step": null,
            "state": "finished",
            "phrase": "finished",
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000010-0000-4000-8000-00009200000a",
            "harness": "claude",
            "label": "“Synthetic exit case 10.”",
            "step": null,
            "state": "idle",
            "phrase": "idle · last active 5:40am",
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      }
    ]
  }
}

// Control the pipe independently of the clock: a live child can be silent.
function watcher(on: On) {
  let wake: (() => void) | undefined
  let pending: ProcessSpawnChunk[] = []
  let exit = false
  let running = 0
  let starts = 0
  let reject = false
  const argv: (readonly string[])[] = []
  on('process.spawn', async function* (_, e, next) {
    starts++
    argv.push(e.argv)
    if (reject) throw new Error('ENOENT')
    running++
    const abort = () => { exit = true; wake?.() }
    next.signal.addEventListener('abort', abort)
    try {
      while (!exit) {
        if (pending.length) yield pending.shift()!
        else await new Promise<void>(resolve => { wake = resolve })
      }
      return { value: { code: 1, signal: null } }
    } finally {
      running--
      next.signal.removeEventListener('abort', abort)
    }
  })
  return {
    chunk(text: string, stream: 'stdout' | 'stderr' = 'stdout') {
      pending.push({ stream, text }); wake?.()
    },
    end() { exit = true; wake?.() },
    missing() { reject = true },
    resume() { exit = false },
    starts: () => starts, running: () => running, argv,
  }
}

function setup(on: On) {
  const clock = mock.clock(on)
  on('command.register', () => ({ value: { command: 'multi-link' } }))
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('session.id', () => ({ value: 'session-id' }))
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('session.end', (_, e) => ({ sessionId: e.sessionId }))
  return clock
}

async function texts(ui: Mounted) {
  return (await ui.findAll({ type: 'Text' })).map(row => row.text)
}

async function end($: Engine) {
  await $.session.end({ sessionId: 'session-id', reason: 'prompt_input_exit' })
}

for (const surface of ['terminal', 'desktop'] as const) {
  for (const [name, report] of Object.entries(keys)) {
    test(name + ' watcher line on ' + surface, async ($, on) => {
      const clock = setup(on)
      const pipe = watcher(on)
      await $.session.start({ cwd: '/repo', surface, isInteractive: true })
      pipe.chunk(JSON.stringify(report) + '\n')
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 100 } })
      const shown = await texts(ui)
      expect(shown[0]).toBe('multi-link · ' + report.repo.split('/').pop())
      expect(shown).toContain('working ─')
      expect(shown).toContain('waiting ─')
      expect(shown).toContain('recent (2h) ─')
      for (const section of report.sections) {
        for (const row of section.rows) {
          const text = shown.find(line => line.includes(row.is_this ? 'this session' : row.label))
          expect(text).toBeDefined()
          if (row.is_this) expect(text).toContain('▌')
          for (const child of row.children) {
            const nested = shown.find(line => line.includes(child.id.slice(0, 8)))
            expect(nested).toContain('└ codex ')
            if (child.match === 'likely') expect(nested).toContain('likely')
            const element = await ui.find({ type: 'Text', text: child.id.slice(0, 8) })
            expect(element?.props.color).toBe('#8cb9f1')
          }
          for (const handoff of row.handoffs) expect(shown.join('\n')).toContain(handoff.outcome)
        }
      }
      if (!report.seen) expect(shown).toContain('▌ this session · not seen yet')
      if (name === 'empty') expect(shown).toContain('No sessions in this repo in the last 2 h.')
      expect(shown[shown.length - 1]).toBe('updated 0s ago')
      expect(pipe.argv[0]).toEqual(['multi-link', 'pane', '--json', '--watch', '--cwd', '/repo', '--session', 'session-id'])
      await end($)
      expect(pipe.running()).toBe(0)
      await ui.unmount()
    })
  }

  test('split lines, errors, heartbeat and footer age on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    const line = JSON.stringify(keys['children-exact-likely'])
    pipe.chunk(line.slice(0, 81))
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 100 } })
    expect(await texts(ui)).toContain('reading sessions…')
    pipe.chunk(line.slice(81) + '\n')
    await clock.settle()
    await clock.advance(3000)
    expect(await texts(ui)).toContain('updated 3s ago')
    pipe.chunk('{"v":1,"error":"read failed"}\nnot json\n{"v":2}\n')
    await clock.settle()
    expect((await texts(ui))[0]).toBe('multi-link · repo-m')
    expect((await texts(ui)).at(-1)).toBe('updated 3s ago · read failed')
    pipe.chunk(line + '\n')
    await clock.settle()
    expect((await texts(ui)).at(-1)).toBe('updated 0s ago')
    await end($)
    await ui.unmount()
  })

  for (const failure of ['missing', 'old', 'exit'] as const) {
    test(failure + ' and retry on ' + surface, async ($, on) => {
      const clock = setup(on)
      const pipe = watcher(on)
      if (failure === 'missing') pipe.missing()
      await $.session.start({ cwd: '/repo', surface, isInteractive: true })
      if (failure !== 'missing') {
        if (failure === 'old') pipe.chunk("error: unrecognized subcommand 'pane'", 'stderr')
        await clock.settle()
        pipe.end()
      }
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 24 } })
      const expected = failure === 'missing' ? "multi-link isn't on PATH. See the README."
        : failure === 'old' ? 'This multi-link is too old for the pane. Update it.'
        : 'watcher stopped, retrying'
      expect((await texts(ui)).join('\n')).toContain(expected)
      if (failure === 'exit') {
        for (const [delay, count] of [[2000, 2], [4000, 3], [8000, 4], [16000, 5], [32000, 6], [60000, 7], [60000, 8]]) {
          await clock.advance(delay - 1)
          expect(pipe.starts()).toBe(count - 1)
          await clock.advance(1)
          expect(pipe.starts()).toBe(count)
        }
      } else {
        await clock.advance(60000)
        expect(pipe.starts()).toBe(1)
      }
      await end($)
      const starts = pipe.starts()
      await clock.advance(120000)
      expect(pipe.starts()).toBe(starts)
      expect(pipe.running()).toBe(0)
      await ui.unmount()
    })
  }

  for (const [columns, rows] of [[20, 6], [30, 12]]) {
    test(`overflow ${columns}x${rows} on ${surface}`, async ($, on) => {
      const clock = setup(on)
      const pipe = watcher(on)
      await $.session.start({ cwd: '/repo', surface, isInteractive: true })
      const report = JSON.parse(JSON.stringify(keys['children-exact-likely'])) as PaneReport
      const own = report.sections[0].rows[0]
      own.children = Array.from({ length: 5 }, (_, i) => ({ ...own.children[0], id: `child00${i}` }))
      // Overflow consisting entirely of working sessions must still keep this.
      report.sections[0].rows = [...Array.from({ length: 20 }, (_, i) => ({ ...own,
        id: `other-${i}`, label: `other-${i}`, is_this: false, children: [] })), own]
      report.sections[2].rows = []
      pipe.chunk(JSON.stringify(report) + '\n')
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns, rows } })
      const shown = await texts(ui)
      expect(shown.length).toBe(rows)
      expect(shown[0]).toBe('multi-link · repo-m')
      expect(shown.some(line => line.startsWith('▌● this session'))).toBe(true)
      expect(shown).toContain(columns === 20 ? '▌  +4 runs' : '▌  +2 runs')
      expect(shown).toContain(columns === 20 ? '+20 more' : '+17 more')
      expect(shown.at(-1)).toBe('updated 0s ago')
      expect(shown.every(line => [...line].length <= columns)).toBe(true)
      await end($)
      await ui.unmount()
    })
  }
}

for (const surface of ['terminal', 'desktop'] as const) {
  for (const phase of ['streaming', 'backoff'] as const) {
      test(`session.end during ${phase} cancels child and timers on ${surface}`, async ($, on) => {
        const clock = setup(on)
        const pipe = watcher(on)
        await $.session.start({ cwd: '/repo', surface, isInteractive: true })
        pipe.chunk(JSON.stringify(keys.busy) + '\n')
        await clock.settle()
        if (phase === 'backoff') { pipe.end(); await clock.settle() }
        const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 100 } })
        const before = await texts(ui)
        await end($)
        expect(pipe.running()).toBe(0)
        await clock.advance(120000)
        expect(pipe.starts()).toBe(1)
        expect(await texts(ui)).toEqual(before)
        await ui.unmount()
      })
  }

  test('the command reopens the pane without starting a second watcher on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    expect(await $.command.run({ command: 'multi-link', args: '' }))
      .toMatchObject({ text: 'multi-link pane opened.' })
    await clock.settle()
    expect(pipe.starts()).toBe(1)
    await end($)
  })

  test('overflow removes recent before waiting before working on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    const report = JSON.parse(JSON.stringify(keys['waiting-stale'])) as PaneReport
    report.sections[0].rows = [{ ...report.sections[1].rows[1], id: 'worker', label: 'worker', state: 'busy' }]
    // At 8 rows: header, working heading + worker, waiting heading + this,
    // the first remaining waiting row, summary, footer. All recent rows yield.
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(report) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 8 } })
    expect(await texts(ui)).toEqual([
      'multi-link · repo-p9c', 'working ─', ' ● worker', 'waiting ─',
      '▌● this session', ' ● “Synthetic exit case 9.”', '+4 more', 'updated 0s ago',
    ])
    await end($)
    await ui.unmount()
  })
}

for (const surface of ['terminal', 'desktop'] as const) {
  test('board C text and harness colors on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(keys['children-exact-likely']) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 60, rows: 24 } })
    expect(await texts(ui)).toEqual([
      'multi-link · repo-m', 'working ─', '▌● this session  2.4',
      '▌ └ codex 14220000 ● likely', ' ● step 2.3', '  └ codex 14210000 ●',
      ' ● step 2.5', '  └ codex 14280000 ●', '  └ codex 14290000 ●',
      'waiting ─', 'recent (2h) ─', ' ● step 2.3', ' ● step 2.5', 'updated 0s ago',
    ])
    const own = await ui.find({ type: 'Text', text: 'this session' })
    expect(own?.props).toMatchObject({ color: '#df9b79', bold: true })
    const footer = await ui.find({ type: 'Text', text: 'updated' })
    expect(footer?.props.dimColor).toBe(true)
    pipe.chunk(JSON.stringify(keys['done-2h']) + '\n')
    await clock.settle()
    expect((await texts(ui)).join('\n')).toContain('✓')
    await end($)
    await ui.unmount()
  })

  test('data after a retry resets the backoff on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.end()
    await clock.settle()
    await clock.advance(2000)
    expect(pipe.starts()).toBe(2)
    pipe.resume()
    await clock.advance(4000)
    expect(pipe.starts()).toBe(3)
    pipe.chunk(JSON.stringify(keys.busy) + '\n')
    await clock.settle()
    pipe.end()
    await clock.settle()
    await clock.advance(1999)
    expect(pipe.starts()).toBe(3)
    await clock.advance(1)
    expect(pipe.starts()).toBe(4)
    await end($)
  })
}
