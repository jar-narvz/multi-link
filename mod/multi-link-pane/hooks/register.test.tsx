import { expect, mock, test } from 'claude-code/testing'

import type { Engine, Mounted } from 'claude-code/testing'
import type { On, ProcessSpawnChunk, RenderElement, RenderNode } from 'claude-code'
import type { PaneReport } from '../types'

const PANE = {
  plugin: 'multi-link-pane', component: 'Pane', requestId: 'multi-link',
  props: { title: 'multi-link', isFocused: false, bodyColumns: 40,
    placement: 'dock', scroll: { offset: 0, bodyRows: 24 }, view: {} },
} as const

for (const surface of ['terminal', 'desktop'] as const) {
  test('lines are cut to the pane body, not the screen, on ' + surface, async ($, on) => {
    mock.clock(on)
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 150, rows: 44 }, props: { ...PANE.props, bodyColumns: 8, scroll: { offset: 0, bodyRows: 24 } } })
    for (const text of await texts(ui)) expect(Array.from(text).length <= 8).toBe(true)
    await ui.unmount()
  })

  test('reading before the first line on ' + surface, async ($, on) => {
    mock.clock(on)
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 40, rows: 24 }, props: { ...PANE.props, bodyColumns: 40, scroll: { offset: 0, bodyRows: 24 } } })
    expect((await ui.find({ type: 'Text', text: 'reading sessions…' }))?.text)
      .toBe('reading sessions…')
    await ui.unmount()
  })
}

