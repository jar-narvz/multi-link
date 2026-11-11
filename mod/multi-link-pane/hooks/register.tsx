import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register, Timer } from 'claude-code'
import type { PaneChild, PaneReport, PaneRow, PaneState } from '../types'

const PANE = 'multi-link'
const initial: PaneState = { report: null, receivedAt: null, now: 0, notice: '' }
const pane = atom({ plugin: 'multi-link-pane', key: 'pane' } as const, initial, { shape: 'pane-1' })
const missing = "multi-link isn't on PATH. See the README."
const tooOld = 'This multi-link is too old for the pane. Update it.'
const retrying = 'watcher stopped, retrying'
const colors = { claude: '#df9b79', codex: '#8cb9f1' }

function object(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function child(value: unknown): value is PaneChild {
  return object(value) && typeof value.id === 'string'
    && (value.harness === 'claude' || value.harness === 'codex')
    && typeof value.state === 'string' && typeof value.phrase === 'string'
    && (value.match === 'exact' || value.match === 'likely')
}

function row(value: unknown): value is PaneRow {
  return object(value) && typeof value.id === 'string' && typeof value.label === 'string'
    && (value.harness === 'claude' || value.harness === 'codex')
    && (value.step === null || typeof value.step === 'string')
    && typeof value.state === 'string' && typeof value.phrase === 'string'
    && typeof value.is_this === 'boolean' && Array.isArray(value.children)
    && value.children.every(child) && Array.isArray(value.handoffs)
    && value.handoffs.every(h => object(h) && typeof h.outcome === 'string'
      && (h.step === null || typeof h.step === 'string')
      && Array.isArray(h.candidates) && h.candidates.every(id => typeof id === 'string'))
}

function report(value: unknown): value is PaneReport {
  return object(value) && value.v === 1 && typeof value.repo === 'string'
    && typeof value.now === 'string' && typeof value.this === 'string'
    && typeof value.seen === 'boolean' && Array.isArray(value.sections)
    && value.sections.length === 3 && value.sections.every((s, i) => object(s)
      && s.name === ['working', 'waiting', 'recent'][i]
      && Array.isArray(s.rows) && s.rows.every(row))
}

async function line($: EngineInterface, text: string): Promise<boolean> {
  let value: unknown
  try { value = JSON.parse(text) } catch { return false }
  if (object(value) && value.v === 1 && typeof value.error === 'string') {
    await update($, pane, s => ({ ...s, notice: value.error as string }))
  } else if (report(value)) {
    const now = await $.clock.now()
    await update($, pane, () => ({ report: value, receivedAt: now, now, notice: '' }))
    return true
  }
  return false
}

// Each register invocation owns one lifecycle. The host cancels streams and
// clock timers on unload; session.end explicitly releases the same resources.
export const register: Register = on => {
  let stop: (() => Promise<void>) | undefined

  on('session.start', async ($, e, next) => {
    const started = await next(e)
    await stop?.()
    const id = await $.session.id()
    await update($, pane, () => ({ ...initial })) // Atoms survive hot reloads.
    await $.command.register({ name: PANE, description: 'Show multi-link sessions in this repo' })
    await $.ui.open({ id: PANE, title: 'multi-link' })

    let active = true
    let retry: Timer | undefined
    let stream: ReturnType<EngineInterface['process']['spawn']> | undefined
    let delay = 2000
    const tick = $.clock.every(1000, async () => {
      if (!active) return
      const now = await $.clock.now()
      if (active) await update($, pane, s => ({ ...s, now }))
    })
    stop = async () => {
      active = false
      retry?.cancel()
      tick.cancel()
      // return() kills even a silent child with a pending read (host contract).
      await stream?.return({ code: null, signal: 'SIGTERM' }).catch(() => undefined)
    }

    async function watch(): Promise<void> {
      if (!active) return
      let stdout = ''
      let stderr = ''
      let pulled = false
      let gotData = false
      const began = await $.clock.now()
      if (!active) return
      try {
        stream = $.process.spawn({ argv: ['multi-link', 'pane', '--json', '--watch', '--cwd', e.cwd, '--session', id] })
        for await (const chunk of stream) {
          if (!active) break
          pulled = true
          if (chunk.stream === 'stderr') {
            stderr = (stderr + chunk.text).slice(-8192)
            continue
          }
          stdout += chunk.text
          let newline: number
          while ((newline = stdout.indexOf('\n')) >= 0) {
            const text = stdout.slice(0, newline)
            stdout = stdout.slice(newline + 1)
            if (!active) break
            if (await line($, text)) { gotData = true; delay = 2000 }
          }
        }
      } catch {
        if (!active) return
        if (!pulled) {
          await update($, pane, s => ({ ...s, notice: missing }))
          return
        }
      }
      if (!active) return
      stream = undefined
      const elapsed = (await $.clock.now()) - began
      if (!gotData && elapsed <= 5000 && /unrecognized subcommand/i.test(stderr)) {
        await update($, pane, s => ({ ...s, notice: tooOld }))
        return
      }
      await update($, pane, s => ({ ...s, notice: retrying }))
      if (!active) return
      retry = $.clock.after(delay, () => { void watch().catch(() => undefined) })
      delay = Math.min(60000, delay * 2)
    }
    void watch().catch(() => undefined)
    return started
  })

  on('session.end', async (_, e, next) => {
    await stop?.()
    stop = undefined
    return next(e)
  })

  on('command.run', { command: PANE }, async $ => {
    await $.ui.open({ id: PANE, title: 'multi-link' })
    return { text: 'multi-link pane opened.' }
  })

  on('ui.render', { component: 'Pane', requestId: PANE }, async ($, e) => {
    const { Box, Text } = $.ui.resolve(e)
    const state = await read($, pane)
    const columns = Math.max(0, e.viewport?.columns ?? e.props.bodyColumns)
    const rows = Math.max(0, e.viewport?.rows ?? e.props.scroll.bodyRows)
    const lines = layout(state, rows)
    return <Box flexDirection="column">
      {lines.map(l => <Text color={l.color} bold={l.bold} dimColor={l.dim} wrap="truncate-end">
        {cut(l.text, columns)}
      </Text>)}
    </Box>
  })
}

type Line = { text: string; color?: string; bold?: boolean; dim?: boolean }
type Group = { name: string; entries: { own: boolean; lines: Line[] }[]; heading: boolean }

function mark(state: string, phrase: string): string {
  return state === 'finished' || phrase.startsWith('done') ? '✓' : '●'
}

function rowLines(r: PaneRow, ownId: string): Line[] {
  const own = r.is_this
  const color = colors[r.harness]
  const label = own ? 'this session' : r.label
  const step = r.step && !label.includes(r.step) ? '  ' + r.step : ''
  const lines: Line[] = [{ text: `${own ? '▌' : ' '}${mark(r.state, r.phrase)} ${label}${step}`, color, bold: own }]
  for (const c of r.children.slice(0, own ? 3 : undefined)) {
    lines.push({ text: `${own ? '▌' : ' '} └ ${c.harness} ${c.id.slice(0, 8)} ${mark(c.state, c.phrase)}${c.match === 'likely' ? ' likely' : ''}${c.id === ownId ? ' · this session' : ''}`, color: colors[c.harness] })
  }
  if (own && r.children.length > 3) lines.push({ text: `▌  +${r.children.length - 3} runs`, dim: true })
  for (const h of r.handoffs) lines.push({ text: `${own ? '▌' : ' '} └ ${h.outcome}`, dim: true })
  return lines
}

function layout(s: PaneState, height: number): Line[] {
  if (!height) return []
  const data = s.report
  const header: Line = { text: 'multi-link' + (data ? ' · ' + (data.repo.split('/').filter(Boolean).pop() ?? '/') : ''), bold: true }
  if (!data) {
    return [header, { text: s.notice || 'reading sessions…', dim: true }].slice(0, height)
  }
  const groups: Group[] = data.sections.map(section => ({ name: section.name === 'recent' ? 'recent (2h)' : section.name,
    heading: true, entries: section.rows.map(r => ({ own: r.is_this || r.children.some(c => c.id === data.this), lines: rowLines(r, data.this) })) }))
  if (!data.seen) groups[0].entries.unshift({ own: true, lines: [{ text: '▌ this session · not seen yet', color: colors.claude, bold: true }] })
  const empty = data.sections.every(section => !section.rows.length)
  const footer: Line = { text: `updated ${Math.max(0, Math.floor((s.now - (s.receivedAt ?? s.now)) / 1000))}s ago${s.notice ? ' · ' + s.notice : ''}`, dim: true }
  let dropped = 0
  const draw = (): Line[] => [header, ...groups.flatMap(g => [
    ...(g.heading ? [{ text: g.name + ' ─', dim: true }] : []),
    ...g.entries.flatMap(entry => entry.lines),
  ]), ...(empty ? [{ text: 'No sessions in this repo in the last 2 h.', dim: true }] : []),
  ...(dropped ? [{ text: `+${dropped} more`, dim: true }] : []), footer]

  // Empty section labels yield before sessions. Remove whole sessions, never
  // orphan a child, and count sessions (rows), rather than their drawn lines.
  for (const g of [...groups].reverse()) {
    if (draw().length > height && !g.entries.length) g.heading = false
  }
  for (const g of [...groups].reverse()) {
    for (let i = g.entries.length - 1; i >= 0 && draw().length > height; i--) {
      if (g.entries[i].own) continue
      g.entries.splice(i, 1)
      dropped++
    }
    if (!g.entries.length && draw().length > height) g.heading = false
  }
  for (const g of [...groups].reverse()) {
    if (draw().length > height) g.heading = false
  }
  // At very short sizes the pinned session's children also need a cut. Replace
  // every omitted child with +N runs; hand-off annotations yield first.
  for (const g of groups) {
    for (const entry of g.entries.filter(e => e.own)) {
      let extra = 0
      const summary = entry.lines.find(l => /\+\d+ runs$/.test(l.text))
      if (summary) { extra = Number(summary.text.match(/\+(\d+)/)![1]); entry.lines.splice(entry.lines.indexOf(summary), 1) }
      while (draw().length + (extra ? 1 : 0) > height && entry.lines.length > 1) {
        const removed = entry.lines.pop()!
        if (/ └ (codex|claude) /.test(removed.text)) extra++
      }
      if (extra) entry.lines.push({ text: `▌  +${extra} runs`, dim: true })
    }
  }
  for (const g of [...groups].reverse()) {
    if (draw().length > height) g.heading = false
  }
  const lines = draw()
  if (lines.length <= height) return lines
  // Degenerate viewports cannot hold all pins. Keep the footer and, if room,
  // the header and this session before any overflow annotation.
  const own = groups.flatMap(g => g.entries).find(e => e.own)?.lines[0]
  return height === 1 ? [footer] : [header, ...(own && height > 2 ? [own] : []), ...lines.slice(1).filter(l => l !== own && l !== footer).slice(0, Math.max(0, height - (own && height > 2 ? 3 : 2))), footer]
}

// Cut in terminal cells, keeping wide characters and combining marks together.
function cut(text: string, columns: number): string {
  const clean = text.replace(/[\x00-\x1f\x7f-\x9f\u2028\u2029]/g, ' ')
  const width = (c: string): number => {
    if (/\p{Mark}|\u200d|\ufe0f/u.test(c)) return 0
    const n = c.codePointAt(0)!
    return n >= 0x1100 && (n <= 0x115f || n === 0x2329 || n === 0x232a
      || (n >= 0x2e80 && n <= 0xa4cf) || (n >= 0xac00 && n <= 0xd7a3)
      || (n >= 0xf900 && n <= 0xfaff) || (n >= 0xfe10 && n <= 0xfe6f)
      || (n >= 0xff01 && n <= 0xff60) || (n >= 0xffe0 && n <= 0xffe6)
      || n >= 0x1f000) ? 2 : 1
  }
  const chars = Array.from(clean)
  if (chars.reduce((n, c) => n + width(c), 0) <= columns) return clean
  let used = 0
  let out = ''
  for (const c of chars) {
    const cells = width(c)
    if (used + cells > columns - 1) break
    out += c; used += cells
  }
  return columns > 0 ? out + '…' : ''
}
