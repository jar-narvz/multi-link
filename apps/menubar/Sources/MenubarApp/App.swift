import AppKit
import MenubarCore
import SwiftUI

@main
struct MenubarApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) private var delegate
    private var controller: AppController { delegate.controller }

    var body: some Scene {
        MenuBarExtra {
            Popover(controller: controller)
        } label: {
            Text(controller.model.iconText)
                .font(.system(size: 12, weight: .semibold, design: .monospaced))
        }
        .menuBarExtraStyle(.window)
    }
}

@MainActor
final class AppDelegate: NSObject, NSApplicationDelegate {
    let controller = AppController()
    func applicationDidFinishLaunching(_ notification: Notification) {
        Task { await controller.start() }
    }
    func applicationShouldTerminate(_ sender: NSApplication) -> NSApplication.TerminateReply {
        Task {
            await controller.quit()
            sender.reply(toApplicationShouldTerminate: true)
        }
        return .terminateLater
    }
}

private struct Popover: View {
    let controller: AppController
    @Environment(\.colorScheme) private var scheme
    private var palette: BoardPalette { BoardPalette(dark: scheme == .dark) }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            ScrollView {
                VStack(alignment: .leading, spacing: 0) {
                    ForEach(controller.model.blocks) { block in
                        Text(block.header.uppercased())
                            .font(.system(size: 10, weight: .semibold))
                            .tracking(0.7)
                            .foregroundStyle(color(palette.dim))
                            .padding(.horizontal, 12).padding(.top, 10).padding(.bottom, 2)
                        ForEach(Array(block.rows.enumerated()), id: \.offset) { _, row in
                            SessionRow(row: row, palette: palette) {
                                Task { await controller.activate(row) }
                            }
                        }
                    }
                }
                .padding(.bottom, 6)
            }
            .frame(maxHeight: 440)
            if controller.model.showsNotificationsOff {
                HStack {
                    Text("Notifications off")
                    Spacer()
                    Button("Open Settings") { controller.openNotificationSettings() }
                    Button { controller.model.dismissNotificationsOff() } label: { Image(systemName: "xmark") }
                        .help("Dismiss")
                }
                .font(.system(size: 11)).padding(12)
            }
            Rectangle().fill(color(palette.line)).frame(height: 1)
            TimelineView(.periodic(from: .now, by: 1)) { context in
                HStack(alignment: .top, spacing: 4) {
                    Text(controller.model.footer(now: context.date)).fixedSize(horizontal: false, vertical: true)
                    Spacer(minLength: 4)
                    Button("Open TUI") { Task { await controller.openTUI() } }
                    Text("·")
                    Button("Quit") { NSApplication.shared.terminate(nil) }
                }
                .font(.system(size: 11)).foregroundStyle(color(palette.dim))
                .buttonStyle(.plain).padding(.horizontal, 12).padding(.vertical, 8)
            }
        }
        .frame(width: 360)
        .foregroundStyle(color(palette.foreground))
        .background(color(palette.background))
        .task { await controller.popoverOpened() }
    }
}

private struct SessionRow: View {
    let row: RowPresentation
    let palette: BoardPalette
    let activate: () -> Void
    @State private var hovering = false

    var body: some View {
        Button(action: activate) {
            HStack(alignment: .firstTextBaseline, spacing: 6) {
                Text(row.symbol).foregroundStyle(color(palette.color(row.tint))).frame(width: 14)
                Text(row.label).lineLimit(1).truncationMode(.tail).layoutPriority(1)
                Spacer(minLength: 0)
                Text(row.phrase).font(.system(size: 11)).monospacedDigit()
                    .foregroundStyle(color(palette.dim)).lineLimit(1).frame(maxWidth: 132, alignment: .trailing)
            }
            .font(.system(size: 12))
            .padding(.leading, row.isChild ? 26 : 12).padding(.trailing, 12).padding(.vertical, 3)
            .contentShape(Rectangle())
            .background(hovering ? color(palette.selection) : .clear)
        }
        .buttonStyle(.plain)
        .onHover { hovering = $0 }
        .help(row.action.title)
        .accessibilityLabel("\(row.label), \(row.phrase), \(row.action.title)")
    }
}

private func color(_ rgb: UInt32) -> Color {
    Color(red: Double((rgb >> 16) & 255) / 255, green: Double((rgb >> 8) & 255) / 255,
          blue: Double(rgb & 255) / 255)
}