// Verbatim pane answer keys (Phase 18 additive opens); the mod sandbox cannot import JSON files.
const keys: Record<string, PaneReport> = {
  "ambiguous": {
    "v": 1,
    "repo": "/home/dev/repo-r",
    "now": "2026-11-10T22:20:00.000Z",
    "this": "141a0000-0000-4000-8000-00000000141a",
    "seen": true,
    "strip": {
      "step": "5.1",
      "title": null,
      "parts": [
        {
          "harness": "claude",
          "kind": "plan",
          "status": "done",
          "active_min": 2,
          "is_this": true
        },
        {
          "harness": "codex",
          "kind": "implement",
          "status": null,
          "active_min": 1,
          "is_this": false
        },
        {
          "harness": "claude",
          "kind": "review",
          "status": null,
          "active_min": 4,
          "is_this": true
        }
      ]
    },
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
            "asked": "try step 5.1 on codex",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "check the bridge script",
            "said": "<said>",
            "openable": false,
            "opens": null,
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
            "asked": "try the bridge check again",
            "said": "<said>",
            "openable": false,
            "opens": null,
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
    "strip": {
      "step": "3.2",
      "title": null,
      "parts": [
        {
          "harness": "codex",
          "kind": "implement",
          "status": "running",
          "active_min": 54,
          "is_this": false
        }
      ]
    },
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
            "asked": "run the three road checks in parallel with codex",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": true,
            "children": [
              {
                "id": "14260000-0000-4000-8000-000000001426",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 59 min",
                "asked": "Read AGENTS.md, then do Step 3.1 in PLAN.md.",
                "said": null,
                "openable": false,
                "opens": null,
                "match": "exact"
              },
              {
                "id": "14250000-0000-4000-8000-000000001425",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 120 min",
                "asked": "Read AGENTS.md, then do Step 3.0 in PLAN.md.",
                "said": null,
                "openable": false,
                "opens": null,
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
            "asked": "run the roads checks by hand",
            "said": null,
            "openable": false,
            "opens": null,
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
    "strip": null,
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
            "asked": "Read AGENTS.md, then do Step 3.2 in PLAN.md.",
            "said": null,
            "openable": false,
            "opens": null,
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
    "strip": {
      "step": "2.4",
      "title": null,
      "parts": [
        {
          "harness": "claude",
          "kind": "plan",
          "status": "done",
          "active_min": 15,
          "is_this": true
        },
        {
          "harness": "codex",
          "kind": "implement",
          "status": "running",
          "active_min": 12,
          "is_this": false
        }
      ]
    },
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
            "asked": "add the feed rows, then hand step 2.4 to codex",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": true,
            "children": [
              {
                "id": "14220000-0000-4000-8000-000000001422",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 12 min",
                "asked": "Read AGENTS.md, then do Step 2.4 in PLAN.md.",
                "said": null,
                "openable": false,
                "opens": null,
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
            "asked": "pick up step 2.3 where the last session stopped",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [
              {
                "id": "14210000-0000-4000-8000-000000001421",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 9 min",
                "asked": "Read AGENTS.md, then do Step 2.3 in PLAN.md.",
                "said": "<said>",
                "openable": false,
                "opens": null,
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
            "asked": "split step 2.5 and 2.6 between two codex runs",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [
              {
                "id": "14280000-0000-4000-8000-000000001428",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 49 min",
                "asked": "Step 2.6: 表示幅で切る — cut every card line by display width, never by bytes",
                "said": null,
                "openable": false,
                "opens": null,
                "match": "exact"
              },
              {
                "id": "14290000-0000-4000-8000-000000001429",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 47 min",
                "asked": "Read AGENTS.md, then do Step 2.5 in PLAN.md.",
                "said": "<said>",
                "openable": false,
                "opens": null,
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
            "asked": "start step 2.3 with codex",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "nudge the step 2.5 run",
            "said": null,
            "openable": false,
            "opens": null,
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
    "strip": null,
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
            "asked": "Synthetic lane parent 4.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": true,
            "children": [
              {
                "id": "95c00004-0000-4000-8000-000095c00004",
                "harness": "codex",
                "state": "unknown",
                "phrase": "done 11:01am",
                "asked": "Tidy the synthetic README wording.",
                "said": "<text>",
                "openable": false,
                "opens": null,
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
            "asked": "Read AGENTS.md, then do Step 2.1 in PLAN.md. Synthetic import.",
            "said": "<text>",
            "openable": false,
            "opens": null,
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
            "asked": "Read AGENTS.md, then do Step 2.2 in PLAN.md. Synthetic export.",
            "said": "<text>",
            "openable": false,
            "opens": null,
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
    "strip": null,
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
    "strip": {
      "step": "2.5",
      "title": null,
      "parts": [
        {
          "harness": "claude",
          "kind": "plan",
          "status": "done",
          "active_min": 2,
          "is_this": false
        },
        {
          "harness": "codex",
          "kind": "implement",
          "status": "done",
          "active_min": 9,
          "is_this": false
        },
        {
          "harness": "claude",
          "kind": "review",
          "status": null,
          "active_min": 2,
          "is_this": false
        },
        {
          "harness": "claude",
          "kind": "plan",
          "status": "done",
          "active_min": 1,
          "is_this": true
        },
        {
          "harness": "codex",
          "kind": "implement",
          "status": null,
          "active_min": 1,
          "is_this": false
        },
        {
          "harness": "claude",
          "kind": "review",
          "status": null,
          "active_min": 1,
          "is_this": true
        },
        {
          "harness": "codex",
          "kind": "implement",
          "status": "running",
          "active_min": 46,
          "is_this": false
        }
      ]
    },
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
            "asked": "add the feed rows, then hand step 2.4 to codex",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [
              {
                "id": "14220000-0000-4000-8000-000000001422",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 12 min",
                "asked": "Read AGENTS.md, then do Step 2.4 in PLAN.md.",
                "said": null,
                "openable": false,
                "opens": null,
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
            "asked": "pick up step 2.3 where the last session stopped",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [
              {
                "id": "14210000-0000-4000-8000-000000001421",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 9 min",
                "asked": "Read AGENTS.md, then do Step 2.3 in PLAN.md.",
                "said": "<said>",
                "openable": false,
                "opens": null,
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
            "asked": "split step 2.5 and 2.6 between two codex runs",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [
              {
                "id": "14280000-0000-4000-8000-000000001428",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 49 min",
                "asked": "Step 2.6: 表示幅で切る — cut every card line by display width, never by bytes",
                "said": null,
                "openable": false,
                "opens": null,
                "match": "exact"
              },
              {
                "id": "14290000-0000-4000-8000-000000001429",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 47 min",
                "asked": "Read AGENTS.md, then do Step 2.5 in PLAN.md.",
                "said": "<said>",
                "openable": false,
                "opens": null,
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
            "asked": "nudge the step 2.5 run",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "start step 2.3 with codex",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [],
            "handoffs": []
          }
        ]
      }
    ]
  },
  "openable": {
    "v": 1,
    "repo": "/home/dev/repo-p9c",
    "now": "2026-11-09T06:00:00.000Z",
    "this": "92000002-0000-4000-8000-000092000002",
    "seen": true,
    "strip": null,
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
            "asked": "Synthetic exit case 12.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
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
            "asked": "Synthetic exit case 9.",
            "said": null,
            "openable": false,
            "opens": null,
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
            "id": "92000002-0000-4000-8000-000092000002",
            "harness": "claude",
            "label": "“Synthetic exit case 2.”",
            "step": null,
            "state": "finished",
            "phrase": "finished",
            "asked": "Synthetic exit case 2.",
            "said": null,
            "openable": true,
            "opens": "app",
            "is_this": true,
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
            "asked": "Synthetic exit case 11.",
            "said": null,
            "openable": false,
            "opens": null,
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
            "phrase": "idle · last active 19 min ago",
            "asked": "Synthetic exit case 10.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000008-0000-4000-8000-000092000008",
            "harness": "claude",
            "label": "“Synthetic exit case 8.”",
            "step": null,
            "state": "finished",
            "phrase": "finished",
            "asked": "Synthetic exit case 8.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000007-0000-4000-8000-000092000007",
            "harness": "claude",
            "label": "“Synthetic resumed prompt.”",
            "step": null,
            "state": "unknown",
            "phrase": "state unknown",
            "asked": "Synthetic exit case 7.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000006-0000-4000-8000-000092000006",
            "harness": "claude",
            "label": "“Synthetic exit case 6.”",
            "step": null,
            "state": "finished",
            "phrase": "finished",
            "asked": "Synthetic exit case 6.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000005-0000-4000-8000-000092000005",
            "harness": "claude",
            "label": "“Synthetic exit case 5.”",
            "step": null,
            "state": "finished",
            "phrase": "finished",
            "asked": "Synthetic exit case 5.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000004-0000-4000-8000-000092000004",
            "harness": "claude",
            "label": "“Synthetic exit case 4.”",
            "step": null,
            "state": "finished",
            "phrase": "finished",
            "asked": "Synthetic exit case 4.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000003-0000-4000-8000-000092000003",
            "harness": "claude",
            "label": "“Synthetic exit case 3.”",
            "step": null,
            "state": "finished",
            "phrase": "finished",
            "asked": "Synthetic exit case 3.",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [],
            "handoffs": []
          },
          {
            "id": "92000001-0000-4000-8000-000092000001",
            "harness": "claude",
            "label": "“Synthetic exit case 1.”",
            "step": null,
            "state": "finished",
            "phrase": "finished",
            "asked": "Synthetic exit case 1.",
            "said": null,
            "openable": false,
            "opens": null,
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
    "strip": null,
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
            "asked": "add the feed rows, then hand step 2.4 to codex",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [
              {
                "id": "14220000-0000-4000-8000-000000001422",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 12 min",
                "asked": "Read AGENTS.md, then do Step 2.4 in PLAN.md.",
                "said": null,
                "openable": false,
                "opens": null,
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
            "asked": "pick up step 2.3 where the last session stopped",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [
              {
                "id": "14210000-0000-4000-8000-000000001421",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 9 min",
                "asked": "Read AGENTS.md, then do Step 2.3 in PLAN.md.",
                "said": "<said>",
                "openable": false,
                "opens": null,
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
            "asked": "split step 2.5 and 2.6 between two codex runs",
            "said": null,
            "openable": false,
            "opens": null,
            "is_this": false,
            "children": [
              {
                "id": "14280000-0000-4000-8000-000000001428",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 49 min",
                "asked": "Step 2.6: 表示幅で切る — cut every card line by display width, never by bytes",
                "said": null,
                "openable": false,
                "opens": null,
                "match": "exact"
              },
              {
                "id": "14290000-0000-4000-8000-000000001429",
                "harness": "codex",
                "state": "busy",
                "phrase": "working now · for 47 min",
                "asked": "Read AGENTS.md, then do Step 2.5 in PLAN.md.",
                "said": "<said>",
                "openable": false,
                "opens": null,
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
            "asked": "start step 2.3 with codex",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "nudge the step 2.5 run",
            "said": null,
            "openable": false,
            "opens": null,
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
    "strip": null,
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
            "asked": "Synthetic exit case 12.",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "Synthetic exit case 9.",
            "said": null,
            "openable": false,
            "opens": null,
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
    "strip": null,
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
            "asked": "Synthetic exit case 12.",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "Synthetic exit case 9.",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "Synthetic exit case 14.",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "Synthetic exit case 13.",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "Synthetic exit case 11.",
            "said": null,
            "openable": false,
            "opens": null,
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
            "asked": "Synthetic exit case 10.",
            "said": null,
            "openable": false,
            "opens": null,
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

function setup(on: On, processRun?: (argv: readonly string[]) => { exitCode: number; stdout: string; stderr: string; isStdoutTruncated: boolean; isStderrTruncated: boolean }, host = 'Darwin') {
  const clock = mock.clock(on)
  on('process.run', (_, e) => {
    if (e.argv[0] !== 'uname') {
      if (processRun) return { value: processRun(e.argv) }
      throw new Error('unexpected process: ' + e.argv[0])
    }
    return { value: { exitCode: 0, stdout: host + '\n', stderr: '', isStdoutTruncated: false, isStderrTruncated: false } }
  })
  on('command.register', () => ({ value: { command: 'multi-link' } }))
  on('ui.open', () => ({ value: { isPlaced: true } }))
  on('session.id', () => ({ value: 'session-id' }))
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('session.end', (_, e) => ({ sessionId: e.sessionId }))
  return clock
}

function nodeText(node: RenderNode): string {
  if (typeof node === 'string') return node
  return 'children' in node ? (node.children ?? []).map(nodeText).join('') : ''
}

async function texts(ui: Mounted) {
  const lines = (node: RenderNode): string[] => {
    if (typeof node === 'string') return []
    if (node.type === 'Text' || (node.type === 'Box' && String(node.props?.key ?? '').startsWith('strip:')))
      return [nodeText(node)]
    return 'children' in node ? (node.children ?? []).flatMap(lines) : []
  }
  return lines(await ui.drawn())
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
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 100 }, props: { ...PANE.props, bodyColumns: 90, scroll: { offset: 0, bodyRows: 100 } } })
      const shown = await texts(ui)
      expect(shown[0]).toBe('multi-link · ' + report.repo.split('/').pop())
      for (const section of report.sections) {
        const visible = section.rows.length > 0 || (section.name === 'working' && !report.seen)
        expect(shown.includes((section.name === 'recent' ? 'recent (2h)' : section.name) + ' ─')).toBe(visible)
      }
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
      expect(shown[shown.length - 1]).toBe('checked 0s ago')
      expect(pipe.argv[0]).toEqual(['multi-link', 'pane', '--json', '--watch', '--cwd', '/repo', '--session', 'session-id', '--notify'])
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
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 100 }, props: { ...PANE.props, bodyColumns: 90, scroll: { offset: 0, bodyRows: 100 } } })
    expect(await texts(ui)).toContain('reading sessions…')
    pipe.chunk(line.slice(81) + '\n')
    await clock.settle()
    await clock.advance(3000)
    expect(await texts(ui)).toContain('checked 3s ago')
    pipe.chunk('{"v":1,"error":"read failed"}\nnot json\n{"v":2}\n')
    await clock.settle()
    expect((await texts(ui))[0]).toBe('multi-link · repo-m')
    expect((await texts(ui)).at(-1)).toBe('checked 0s ago · read failed')
    pipe.chunk(line + '\n')
    await clock.settle()
    expect((await texts(ui)).at(-1)).toBe('checked 0s ago')
    await end($)
    await ui.unmount()
  })

  for (const failure of ['missing', 'old', 'old-flag', 'exit'] as const) {
    test(failure + ' and retry on ' + surface, async ($, on) => {
      const clock = setup(on)
      const pipe = watcher(on)
      if (failure === 'missing') pipe.missing()
      await $.session.start({ cwd: '/repo', surface, isInteractive: true })
      if (failure !== 'missing') {
        if (failure === 'old') pipe.chunk("error: unrecognized subcommand 'pane'", 'stderr')
        if (failure === 'old-flag') pipe.chunk("error: unexpected argument '--notify' found", 'stderr')
        await clock.settle()
        pipe.end()
      }
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 24 }, props: { ...PANE.props, bodyColumns: 90, scroll: { offset: 0, bodyRows: 24 } } })
      const expected = failure === 'missing' ? "multi-link isn't on PATH. See the README."
        : failure === 'old' || failure === 'old-flag' ? 'This multi-link is too old for the pane. Update it.'
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
      report.strip = null // Phase 16 overflow regression, without the new strip.
      // Overflow consisting entirely of working sessions must still keep this.
      report.sections[0].rows = [...Array.from({ length: 20 }, (_, i) => ({ ...own,
        id: `other-${i}`, label: `other-${i}`, is_this: false, children: [] })), own]
      report.sections[2].rows = []
      pipe.chunk(JSON.stringify(report) + '\n')
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: columns, rows: rows }, props: { ...PANE.props, bodyColumns: columns, scroll: { offset: 0, bodyRows: rows } } })
      const shown = await texts(ui)
      expect(shown.length).toBe(rows)
      expect(shown[0]).toBe('multi-link · repo-m')
      expect(shown.some(line => line.startsWith('▌● this session'))).toBe(true)
      expect(shown).toContain(columns === 20 ? '▌  +4 runs' : '▌  +2 runs')
      expect(shown).toContain(columns === 20 ? '+20 more' : '+17 more')
      expect(shown.at(-1)).toBe('checked 0s ago')
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
        const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 100 }, props: { ...PANE.props, bodyColumns: 90, scroll: { offset: 0, bodyRows: 100 } } })
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
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 90, rows: 8 }, props: { ...PANE.props, bodyColumns: 90, scroll: { offset: 0, bodyRows: 8 } } })
    expect(await texts(ui)).toEqual([
      'multi-link · repo-p9c', 'working ─', ' ● worker', 'waiting ─',
      '▌● this session', ' ● “Synthetic exit case 9.”', '+4 more', 'checked 0s ago',
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
    pipe.chunk(JSON.stringify({ ...keys['children-exact-likely'], strip: null }) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 60, rows: 24 }, props: { ...PANE.props, bodyColumns: 60, scroll: { offset: 0, bodyRows: 24 } } })
    expect(await texts(ui)).toEqual([
      'multi-link · repo-m', 'working ─', '▌● this session  2.4',
      '▌ └ codex 14220000 ● likely', ' ● step 2.3', '  └ codex 14210000 ●',
      ' ● step 2.5', '  └ codex 14280000 ●', '  └ codex 14290000 ●',
      'recent (2h) ─', ' ● step 2.3', ' ● step 2.5', 'checked 0s ago',
    ])
    const own = await ui.find({ type: 'Text', text: 'this session' })
    expect(own?.props).toMatchObject({ color: '#df9b79', bold: true })
    const footer = await ui.find({ type: 'Text', text: 'checked' })
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

for (const surface of ['terminal', 'desktop'] as const) {
  test('strip matches every key on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 120, rows: 100 }, props: { ...PANE.props, bodyColumns: 120, scroll: { offset: 0, bodyRows: 100 } } })
    for (const report of Object.values(keys)) {
      pipe.chunk(JSON.stringify(report) + '\n')
      await clock.settle()
      const shown = await texts(ui)
      if (!report.strip) continue
      expect(shown).toContain('step ' + report.strip.step + (report.strip.title ? ' ' + report.strip.title : ''))
      let parts = report.strip.parts.slice(-4)
      const own = report.strip.parts.filter(p => p.is_this).at(-1)
      if (own && !parts.some(p => p.is_this)) parts = [own, ...parts.slice(1)]
      if (report.strip.parts.length > parts.length)
        expect(shown).toContain('+' + (report.strip.parts.length - parts.length) + ' earlier')
      for (const p of parts) {
        const glyph = p.status === 'done' ? '✓' : p.status === 'running' ? '●' : p.status === 'waiting' ? '◆' : '·'
        expect(shown).toContain((p.is_this ? '▌' : ' ') + p.harness.padEnd(6) + ' ' + p.kind.padEnd(10) + ' ' + glyph + ' ' + p.active_min + 'm')
      }
    }
    await end($)
    await ui.unmount()
  })

  test('row actions survive redraw and close when removed on ' + surface, async ($, on) => {
    const copies: unknown[] = []
    const commands: (readonly string[])[] = []
    const clock = setup(on, argv => {
      commands.push(argv)
      return { exitCode: 1, stdout: '', stderr: 'Session is no longer openable.', isStdoutTruncated: false, isStderrTruncated: false }
    })
    const pipe = watcher(on)
    on('ui.copy', (_, e) => { copies.push(e); return { value: { isCopied: true } } })
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(keys.openable) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 120, rows: 100 }, props: { ...PANE.props, bodyColumns: 120, scroll: { offset: 0, bodyRows: 100 } } })
    const id = keys.openable.this
    await ui.press({ key: id })
    expect(await texts(ui)).toContain('   asked Synthetic exit case 2.')
    expect(await texts(ui)).toContain('   said  —')
    await ui.press({ key: 'copy:' + id })
    expect(copies).toEqual([{ text: id, surface }])
    await ui.press({ key: 'open:' + id })
    expect(commands).toEqual([['multi-link', 'open', '--no-resume', id]])
    expect((await texts(ui)).at(-1)).toContain('Session is no longer openable.')
    pipe.chunk(JSON.stringify(keys.openable) + '\n')
    await clock.settle()
    expect((await ui.findAll({ type: 'Button', text: 'Copy id' })).length).toBe(1)
    const other = keys.openable.sections[1].rows[0].id
    await ui.press({ key: other })
    expect(await ui.find({ key: 'open:' + other })).toBeUndefined()
    expect(await ui.find({ key: 'copy:' + id })).toBeUndefined()
    await ui.press({ key: other })
    expect(await ui.find({ key: 'copy:' + other })).toBeUndefined()
    await ui.press({ key: id })
    pipe.chunk(JSON.stringify(keys.empty) + '\n')
    await clock.settle()
    pipe.chunk(JSON.stringify(keys.openable) + '\n')
    await clock.settle()
    expect(await ui.find({ key: 'copy:' + id })).toBeUndefined()
    await end($)
    await ui.unmount()
  })

  test('ticks reset checked age without replacing data on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(keys.busy) + '\n')
    await clock.settle()
    await clock.advance(2500)
    pipe.chunk('{"v":1,"tick":"2026-11-11T00:00:00Z"}\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 100, rows: 100 }, props: { ...PANE.props, bodyColumns: 100, scroll: { offset: 0, bodyRows: 100 } } })
    expect((await texts(ui)).at(-1)).toBe('checked 0s ago')
    expect((await texts(ui))[0]).toBe('multi-link · repo-p')
    await end($)
    await ui.unmount()
  })

  for (const notify of [true, false]) {
    test('notification option ' + notify + ' on ' + surface, { options: { notify } }, async ($, on) => {
      const calls: (readonly string[])[] = []
      const clock = setup(on, argv => {
        calls.push(argv)
        return { exitCode: 1, stdout: '', stderr: 'denied', isStdoutTruncated: false, isStderrTruncated: false }
      })
      const pipe = watcher(on)
      await $.session.start({ cwd: '/repo', surface, isInteractive: true })
      await clock.settle()
      expect(pipe.argv[0].includes('--notify')).toBe(notify)
      const event = { v: 1, event: { kind: 'codex_done', run: 'run-1', id: 'child', step: '17.3', said: 'Quotes " and $ and \n stay data' } }
      pipe.chunk(JSON.stringify(event) + '\n' + JSON.stringify(keys.busy) + '\n')
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 120, rows: 100 }, props: { ...PANE.props, bodyColumns: 120, scroll: { offset: 0, bodyRows: 100 } } })
      expect(calls).toEqual(notify ? [[
        'osascript', '-e', 'on run argv\n  display notification (item 2 of argv) with title (item 1 of argv)\nend run',
        'codex finished (17.3)', event.event.said,
      ]] : [])
      if (notify) expect((await texts(ui)).at(-1)).toContain('notification failed')
      pipe.chunk(JSON.stringify(event) + '\n')
      await clock.settle()
      expect(calls.length).toBe(notify ? 1 : 0)
      await end($)
      await ui.unmount()
    })
  }

  for (const [height, asked, said] of [[8, true, true], [7, true, false], [6, false, false]] as const) {
    test('open row overflow at ' + height + ' on ' + surface, async ($, on) => {
      const clock = setup(on)
      const pipe = watcher(on)
      const report = JSON.parse(JSON.stringify(keys.openable)) as PaneReport
      report.sections[1].rows = []
      report.sections[2].rows = report.sections[2].rows.filter(r => r.is_this)
      report.strip = { step: '17.3', title: 'The mod', parts: [] }
      await $.session.start({ cwd: '/repo', surface, isInteractive: true })
      pipe.chunk(JSON.stringify(report) + '\n')
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 100, rows: height }, props: { ...PANE.props, bodyColumns: 100, scroll: { offset: 0, bodyRows: height } } })
      await ui.press({ key: report.this })
      expect((await texts(ui)).some(t => t.startsWith('   asked'))).toBe(asked)
      expect((await texts(ui)).some(t => t.startsWith('   said'))).toBe(said)
      expect(await ui.find({ key: 'copy:' + report.this })).toBeDefined()
      expect(await ui.find({ key: 'open:' + report.this })).toBeDefined()
      await end($)
      await ui.unmount()
    })
  }
}

