import AppKit
import MenubarCore
import SwiftUI

struct SetupView: View {
    let model: SetupModel
    var close: () -> Void = {}
    var poll = true
    @Environment(\.colorScheme) private var scheme
    private var palette: BoardPalette { BoardPalette(dark: scheme == .dark) }
    private var dim: Color { boardColor(palette.dim) }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack(spacing: 12) {
                SetupIcon().frame(width: 44, height: 44)
                VStack(alignment: .leading, spacing: 3) {
                    Text(model.isTemporary ? "Move multi-link to Applications first" :
                            model.remaining == 0 ? "Everything is on" : model.remaining == 1 ? "1 thing needs you" : "\(model.remaining) things need you")
                        .font(.system(size: 15, weight: .semibold))
                    Text(model.isTemporary ? "It's running from the download. Drag it into Applications, then open it from there. Setup waits until then, so nothing points at a copy that goes away." : "Everything stays on this Mac.")
                        .font(.system(size: 12.5)).foregroundStyle(dim).fixedSize(horizontal: false, vertical: true)
                }
            }.padding(18)
            if model.isTemporary {
                Divider()
                HStack {
                    Text("Applications or ~/Applications both work.").foregroundStyle(dim)
                    Spacer()
                    Button("Show in Finder") { reveal(model.bundlePath) }.buttonStyle(SetupButtonStyle(primary: true))
                }.padding(18)
            } else {
                if model.remaining > 0 {
                    VStack(alignment: .leading, spacing: 4) {
                        Button { Task { await model.turnOnEverything() } } label: {
                            Text("Turn on everything").font(.system(size: 14, weight: .semibold))
                                .frame(maxWidth: .infinity).padding(.vertical, 5)
                        }.buttonStyle(SetupButtonStyle(primary: true))
                        Text("or turn each on below").font(.system(size: 12)).foregroundStyle(dim)
                    }.padding(.horizontal, 18).padding(.bottom, 8)
                } else if !model.loginEnabled {
                    Button("Turn on everything") { Task { await model.turnOnEverything() } }
                        .buttonStyle(SetupButtonStyle()).padding(.horizontal, 18).padding(.bottom, 8)
                }
                VStack(spacing: 0) {
                    setupRow(green: model.hooksGreen) {
                        VStack(alignment: .leading, spacing: 5) {
                            HStack(spacing: 6) {
                                Text("Claude hooks").fontWeight(.semibold)
                                Text("Recommended").font(.system(size: 10.5, weight: .semibold))
                                    .padding(.horizontal, 6).padding(.vertical, 1)
                                    .foregroundStyle(boardColor(palette.ok))
                                    .overlay(Capsule().stroke(boardColor(palette.ok)))
                            }
                            Text(model.hooksMessage).font(.system(size: 12)).foregroundStyle(dim)
                            DisclosureGroup("What this does") {
                                Text("Adds multi-link's entries to \(model.hooks?.file ?? "~/.claude/settings.json"). A copy of the file is saved beside it first (\(model.hooks?.file ?? "~/.claude/settings.json").multi-link-backup-…). Your other hooks stay as they are. multi-link works without them; links are then matched by time more often. Remove them any time from this window.")
                                    .font(.system(size: 12)).foregroundStyle(dim).fixedSize(horizontal: false, vertical: true)
                            }.font(.system(size: 12)).tint(boardColor(palette.codex))
                            if model.hooksChanged {
                                Text("Start a new Claude session to load them.").font(.system(size: 12)).foregroundStyle(dim)
                            }
                        }
                    } action: {
                        if model.hooks?.state == .fixable {
                            Button("Turn on") { Task { await model.turnOnHooks() } }
                        } else if let file = model.hooks?.file {
                            Button("Show file") { reveal(file) }
                        } else {
                            Button("Try again") { Task { await model.refresh() } }
                        }
                    }
                    Divider()
                    setupRow(green: model.notificationsGreen) {
                        VStack(alignment: .leading, spacing: 4) {
                            Text("Notifications").fontWeight(.semibold)
                            Text(model.notificationsGreen ? "On. A banner when a Codex hand-off finishes." :
                                    model.notification.authorization == .notDetermined ? "A banner when a Codex hand-off finishes." :
                                    "Turned off in System Settings. Turn on Allow Notifications for multi-link.")
                                .font(.system(size: 12)).foregroundStyle(dim)
                            if let error = model.notificationError {
                                Text(error).font(.system(size: 12)).foregroundStyle(boardColor(palette.warn))
                                    .fixedSize(horizontal: false, vertical: true)
                            }
                        }
                    } action: {
                        if model.notificationsGreen {
                            Text("On").foregroundStyle(boardColor(palette.ok))
                        } else if model.notification.authorization == .notDetermined {
                            Button("Turn on") { Task { await model.turnOnNotifications() } }
                        } else {
                            Button("Open Settings") { NSWorkspace.shared.open(URL(string: MenubarActions.notificationSettings)!) }
                                .buttonStyle(SetupButtonStyle(primary: true))
                        }
                    }
                }.padding(.horizontal, 18)
                terminalLine
                if let error = model.actionError { errorText(error) }
                if let error = model.loginError { errorText(error) }
                if model.loginStatus == .requiresApproval {
                    Button("Open Login Items Settings") { model.openLoginSettings() }.padding(.horizontal, 18).padding(.bottom, 10)
                }
                Divider()
                VStack(alignment: .leading, spacing: 10) {
                    HStack {
                        Button("Remove hooks") { Task { await model.removeHooks() } }
                            .buttonStyle(.plain).foregroundStyle(dim).font(.system(size: 12))
                        Spacer()
                        Button("Later", action: close)
                    }
                    Text("Rows turn green on their own once you've done it.").foregroundStyle(dim).font(.system(size: 12))
                }.padding(18)
            }
        }
        .font(.system(size: 13)).foregroundStyle(boardColor(palette.foreground))
        .background(boardColor(palette.background)).tint(boardColor(palette.codex))
        .frame(width: 520).fixedSize(horizontal: false, vertical: true)
        .buttonStyle(SetupButtonStyle())
        .disabled(model.busy)
        .task {
            guard poll else { return }
            await model.refresh()
            while !Task.isCancelled {
                do { try await Task.sleep(for: .seconds(2)) } catch { return }
                await model.pollTick()
            }
        }
    }
    @ViewBuilder private var terminalLine: some View {
        if !model.terminalLinks.isEmpty {
            VStack(alignment: .leading, spacing: 6) {
                Text("Use in Terminal").fontWeight(.semibold)
                    .foregroundStyle(boardColor(palette.foreground))
                ForEach(model.terminalLinks.filter { $0.state != .absent }, id: \.path) { link in
                    Text(link.message).textSelection(.enabled)
                }
                if let command = model.terminalCommand {
                    Text("Not on your PATH yet. To use multi-link in Terminal, paste this there (it asks for your password):")
                    Text(command).font(.system(size: 12, design: .monospaced))
                        .foregroundStyle(boardColor(palette.foreground))
                        .textSelection(.enabled)
                        .padding(8)
                        .background(boardColor(palette.selection), in: RoundedRectangle(cornerRadius: 6))
                    Button("Copy command") {
                        NSPasteboard.general.clearContents()
                        NSPasteboard.general.setString(command, forType: .string)
                    }.buttonStyle(SetupButtonStyle())
                }
                (Text("Check with ") + Text("command -v multi-link").monospaced()
                    + Text(" in Terminal: an earlier PATH entry wins."))
            }
            .font(.system(size: 12)).foregroundStyle(dim)
            .fixedSize(horizontal: false, vertical: true)
            .padding(.horizontal, 18).padding(.vertical, 12)
        }
    }
    private func errorText(_ text: String) -> some View {
        Text(text).font(.system(size: 12)).foregroundStyle(boardColor(palette.warn))
            .textSelection(.enabled).padding(.horizontal, 18).padding(.bottom, 10)
    }
    private func setupRow<Content: View, Action: View>(green: Bool, @ViewBuilder content: () -> Content,
                                                     @ViewBuilder action: () -> Action) -> some View {
        HStack(alignment: .top, spacing: 10) {
            SetupStatusCircle(green: green, palette: palette)
            content().frame(maxWidth: .infinity, alignment: .leading)
            action().buttonStyle(SetupButtonStyle()).controlSize(.small)
        }.padding(.vertical, 12)
    }
}

