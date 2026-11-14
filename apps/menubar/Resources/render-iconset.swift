#!/usr/bin/env swift
import AppKit

// Run with: swift apps/menubar/Resources/render-iconset.swift
// Build packaging uses these committed PNGs; it never invokes this renderer.
let resources = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let source = resources.appendingPathComponent("AppIcon.svg")
guard let image = NSImage(contentsOf: source), image.isValid else {
    fatalError("Cannot load AppIcon.svg")
}
let iconset = resources.appendingPathComponent("AppIcon.iconset")
try FileManager.default.createDirectory(at: iconset, withIntermediateDirectories: true)
for points in [16, 32, 128, 256, 512] {
    for scale in [1, 2] {
        let pixels = points * scale
        guard let bitmap = NSBitmapImageRep(
            bitmapDataPlanes: nil, pixelsWide: pixels, pixelsHigh: pixels,
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0
        ), let context = NSGraphicsContext(bitmapImageRep: bitmap) else {
            fatalError("Cannot allocate \(pixels) px bitmap")
        }
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = context
        context.imageInterpolation = .high
        let rect = NSRect(x: 0, y: 0, width: pixels, height: pixels)
        NSColor.clear.setFill()
        rect.fill(using: .copy)
        image.draw(in: rect, from: .zero, operation: .copy, fraction: 1,
                   respectFlipped: false, hints: nil)
        NSGraphicsContext.restoreGraphicsState()
        guard let png = bitmap.representation(using: .png, properties: [:]) else {
            fatalError("Cannot encode \(pixels) px PNG")
        }
        let suffix = scale == 2 ? "@2x" : ""
        let name = "icon_\(points)x\(points)\(suffix).png"
        try png.write(to: iconset.appendingPathComponent(name))
        print("\(name): \(pixels)x\(pixels)")
    }
}
