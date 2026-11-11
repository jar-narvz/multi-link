export type PaneChild = {
  id: string
  harness: 'claude' | 'codex'
  state: string
  phrase: string
  match: 'exact' | 'likely'
}
export type PaneRow = {
  id: string
  harness: 'claude' | 'codex'
  label: string
  step: string | null
  state: string
  phrase: string
  is_this: boolean
  children: PaneChild[]
  handoffs: { step: string | null; outcome: string; candidates: string[] }[]
}
export type PaneReport = {
  v: 1
  repo: string
  now: string
  this: string
  seen: boolean
  sections: { name: 'working' | 'waiting' | 'recent'; rows: PaneRow[] }[]
}
export type PaneState = {
  report: PaneReport | null
  receivedAt: number | null
  now: number
  notice: string
}

declare module 'claude-code' {
  interface PluginState {
    'multi-link-pane': { pane: Shaped<PaneState> }
  }
}
