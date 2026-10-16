import AppKit
import Foundation

// Run from WindowsPort on macOS to regenerate the embedded reference assets.
enum SAExportConnectionAssets {
    static func run() throws {
        let directory = URL(fileURLWithPath: "assets/connection", isDirectory: true)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        let symbols = [
            "structure": "scale.3d", "content": "text.justify",
            "relations": "arrow.2.squarepath", "triggers": "bolt.circle",
            "info": "info.circle", "query": "terminal", "users": "person.3",
            "console": "text.and.command.macwindow", "search": "magnifyingglass",
            "eye": "eye", "eye-slash": "eye.slash"
        ]
        for (name, symbol) in symbols {
            guard let image = NSImage(systemSymbolName: symbol, accessibilityDescription: nil)?
                .withSymbolConfiguration(.init(pointSize: 24, weight: .regular)) else {
                throw NSError(domain: symbol, code: 1)
            }
            try write(image, name: name, to: directory)
        }
        guard let quick = NSImage(contentsOfFile: "../Resources/Images/quick-connect-icon.pdf") else {
            throw NSError(domain: "quick-connect-icon", code: 1)
        }
        try write(quick, name: "quick-connect", to: directory)
    }

    static func write(_ image: NSImage, name: String, to directory: URL) throws {
        let bitmap = NSBitmapImageRep(bitmapDataPlanes: nil, pixelsWide: 96, pixelsHigh: 96,
            bitsPerSample: 8, samplesPerPixel: 4, hasAlpha: true, isPlanar: false,
            colorSpaceName: .deviceRGB, bytesPerRow: 0, bitsPerPixel: 0)!
        NSGraphicsContext.saveGraphicsState()
        NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: bitmap)
        let ratio = min(88 / image.size.width, 88 / image.size.height)
        let size = NSSize(width: image.size.width * ratio, height: image.size.height * ratio)
        image.draw(in: NSRect(x: (96-size.width)/2, y: (96-size.height)/2,
                             width: size.width, height: size.height))
        NSGraphicsContext.restoreGraphicsState()
        try bitmap.representation(using: .png, properties: [:])!
            .write(to: directory.appendingPathComponent(name + ".png"))
    }
}
try SAExportConnectionAssets.run()