for (const surface of ['terminal', 'desktop'] as const) {
  test('four part cap keeps this sessions newest part on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    const report = JSON.parse(JSON.stringify(keys.busy)) as PaneReport
    report.strip = { step: '17.3', title: 'Synthetic strip', parts: Array.from({ length: 8 }, (_, i) => ({
      harness: i % 2 ? 'codex' : 'claude', kind: 'review', status: 'done', active_min: i + 1, is_this: i < 2,
    })) }
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(report) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 100, rows: 40 }, props: { ...PANE.props, bodyColumns: 100, scroll: { offset: 0, bodyRows: 40 } } })
    const shown = await texts(ui)
    expect(shown.slice(1, 7)).toEqual([
      'step 17.3 Synthetic strip', '+4 earlier', '▌codex  review     ✓ 2m',
      ' codex  review     ✓ 6m', ' claude review     ✓ 7m', ' codex  review     ✓ 8m',
    ])
    await end($)
    await ui.unmount()
  })

  test('short viewport keeps the expanded child and its buttons together on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    const report = JSON.parse(JSON.stringify(keys.busy)) as PaneReport
    report.strip = null
    report.sections[1].rows = []
    const own = report.sections[0].rows[0]
    own.children = Array.from({ length: 5 }, (_, i) => ({ ...own.children[0], id: 'child-' + i }))
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(report) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 60, rows: 20 }, props: { ...PANE.props, bodyColumns: 60, scroll: { offset: 0, bodyRows: 20 } } })
    await ui.press({ key: 'child-2' })
    expect(await ui.find({ key: 'open:child-2' })).toBeUndefined()
    await ui.unmount()
    const short = await $.ui.mount({ ...PANE, surface, viewport: { columns: 60, rows: 6 }, props: { ...PANE.props, bodyColumns: 60, scroll: { offset: 0, bodyRows: 6 } } })
    expect(await short.find({ key: 'child-2' })).toBeDefined()
    expect(await short.find({ key: 'copy:child-2' })).toBeDefined()
    expect(await texts(short)).toContain('▌  +4 runs')
    expect((await texts(short)).some(t => t.startsWith('   asked') || t.startsWith('   said'))).toBe(false)
    await end($)
    await short.unmount()
  })
}

