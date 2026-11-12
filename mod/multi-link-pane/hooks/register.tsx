import { atom, read, update } from 'claude-code'
import type { EngineInterface, Register, Timer } from 'claude-code'
import type { PaneChild, PaneDetails, PaneReport, PaneRow, PaneState } from '../types'

const PANE = 'multi-link'
const initial: PaneState = { openId: null, actionNotice: '', report: null, receivedAt: null, now: 0, notice: '' }
const pane = atom({ plugin: 'multi-link-pane', key: 'pane' } as const, initial, { shape: 'pane-2' })
const missing = "multi-link isn't on PATH. See the README."
const tooOld = 'This multi-link is too old for the pane. Update it.'
const retrying = 'watcher stopped, retrying'
const colors = { claude: '#df9b79', codex: '#8cb9f1' }

function object(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === 'object' && !Array.isArray(value)
}

function details(value: Record<string, unknown>): boolean {
  if (value.opens === undefined) value.opens = null
  return (value.asked === null || typeof value.asked === 'string')
    && (value.said === null || typeof value.said === 'string') && typeof value.openable === 'boolean'
    && (value.opens === null || value.opens === 'app' || value.opens === 'terminal')
}

function child(value: unknown): value is PaneChild {
  return object(value) && details(value) && typeof value.id === 'string'
    && (value.harness === 'claude' || value.harness === 'codex')
    && typeof value.state === 'string' && typeof value.phrase === 'string'
    && (value.match === 'exact' || value.match === 'likely')
}

function row(value: unknown): value is PaneRow {
  return object(value) && details(value) && typeof value.id === 'string' && typeof value.label === 'string'
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
    && typeof value.seen === 'boolean'
    && (value.strip === null || (object(value.strip) && typeof value.strip.step === 'string'
      && (value.strip.title === null || typeof value.strip.title === 'string') && Array.isArray(value.strip.parts)
      && value.strip.parts.every(p => object(p) && (p.harness === 'claude' || p.harness === 'codex')
        && typeof p.kind === 'string' && [null, 'done', 'running', 'waiting', 'stale'].includes(p.status as string)
        && typeof p.active_min === 'number' && Number.isFinite(p.active_min) && p.active_min >= 1
        && typeof p.is_this === 'boolean')))
    && Array.isArray(value.sections)
    && value.sections.length === 3 && value.sections.every((s, i) => object(s)
      && s.name === ['working', 'waiting', 'recent'][i]
      && Array.isArray(s.rows) && s.rows.every(row))
}

const notificationScript = 'on run argv\n  display notification (item 2 of argv) with title (item 1 of argv)\nend run'

function contains(data: PaneReport, id: string | null): boolean {
  return id !== null && data.sections.some(s => s.rows.some(r => r.id === id || r.children.some(c => c.id === id)))
}

async function line($: EngineInterface, text: string, notify: boolean, announced: Set<string>): Promise<boolean> {
  let value: unknown
  try { value = JSON.parse(text) } catch { return false }
  if (!object(value) || value.v !== 1) return false
  const now = await $.clock.now()
  if (typeof value.error === 'string') {
    await update($, pane, s => ({ ...s, receivedAt: now, now, notice: value.error as string }))
  } else if (typeof value.tick === 'string') {
    await update($, pane, s => ({ ...s, receivedAt: now, now }))
  } else if (object(value.event) && value.event.kind === 'codex_done'
    && typeof value.event.run === 'string' && typeof value.event.id === 'string'
    && (value.event.step === null || typeof value.event.step === 'string')
    && (value.event.said === null || typeof value.event.said === 'string')) {
    await update($, pane, s => ({ ...s, receivedAt: now, now }))
    const event = value.event
    if (notify && !announced.has(event.run as string)) {
      announced.add(event.run as string)
      try {
        const result = await $.process.run(['osascript', '-e', notificationScript,
          event.step ? 'codex finished (' + event.step + ')' : 'codex finished',
          (event.said as string | null) ?? 'no final message'])
        if (result.exitCode !== 0) throw new Error('notification failed')
      } catch {
        await update($, pane, s => ({ ...s, actionNotice: 'notification failed' }))
      }
    }
  } else if (report(value)) {
    await update($, pane, s => ({ ...s, report: value, receivedAt: now, now, notice: '',
      openId: contains(value, s.openId) ? s.openId : null }))
    return true
  }
  return false
}

