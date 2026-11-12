export type PaneDetails = {
  asked: string | null
  said: string | null
  openable: boolean
}
export type PaneChild = PaneDetails & {
  id: string
  harness: 'claude' | 'codex'
  state: string
  phrase: string
  match: 'exact' | 'likely'
}
export type PaneRow = PaneDetails & {
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
  strip: { step: string; title: string | null; parts: PanePart[] } | null
  sections: { name: 'working' | 'waiting' | 'recent'; rows: PaneRow[] }[]
}
export type PanePart = {
  harness: 'claude' | 'codex'
  kind: string
  status: 'done' | 'running' | 'waiting' | 'stale' | null
  active_min: number
  is_this: boolean
}
export type PaneState = {
  openId: string | null
  actionNotice: string
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