for (const surface of ['terminal', 'desktop'] as const) {
  test('non macOS host never claims or displays notifications on ' + surface, async ($, on) => {
    const calls: (readonly string[])[] = []
    const clock = setup(on, argv => {
      calls.push(argv)
      return { exitCode: 0, stdout: '', stderr: '', isStdoutTruncated: false, isStderrTruncated: false }
    }, 'Linux')
    const pipe = watcher(on)
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    await clock.settle()
    expect(pipe.argv[0].includes('--notify')).toBe(false)
    pipe.chunk('{"v":1,"event":{"kind":"codex_done","run":"run","id":"id","step":null,"said":null}}\n')
    await clock.settle()
    expect(calls).toEqual([])
    await end($)
  })

  for (const failure of [false, true]) {
    test('notification fallback and spawn failure ' + failure + ' on ' + surface, async ($, on) => {
      const calls: (readonly string[])[] = []
      const clock = setup(on, argv => {
        calls.push(argv)
        if (failure) throw new Error('ENOENT')
        return { exitCode: 0, stdout: '', stderr: '', isStdoutTruncated: false, isStderrTruncated: false }
      })
      const pipe = watcher(on)
      await $.session.start({ cwd: '/repo', surface, isInteractive: true })
      const event = '{"v":1,"event":{"kind":"codex_done","run":"run","id":"id","step":null,"said":null}}\n'
      pipe.chunk(JSON.stringify(keys.busy) + '\n' + event)
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 120, rows: 40 }, props: { ...PANE.props, bodyColumns: 120, scroll: { offset: 0, bodyRows: 40 } } })
      expect(calls).toEqual([['osascript', '-e',
        'on run argv\n  display notification (item 2 of argv) with title (item 1 of argv)\nend run',
        'codex finished', 'no final message']])
      expect((await texts(ui)).at(-1)).toBe('checked 0s ago' + (failure ? ' · notification failed' : ''))
      pipe.end()
      await clock.settle()
      pipe.resume()
      await clock.advance(2000)
      pipe.chunk(event)
      await clock.settle()
      expect(calls.length).toBe(1)
      await end($)
      await ui.unmount()
    })
  }

  test('copy refusal is shown in the footer on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    on('ui.copy', () => ({ value: { isCopied: false, reason: 'unavailable' } }))
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(keys.openable) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 120, rows: 40 }, props: { ...PANE.props, bodyColumns: 120, scroll: { offset: 0, bodyRows: 40 } } })
    await ui.press({ key: keys.openable.this })
    await ui.press({ key: 'copy:' + keys.openable.this })
    expect((await texts(ui)).at(-1)).toBe('checked 0s ago · could not copy id')
    await end($)
    await ui.unmount()
  })
}