// Each register invocation owns one lifecycle. The host cancels streams and
// clock timers on unload; session.end explicitly releases the same resources.
export const register: Register = (on, options) => {
  let stop: (() => Promise<void>) | undefined

  on('session.start', async ($, e, next) => {
    const started = await next(e)
    await stop?.()
    const id = await $.session.id()
    await update($, pane, () => ({ ...initial })) // Atoms survive hot reloads.
    await $.command.register({ name: PANE, description: 'Show multi-link sessions in this repo' })
    await $.ui.open({ id: PANE, title: 'multi-link' })

    let notify = false
    if (options.notify !== false) {
      try { notify = (await $.process.run(['uname', '-s'])).stdout.trim() === 'Darwin' } catch { /* Unsupported host. */ }
    }
    const announced = new Set<string>()
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
        stream = $.process.spawn({ argv: ['multi-link', 'pane', '--json', '--watch', '--cwd', e.cwd, '--session', id, ...(notify ? ['--notify'] : [])] })
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
            if (await line($, text, notify, announced)) { gotData = true; delay = 2000 }
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
      if (!gotData && elapsed <= 5000 && /unrecognized subcommand|unexpected argument/i.test(stderr)) {
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
    const { Box, Text, Button } = $.ui.resolve(e)
    const state = await read($, pane)
    // The pane's own body, not the surface: viewport is the whole screen.
    const columns = Math.max(0, e.props.bodyColumns)
    const rows = Math.max(0, e.props.scroll.bodyRows)
    const lines = layout(state, rows)
    const notice = async (text: string) => { await update($, pane, s => ({ ...s, actionNotice: text })) }
    const toggle = async (id: string) => { await update($, pane, s => ({ ...s, openId: s.openId === id ? null : id })) }
    return <Box flexDirection="column">
      {lines.map((l, i) => l.actions ? <Box key={'actions:' + l.actions.id} flexDirection="row">
        {l.actions.openable && <Button key={'open:' + l.actions.id} onPress={async () => {
          try {
            const result = await $.process.run(['multi-link', 'open', '--no-resume', l.actions!.id])
            await notice(result.exitCode === 0
              ? result.stdout.trim() || (l.actions!.opens === 'terminal' ? 'jumped to terminal' : 'opened in app')
              : result.stderr.trim() || 'could not open session')
          } catch { await notice('could not open session') }
        }}><Text>{cut(l.actions.opens === 'terminal' ? 'Jump to terminal' : 'Open in app', columns)}</Text></Button>}
        <Button key={'copy:' + l.actions.id} onPress={async press => {
          try {
            const result = await $.ui.copy({ text: l.actions!.id, surface: press.surface })
            await notice(result.isCopied ? 'copied id' : 'could not copy id')
          } catch { await notice('could not copy id') }
        }}><Text>{cut('Copy id', columns)}</Text></Button>
      </Box> : l.segments ? <Box key={l.key} flexDirection="row">
        {cutSegments(l.segments, columns).map((segment, j) => <Text key={String(j)}
          color={segment.color} bold={segment.bold ?? l.bold} dimColor={segment.dim} wrap="truncate-end">
          {segment.text}
        </Text>)}
      </Box> : l.id ? <Button key={l.id} plain onPress={() => toggle(l.id!)}>
        <Text color={l.color} bold={l.bold} dimColor={l.dim} wrap="truncate-end">{cut(l.text, columns)}</Text>
      </Button> : <Text key={'line:' + i} color={l.color} bold={l.bold} dimColor={l.dim} wrap="truncate-end">
        {cut(l.text, columns)}
      </Text>)}
    </Box>
  })
}

type Segment = { text: string; color?: string; bold?: boolean; dim?: boolean }
type Line = { text: string; key?: string; segments?: Segment[]; id?: string; detail?: 'asked' | 'said'; actions?: PaneDetails & { id: string }; color?: string; bold?: boolean; dim?: boolean }
type Group = { name: string; entries: { own: boolean; open: boolean; lines: Line[] }[]; heading: boolean }

function styledLine(key: string, segments: Segment[], bold = false): Line {
  return { key, text: segments.map(s => s.text).join(''), segments, bold }
}

// Clip the complete line once, then give each surviving piece its original style.
function cutSegments(segments: Segment[], columns: number): Segment[] {
  const clean = segments.map(s => ({ ...s, text: cut(s.text, Infinity) }))
  const clipped = cut(clean.map(s => s.text).join(''), columns)
  let offset = 0
  return clean.map(s => {
    const text = clipped.slice(offset, offset + s.text.length)
    offset += s.text.length
    return { ...s, text }
  }).filter(s => s.text.length > 0)
}

function mark(state: string, phrase: string): string {
  return state === 'finished' || phrase.startsWith('done') ? '✓' : '●'
}

function rowLines(r: PaneRow, ownId: string, openId: string | null): Line[] {
  const own = r.is_this
  const color = colors[r.harness]
  const label = own ? 'this session' : r.label
  const step = r.step && !label.includes(r.step) ? '  ' + r.step : ''
  const lines: Line[] = [{ id: r.id, text: `${own ? '▌' : ' '}${mark(r.state, r.phrase)} ${label}${step}`, color, bold: own }]
  const expand = (r: PaneDetails & { id: string }) => {
    if (r.id === openId) lines.push(
      { text: '   asked ' + (r.asked ?? '—'), dim: true, detail: 'asked' },
      { text: '   said  ' + (r.said ?? '—'), dim: true, detail: 'said' },
      { text: '', actions: r })
  }
  expand(r)
  for (const c of r.children.slice(0, own ? 3 : undefined)) {
    lines.push({ id: c.id, text: `${own ? '▌' : ' '} └ ${c.harness} ${c.id.slice(0, 8)} ${mark(c.state, c.phrase)}${c.match === 'likely' ? ' likely' : ''}${c.id === ownId ? ' · this session' : ''}`, color: colors[c.harness] })
    expand(c)
  }
  if (own && r.children.length > 3) lines.push({ text: `▌  +${r.children.length - 3} runs`, dim: true })
  for (const h of r.handoffs) lines.push({ text: `${own ? '▌' : ' '} └ ${h.outcome}`, dim: true })
  return lines
}

function layout(s: PaneState, height: number): Line[] {
  if (!height) return []
  const data = s.report
  const header: Line = { text: 'multi-link' + (data ? ' · ' + (data.repo.split('/').filter(Boolean).pop() ?? '/') : ''), bold: true }
  if (!data) return [header, { text: s.notice || 'reading sessions…', dim: true }].slice(0, height)
  const groups: Group[] = data.sections.map(section => ({ name: section.name === 'recent' ? 'recent (2h)' : section.name,
    heading: true, entries: section.rows.map(r => ({
      own: r.is_this || r.children.some(c => c.id === data.this),
      open: r.id === s.openId || r.children.some(c => c.id === s.openId),
      lines: rowLines(r, data.this, s.openId),
    })) }))
  if (!data.seen) groups[0].entries.unshift({ own: true, open: false, lines: [{ text: '▌ this session · not seen yet', color: colors.claude, bold: true }] })
  const allParts = data.strip?.parts ?? []
  let parts = allParts.slice(-4)
  const ownPart = allParts.filter(p => p.is_this).at(-1)
  if (ownPart && !parts.some(p => p.is_this)) parts = [ownPart, ...parts.slice(1)]
  const stripHeading: Line[] = data.strip ? [styledLine('strip:heading', [
    { text: 'step ', dim: true },
    { text: data.strip.step, bold: true },
    ...(data.strip.title ? [{ text: ' ' + data.strip.title, dim: true }] : []),
  ])] : []
  const empty = data.sections.every(section => !section.rows.length)
  const notice = [s.notice, s.actionNotice].filter(Boolean).join(' · ')
  const footer: Line = { text: `checked ${Math.max(0, Math.floor((s.now - (s.receivedAt ?? s.now)) / 1000))}s ago${notice ? ' · ' + notice : ''}`, dim: true }
  let dropped = 0
  const draw = (): Line[] => [header, ...stripHeading,
    ...(allParts.length > parts.length ? [{ text: '+' + (allParts.length - parts.length) + ' earlier', dim: true }] : []),
    ...parts.map((p, i) => styledLine('strip:part:' + i, [
      { text: p.is_this ? '▌' : ' ', color: p.is_this ? colors.claude : undefined },
      { text: p.harness.padEnd(6) + ' ', color: colors[p.harness] },
      { text: p.kind.padEnd(10) + ' ' },
      { text: p.status === 'done' ? '✓' : p.status === 'running' ? '●' : p.status === 'waiting' ? '◆' : '·',
        color: p.status === 'done' ? 'success' : p.status === 'running' || p.status === 'waiting' ? 'warning' : undefined,
        dim: p.status !== 'done' && p.status !== 'running' && p.status !== 'waiting' },
      { text: ' ' + p.active_min + 'm', dim: true },
    ], p.is_this)),
    ...groups.flatMap(g => [
      ...(g.heading && g.entries.length ? [{ text: g.name + ' ─', dim: true }] : []),
      ...g.entries.flatMap(entry => entry.lines),
    ]), ...(empty ? [{ text: 'No sessions in this repo in the last 2 h.', dim: true }] : []),
    ...(dropped ? [{ text: `+${dropped} more`, dim: true }] : []), footer]
  const dropRows = (g: Group, keepOpen: boolean) => {
    for (let i = g.entries.length - 1; i >= 0 && draw().length > height; i--) {
      if (g.entries[i].own || (keepOpen && g.entries[i].open)) continue
      g.entries.splice(i, 1)
      dropped++
    }
  }
  dropRows(groups[2], true)
  dropRows(groups[1], true)
  while (draw().length > height && parts.length) parts.shift()
  for (const detail of ['said', 'asked']) {
    for (const g of groups) for (const entry of g.entries) {
      if (draw().length > height) entry.lines = entry.lines.filter(l => l.detail !== detail)
    }
  }
  dropRows(groups[0], true)
  // An open row and its actions yield only after the working rows.
  for (const g of [...groups].reverse()) dropRows(g, false)
  // Section headings yield before the pinned row's children.
  for (const g of [...groups].reverse()) if (draw().length > height) g.heading = false
  for (const g of groups) for (const entry of g.entries.filter(e => e.own)) {
    let extra = 0
    const summary = entry.lines.find(l => /\+\d+ runs$/.test(l.text))
    if (summary) { extra = Number(summary.text.match(/\+(\d+)/)![1]); entry.lines.splice(entry.lines.indexOf(summary), 1) }
    while (draw().length + (extra ? 1 : 0) > height && entry.lines.length > 1) {
      const removable = entry.lines.findLastIndex((l, i) => i > 0 && !l.actions
        && !l.detail && l.id !== s.openId && l.id !== data.this)
      if (removable < 0) break
      const [removed] = entry.lines.splice(removable, 1)
      if (removed.id) extra++
    }
    if (extra) entry.lines.push({ text: `▌  +${extra} runs`, dim: true })
  }
  const lines = draw()
  if (lines.length <= height) return lines
  const own = groups.flatMap(g => g.entries).find(e => e.own)?.lines[0]
  const pins = [header, ...stripHeading, ...(own ? [own] : []), footer]
  if (pins.length >= height) return height === 1 ? [footer] : [...pins.slice(0, height - 1), footer]
  return [...pins.slice(0, -1), ...lines.filter(l => !pins.includes(l)).slice(0, height - pins.length), footer]
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