struct SetupStatusCircle: View {
    let green: Bool
    let palette: BoardPalette
    var body: some View {
        Text(green ? "✓" : "!").font(.system(size: 12, weight: .bold)).frame(width: 20, height: 20)
            .foregroundStyle(boardColor(green ? palette.background : palette.warn))
            .background(Circle().fill(green ? boardColor(palette.ok) : .clear))
            .overlay(Circle().stroke(green ? .clear : boardColor(palette.warn), lineWidth: 1.5))
    }
}

struct SetupFinishLine: View {
    let model: SetupModel
    var open: () -> Void = {}
    @Environment(\.colorScheme) private var scheme
    var body: some View {
        let palette = BoardPalette(dark: scheme == .dark)
        if model.remaining > 0 {
            Button(action: open) {
                HStack(spacing: 8) {
                    SetupStatusCircle(green: false, palette: palette)
                    Text("Finish setup · \(model.remaining) left").fontWeight(.semibold)
                    Spacer(); Image(systemName: "chevron.right")
                }.padding(8).background(boardColor(palette.selection), in: RoundedRectangle(cornerRadius: 7))
            }.buttonStyle(.plain).font(.system(size: 12.5)).padding(8)
        }
    }
}

struct SetupFooterButton: View {
    let model: SetupModel
    var open: () -> Void = {}
    @Environment(\.colorScheme) private var scheme
    var body: some View {
        Button(action: open) {
            HStack(spacing: 4) {
                Text("Setup…")
                if model.hooksFixableAtLaunch {
                    Circle().fill(boardColor(BoardPalette(dark: scheme == .dark).warn))
                        .frame(width: 6, height: 6)
                }
            }
        }
        .buttonStyle(.plain)
        .accessibilityLabel(model.hooksFixableAtLaunch ? "Setup, needs attention" : "Setup")
    }
}