for (const surface of ['terminal', 'desktop'] as const) {
  test('S1 heading and part segments use board styles on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    const report = JSON.parse(JSON.stringify(keys.busy)) as PaneReport
    report.strip = { step: '17.3', title: 'Styled strip', parts: [
      { harness: 'codex', kind: 'implement', status: 'done', active_min: 9, is_this: true },
      { harness: 'claude', kind: 'review', status: 'running', active_min: 2, is_this: false },
      { harness: 'codex', kind: 'plan', status: null, active_min: 1, is_this: false },
      { harness: 'claude', kind: 'review', status: 'waiting', active_min: 3, is_this: false },
    ] }
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(report) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 100, rows: 40 }, props: { ...PANE.props, bodyColumns: 100, scroll: { offset: 0, bodyRows: 40 } } })
    const heading = await ui.find({ key: 'strip:heading' })
    expect(heading?.props.flexDirection).toBe('row')
    expect(heading?.text).toBe('step 17.3 Styled strip')
    const headingSegments = (heading?.children ?? []) as RenderElement[]
    expect(headingSegments.map(s => [nodeText(s), s.props?.color ?? null, s.props?.bold ?? false, s.props?.dimColor ?? false]))
      .toEqual([['step ', null, false, true], ['17.3', null, true, false], [' Styled strip', null, false, true]])
    for (const [index, harness, kind, glyph, color, minutes, own, dim] of [
      [0, 'codex', 'implement', '✓', 'success', 9, true, false],
      [1, 'claude', 'review', '●', 'warning', 2, false, false],
      [2, 'codex', 'plan', '·', null, 1, false, true],
      [3, 'claude', 'review', '◆', 'warning', 3, false, false],
    ] as const) {
      const part = await ui.find({ key: 'strip:part:' + index })
      expect(part?.props.flexDirection).toBe('row')
      const segments = (part?.children ?? []) as RenderElement[]
      expect(segments.map(s => [nodeText(s), s.props?.color ?? null, s.props?.bold ?? false, s.props?.dimColor ?? false]))
        .toEqual([
          [own ? '▌' : ' ', own ? '#df9b79' : null, own, false],
          [harness.padEnd(6) + ' ', harness === 'codex' ? '#8cb9f1' : '#df9b79', own, false],
          [kind.padEnd(10) + ' ', null, own, false],
          [glyph, color, own, dim],
          [' ' + minutes + 'm', null, own, true],
        ])
    }
    await end($)
    await ui.unmount()
  })

  for (const [columns, expectedHeading, expectedPart] of [
    [5, 'step…', '▌cod…'],
    [10, 'step 17.3…', '▌codex  i…'],
    [22, 'step 17.3 界界 title', '▌codex  implement  ✓ …'],
    [0, '', ''],
  ] as const) {
    test('strip segments share one column budget ' + columns + ' on ' + surface, async ($, on) => {
      const clock = setup(on)
      const pipe = watcher(on)
      const report = JSON.parse(JSON.stringify(keys.busy)) as PaneReport
      report.strip = { step: '17.3', title: '界界 title', parts: [
        { harness: 'codex', kind: 'implement', status: 'done', active_min: 99, is_this: true },
      ] }
      await $.session.start({ cwd: '/repo', surface, isInteractive: true })
      pipe.chunk(JSON.stringify(report) + '\n')
      await clock.settle()
      const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: columns, rows: 40 }, props: { ...PANE.props, bodyColumns: columns, scroll: { offset: 0, bodyRows: 40 } } })
      expect((await ui.find({ key: 'strip:heading' }))?.text).toBe(expectedHeading)
      expect((await ui.find({ key: 'strip:part:0' }))?.text).toBe(expectedPart)
      await end($)
      await ui.unmount()
    })
  }
}

