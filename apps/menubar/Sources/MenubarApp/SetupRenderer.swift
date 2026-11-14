import AppKit
import MenubarCore
import SwiftUI
import UserNotifications

/// Entirely fake dependencies: render mode never constructs the app delegate or controller.
@MainActor private final class RenderSetupSources: SetupRunning, SetupNotificationSource, LoginItemService, SetupDefaults {
    var hooks = "ok"
    var authorization: UNAuthorizationStatus = .authorized
    var status: LoginItemStatus = .enabled
    var fail = false
    func run(_ arguments: [String]) async throws -> CommandOutput {
        if arguments == ["setup", "--json"] {
            return CommandOutput(status: 0, stdout: """
            {"v":1,"items":[{"id":"hooks","state":"\(hooks)","file":"~/.claude/settings.json","events":[{"event":"PreToolUse","status":"customized"}]}]}
            """, stderr: "")
        }
        return CommandOutput(status: fail ? 1 : 0, stdout: "", stderr: "Could not update Claude settings: permission denied.")
    }
    func settings() async -> SetupNotificationStatus { .init(authorization: authorization, alerts: .enabled) }
    func request() async throws {}
    func register() throws { if fail { throw RenderFailure.denied } }
    func unregister() throws { if fail { throw RenderFailure.denied } }
    func openSettings() {}
    func hasShownSetup() -> Bool { true }
    func markSetupShown() {}
}
private enum RenderFailure: LocalizedError {
    case denied
    var errorDescription: String? { "Operation not permitted" }
}

@MainActor func renderSetup(to directory: String) async throws {
    let dir = URL(fileURLWithPath: directory, isDirectory: true)
    try FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)
    let states = ["all-green", "all-green-login-off", "first-launch", "hooks-fixable", "hooks-attention", "hooks-unreadable",
                  "notifications-denied", "apply-error", "translocated-gate", "popover-requiresApproval", "popover-error"]
    for state in states {
        let sources = RenderSetupSources()
        switch state {
        case "all-green-login-off": sources.status = .notRegistered
        case "first-launch": sources.hooks = "fixable"; sources.authorization = .notDetermined; sources.status = .notRegistered
        case "hooks-fixable", "apply-error": sources.hooks = "fixable"
        case "hooks-attention": sources.hooks = "attention"
        case "hooks-unreadable": sources.hooks = "unreadable"
        case "notifications-denied": sources.authorization = .denied
        case "popover-requiresApproval": sources.status = .requiresApproval; sources.hooks = "fixable"
        case "popover-error": sources.status = .notFound; sources.hooks = "fixable"
        default: break
        }
        let model = SetupModel(runner: sources, notifications: sources, login: sources, defaults: sources,
                               bundlePath: state == "translocated-gate" ? "/Volumes/multi-link/multi-link.app" : "/Applications/multi-link.app")
        await model.refresh()
        if state == "apply-error" { sources.fail = true; await model.turnOnHooks() }
        if state == "popover-error" { sources.fail = true; await model.setLoginEnabled(false) }
        for dark in [false, true] {
            let palette = BoardPalette(dark: dark)
            let view: AnyView
            if state.hasPrefix("popover-") {
                view = AnyView(VStack(alignment: .leading, spacing: 0) {
                    SetupPopoverControls(model: model)
                    Divider()
                    HStack(spacing: 4) {
                        Text("checked 1s ago")
                        Spacer(minLength: 4)
                        SetupFooterButton(model: model)
                        Text("· Open TUI · Quit")
                    }.font(.system(size: 11)).foregroundStyle(boardColor(palette.dim)).padding(12)
                }.frame(width: 360).foregroundStyle(boardColor(palette.foreground)).background(boardColor(palette.background)))
            } else {
                view = AnyView(VStack(spacing: 0) {
                    Text("Set up multi-link").font(.system(size: 13, weight: .semibold))
                        .frame(maxWidth: .infinity).padding(10)
                    Divider()
                    SetupView(model: model, poll: false)
                }.frame(width: 520).foregroundStyle(boardColor(palette.foreground)).background(boardColor(palette.background)))
            }
            let renderer = ImageRenderer(content: view.environment(\.colorScheme, dark ? .dark : .light))
            renderer.scale = 2
            guard let image = renderer.cgImage else { throw RenderFailure.denied }
            let bitmap = NSBitmapImageRep(cgImage: image)
            guard let png = bitmap.representation(using: .png, properties: [:]) else { throw RenderFailure.denied }
            let file = dir.appendingPathComponent("\(state)-\(dark ? "dark" : "light").png")
            try png.write(to: file)
            print(file.lastPathComponent)
        }
    }
}