struct SetupLoginControl: View {
    let model: SetupModel
    @Environment(\.colorScheme) private var scheme
    var body: some View {
        let palette = BoardPalette(dark: scheme == .dark)
        VStack(alignment: .leading, spacing: 8) {
            Toggle("Open at login", isOn: Binding(get: { model.loginEnabled }, set: { value in
                Task { await model.setLoginEnabled(value) }
            })).toggleStyle(SetupCheckboxStyle()).disabled(model.isTemporary || model.busy)
            if model.loginStatus == .requiresApproval {
                Text("macOS wants you to approve it in Login Items.").foregroundStyle(boardColor(palette.dim))
                Button("Open Login Items Settings") { model.openLoginSettings() }
            }
            if model.loginStatus == .unknown { Text("Login item status is unavailable.") }
            if let error = model.loginError { Text(error).foregroundStyle(boardColor(palette.warn)) }
        }.font(.system(size: 12.5)).buttonStyle(SetupButtonStyle()).padding(12)
    }
}

struct SetupPopoverControls: View {
    let model: SetupModel
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SetupFinishLine(model: model)
            SetupLoginControl(model: model)
        }
    }
}

/// Board 21's chosen C hand-off icon, drawn with the same geometry as the app asset.
private struct SetupIcon: View {
    var body: some View {
        Canvas { context, size in
            let scale = size.width / 824
            context.scaleBy(x: scale, y: scale)
            context.translateBy(x: -100, y: -100)
            context.fill(Path(roundedRect: CGRect(x: 100, y: 100, width: 824, height: 824), cornerRadius: 180),
                         with: .linearGradient(Gradient(colors: [boardColor(0x30343b), boardColor(0x17191e)]),
                                               startPoint: CGPoint(x: 100, y: 100), endPoint: CGPoint(x: 924, y: 924)))
            var line = Path()
            line.move(to: CGPoint(x: 372, y: 470)); line.addLine(to: CGPoint(x: 372, y: 600))
            line.addCurve(to: CGPoint(x: 450, y: 672), control1: CGPoint(x: 372, y: 650), control2: CGPoint(x: 400, y: 672))
            line.addLine(to: CGPoint(x: 560, y: 672))
            context.stroke(line, with: .color(boardColor(0xece8e1)), style: StrokeStyle(lineWidth: 46, lineCap: .round))
            context.fill(Path(ellipseIn: CGRect(x: 268, y: 268, width: 208, height: 208)), with: .color(boardColor(0xdf9b79)))
            context.fill(Path(ellipseIn: CGRect(x: 558, y: 568, width: 208, height: 208)), with: .color(boardColor(0x8cb9f1)))
        }.accessibilityLabel("multi-link")
    }
}