for (const surface of ['terminal', 'desktop', 'vscode', 'mobile'] as const) {
  for (const opens of ['app', 'terminal', null] as const) {
    for (const exitCode of [0, 3]) {
      test('open destination ' + opens + ' exit ' + exitCode + ' on ' + surface, async ($, on) => {
        const commands: (readonly string[])[] = []
        const stdout = opens === 'terminal' ? 'jumped to Terminal · window 2 · tab 3 (ttys004)' : 'opened in app'
        const stderr = 'cannot jump: its tab is gone'
        const clock = setup(on, argv => {
          commands.push(argv)
          return { exitCode, stdout: stdout + '\n', stderr: stderr + '\n', isStdoutTruncated: false, isStderrTruncated: false }
        })
        const pipe = watcher(on)
        const data = JSON.parse(JSON.stringify(keys.openable)) as PaneReport
        const own = data.sections.flatMap(s => s.rows).find(r => r.is_this)!
        // Exercise the same action for a top-level row and an expanded child.
        Object.assign(own, { opens, openable: opens !== null })
        own.children = [{ id: 'child; $(echo data)', harness: 'codex', state: 'busy', phrase: 'working',
          asked: null, said: null, openable: opens !== null, opens, match: 'exact' }]
        await $.session.start({ cwd: '/repo', surface, isInteractive: true })
        pipe.chunk(JSON.stringify(data) + '\n')
        await clock.settle()
        const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 160, rows: 100 }, props: { ...PANE.props, bodyColumns: 160, scroll: { offset: 0, bodyRows: 100 } } })
        for (const id of [own.id, own.children[0].id]) {
          await ui.press({ key: id })
          const button = await ui.find({ key: 'open:' + id })
          if (opens === null) {
            expect(button).toBeUndefined()
          } else {
            expect(button?.text).toBe(opens === 'terminal' ? 'Jump to terminal' : 'Open in app')
            await ui.press({ key: 'open:' + id })
            expect(commands.at(-1)).toEqual(['multi-link', 'open', '--no-resume', id])
            expect((await texts(ui)).at(-1)).toBe('checked 0s ago · ' + (exitCode === 0 ? stdout : stderr))
          }
        }
        expect(commands.length).toBe(opens === null ? 0 : 2)
        await end($)
        await ui.unmount()
      })
    }
  }
}

for (const surface of ['terminal', 'desktop', 'vscode', 'mobile'] as const) {
  test('Phase 17 rows and children without opens still render on ' + surface, async ($, on) => {
    const clock = setup(on)
    const pipe = watcher(on)
    await $.session.start({ cwd: '/repo', surface, isInteractive: true })
    pipe.chunk(JSON.stringify(keys.busy, (key, value) => key === 'opens' ? undefined : value) + '\n')
    await clock.settle()
    const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 160, rows: 100 }, props: { ...PANE.props, bodyColumns: 160, scroll: { offset: 0, bodyRows: 100 } } })
    expect((await texts(ui))[0]).toBe('multi-link · repo-p')
    const own = keys.busy.sections.flatMap(s => s.rows).find(r => r.is_this)!
    await ui.press({ key: own.id })
    expect(await ui.find({ key: 'copy:' + own.id })).toBeDefined()
    await ui.press({ key: own.children[0].id })
    expect(await ui.find({ key: 'copy:' + own.children[0].id })).toBeDefined()
    // A missing field is compatible; an invalid destination must still be rejected.
    for (const invalid of [false, 1, {}, [], 'other']) {
      for (const child of [false, true]) {
        const data = JSON.parse(JSON.stringify(keys.busy)) as PaneReport
        data.repo = '/repo/rejected'
        const row = data.sections.flatMap(s => s.rows).find(r => r.is_this)!
        Object.assign(child ? row.children[0] : row, { opens: invalid })
        pipe.chunk(JSON.stringify(data) + '\n')
        await clock.settle()
        expect((await texts(ui))[0]).toBe('multi-link · repo-p')
      }
    }
    await end($)
    await ui.unmount()
  })

  for (const opens of ['app', 'terminal'] as const) {
    for (const exitCode of [0, 3]) {
      test('empty CLI output for ' + opens + ' exit ' + exitCode + ' on ' + surface, async ($, on) => {
        const clock = setup(on, () => ({ exitCode, stdout: ' \n', stderr: ' \n', isStdoutTruncated: false, isStderrTruncated: false }))
        const pipe = watcher(on)
        const data = JSON.parse(JSON.stringify(keys.openable)) as PaneReport
        const own = data.sections.flatMap(s => s.rows).find(r => r.is_this)!
        own.opens = opens
        await $.session.start({ cwd: '/repo', surface, isInteractive: true })
        pipe.chunk(JSON.stringify(data) + '\n')
        await clock.settle()
        const ui = await $.ui.mount({ ...PANE, surface, viewport: { columns: 160, rows: 100 }, props: { ...PANE.props, bodyColumns: 160, scroll: { offset: 0, bodyRows: 100 } } })
        await ui.press({ key: own.id })
        await ui.press({ key: 'open:' + own.id })
        const expected = exitCode !== 0 ? 'could not open session' : opens === 'app' ? 'opened in app' : 'jumped to terminal'
        expect((await texts(ui)).at(-1)).toBe('checked 0s ago · ' + expected)
        await end($)
        await ui.unmount()
      })
    }
  }
}