@MainActor final class SetupWindowController: NSObject, NSWindowDelegate {
    private var window: NSWindow?
    let model: SetupModel
    init(model: SetupModel) { self.model = model }
    func show() {
        if let window { window.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true); return }
        NSApp.setActivationPolicy(.regular)
        let window = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 520, height: 400),
                              styleMask: [.titled, .closable, .miniaturizable], backing: .buffered, defer: false)
        window.title = "Set up multi-link"
        window.isReleasedWhenClosed = false
        window.contentView = NSHostingView(rootView: SetupView(model: model, close: { [weak self] in self?.window?.close() }))
        window.delegate = self
        self.window = window
        window.center(); window.makeKeyAndOrderFront(nil); NSApp.activate(ignoringOtherApps: true)
    }
    func windowWillClose(_ notification: Notification) {
        window?.contentView = nil // releases the view and cancels its polling task
        window = nil
        NSApp.setActivationPolicy(.accessory)
    }
}

func boardColor(_ rgb: UInt32) -> Color {
    Color(red: Double((rgb >> 16) & 255) / 255, green: Double((rgb >> 8) & 255) / 255, blue: Double(rgb & 255) / 255)
}
private func reveal(_ path: String) { NSWorkspace.shared.activateFileViewerSelecting([URL(fileURLWithPath: path)]) }

// Native AppKit-backed controls render as placeholders in ImageRenderer. These SwiftUI
// styles keep the shipped controls and the review PNGs identical, including their labels.
struct SetupButtonStyle: ButtonStyle {
    var primary = false
    @Environment(\.colorScheme) private var scheme
    @Environment(\.isEnabled) private var enabled
    func makeBody(configuration: Configuration) -> some View {
        let palette = BoardPalette(dark: scheme == .dark)
        configuration.label
            .font(.system(size: 12.5))
            .padding(.horizontal, 11).padding(.vertical, 5)
            .foregroundStyle(primary ? (scheme == .dark ? boardColor(0x1d1c1a) : Color.white) : boardColor(palette.foreground))
            .background(primary ? boardColor(palette.codex) : boardColor(palette.background),
                        in: RoundedRectangle(cornerRadius: 6))
            .overlay(RoundedRectangle(cornerRadius: 6).stroke(boardColor(primary ? palette.codex : palette.line)))
            .opacity(!enabled ? 0.5 : configuration.isPressed ? 0.75 : 1)
    }
}
struct SetupCheckboxStyle: ToggleStyle {
    func makeBody(configuration: Configuration) -> some View {
        Button { configuration.isOn.toggle() } label: {
            HStack(spacing: 6) {
                Image(systemName: configuration.isOn ? "checkmark.square.fill" : "square")
                configuration.label
            }
        }.buttonStyle(.plain)
        .accessibilityValue(configuration.isOn ? "Checked" : "Unchecked")
    }
}
