import CoreGraphics
import Foundation

/// OCRRegionMapper OCR 区域映射器
/// 核心职责：
/// - 从应用宝 Mac 截图中定位游戏内容区域
/// - 将标准 1920x1080 游戏坐标映射为实际截图像素 ROI
final class OCRRegionMapper {
    private let standardSize = CGSize(width: 1920, height: 1080)

    /// 计算游戏内容区域
    func gameContentRect(imageSize: CGSize, profile: OCRRegionProfile = .appBaoMac) -> CGRect {
        switch profile {
        case .appBaoMac:
            let titleBarHeight: CGFloat = imageSize.width >= 3840 ? 68 : 0
            let contentHeight = max(0, imageSize.height - titleBarHeight)
            return CGRect(x: 0, y: titleBarHeight, width: imageSize.width, height: contentHeight)
        }
    }

    /// 从截图像素计算游戏内容区域
    func gameContentRect(cgImage: CGImage, profile: OCRRegionProfile = .appBaoMac) -> CGRect {
        switch profile {
        case .appBaoMac:
            let imageSize = CGSize(width: cgImage.width, height: cgImage.height)
            let titleBarHeight = hasLightMacTitleBar(cgImage) ? titleBarHeight(for: imageSize) : 0
            let contentHeight = max(0, imageSize.height - titleBarHeight)
            return CGRect(x: 0, y: titleBarHeight, width: imageSize.width, height: contentHeight)
        }
    }

    /// 生成截图 OCR 区域
    func regions(imageSize: CGSize, profile: OCRRegionProfile = .appBaoMac) -> [OCRRegion] {
        let contentRect = gameContentRect(imageSize: imageSize, profile: profile)
        return regions(contentRect: contentRect, profile: profile)
    }

    /// 生成截图 OCR 区域
    func regions(cgImage: CGImage, profile: OCRRegionProfile = .appBaoMac) -> [OCRRegion] {
        let contentRect = gameContentRect(cgImage: cgImage, profile: profile)
        return regions(contentRect: contentRect, profile: profile)
    }

    private func regions(contentRect: CGRect, profile: OCRRegionProfile) -> [OCRRegion] {
        return standardTemplates(profile: profile, contentRect: contentRect).map { template in
            OCRRegion(
                id: template.id,
                kind: template.kind,
                rect: mapStandardRect(template.rect, into: contentRect)
            )
        }
    }

    private func titleBarHeight(for imageSize: CGSize) -> CGFloat {
        imageSize.width >= 3000 ? 68 : 34
    }

    private func hasLightMacTitleBar(_ cgImage: CGImage) -> Bool {
        let sampleHeight = min(12, cgImage.height)
        guard sampleHeight > 0,
              let crop = cgImage.cropping(to: CGRect(x: 0, y: 0, width: cgImage.width, height: sampleHeight))
        else { return false }

        let sampleWidth = 64
        let bytesPerPixel = 4
        let bytesPerRow = sampleWidth * bytesPerPixel
        var pixels = Data(count: sampleWidth * bytesPerPixel)
        let colorSpace = CGColorSpaceCreateDeviceRGB()

        pixels.withUnsafeMutableBytes { rawBuffer in
            guard let base = rawBuffer.baseAddress else { return }
            let context = CGContext(
                data: base,
                width: sampleWidth,
                height: 1,
                bitsPerComponent: 8,
                bytesPerRow: bytesPerRow,
                space: colorSpace,
                bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            )
            context?.interpolationQuality = .none
            context?.draw(crop, in: CGRect(x: 0, y: 0, width: sampleWidth, height: 1))
        }

        let totals = pixels.withUnsafeBytes { rawBuffer -> (Int, Int, Int) in
            guard let base = rawBuffer.baseAddress?.assumingMemoryBound(to: UInt8.self) else {
                return (0, 0, 0)
            }
            var red = 0
            var green = 0
            var blue = 0
            for index in 0..<sampleWidth {
                let offset = index * bytesPerPixel
                red += Int(base[offset])
                green += Int(base[offset + 1])
                blue += Int(base[offset + 2])
            }
            return (red, green, blue)
        }

        let red = totals.0 / sampleWidth
        let green = totals.1 / sampleWidth
        let blue = totals.2 / sampleWidth
        let brightness = (red + green + blue) / 3
        let chroma = max(red, green, blue) - min(red, green, blue)
        return brightness >= 220 && chroma <= 18
    }

    private func mapStandardRect(_ rect: CGRect, into contentRect: CGRect) -> CGRect {
        let scaleX = contentRect.width / standardSize.width
        let scaleY = contentRect.height / standardSize.height
        return CGRect(
            x: contentRect.minX + rect.minX * scaleX,
            y: contentRect.minY + rect.minY * scaleY,
            width: rect.width * scaleX,
            height: rect.height * scaleY
        ).integral
    }

    private func standardTemplates(profile: OCRRegionProfile, contentRect: CGRect) -> [OCRRegion] {
        switch profile {
        case .appBaoMac:
            return isInGameHud(contentRect) ? appBaoMacInGameHudTemplates() : appBaoMacShopHudTemplates()
        }
    }

    private func isInGameHud(_ contentRect: CGRect) -> Bool {
        contentRect.width < 3840
    }

    private func appBaoMacShopHudTemplates() -> [OCRRegion] {
        var regions: [OCRRegion] = [
            OCRRegion(id: "round", kind: .round, rect: CGRect(x: 700, y: 0, width: 125, height: 48)),
        ]

        let shopX: [CGFloat] = [405, 715, 1025, 1335, 1645]
        for (index, x) in shopX.enumerated() {
            regions.append(OCRRegion(
                id: "shop_hero_name_\(index)",
                kind: .shopHeroName,
                rect: CGRect(x: x, y: 1014, width: 145, height: 32)
            ))
            regions.append(OCRRegion(
                id: "shop_trait_name_\(index)",
                kind: .shopTraitName,
                rect: CGRect(x: x - 4, y: 910, width: 118, height: 72)
            ))
        }

        let opponentRows: [CGFloat] = [74, 154, 244, 334, 424, 514, 604, 694]
        for (index, y) in opponentRows.enumerated() {
            regions.append(OCRRegion(
                id: "opponent_name_\(index)",
                kind: .opponentName,
                rect: CGRect(x: 1455, y: y - 4, width: 250, height: 34)
            ))
            regions.append(OCRRegion(
                id: "opponent_hp_\(index)",
                kind: .opponentHP,
                rect: CGRect(x: 1618, y: y + 34, width: 88, height: 36)
            ))
        }

        let traitRows: [CGFloat] = [98, 166, 234, 302, 370, 438, 506, 574]
        for (index, y) in traitRows.enumerated() {
            regions.append(OCRRegion(
                id: "active_trait_name_\(index)",
                kind: .activeTraitName,
                rect: CGRect(x: 145, y: y, width: 135, height: 32)
            ))
            regions.append(OCRRegion(
                id: "active_trait_count_\(index)",
                kind: .activeTraitCount,
                rect: CGRect(x: 142, y: y + 30, width: 95, height: 26)
            ))
        }

        let augmentX: [CGFloat] = [430, 830, 1230]
        for (index, x) in augmentX.enumerated() {
            regions.append(OCRRegion(
                id: "augment_name_\(index)",
                kind: .augmentName,
                rect: CGRect(x: x, y: 465, width: 260, height: 52)
            ))
        }

        return regions
    }

    private func appBaoMacInGameHudTemplates() -> [OCRRegion] {
        var regions: [OCRRegion] = [
            OCRRegion(id: "round", kind: .round, rect: CGRect(x: 640, y: 0, width: 170, height: 56)),
        ]

        let shopX: [CGFloat] = [462, 705, 950, 1190, 1435]
        for (index, x) in shopX.enumerated() {
            regions.append(OCRRegion(
                id: "shop_hero_name_\(index)",
                kind: .shopHeroName,
                rect: CGRect(x: x, y: 1040, width: 140, height: 36)
            ))
            regions.append(OCRRegion(
                id: "shop_trait_name_\(index)",
                kind: .shopTraitName,
                rect: CGRect(x: x + 8, y: 986, width: 125, height: 42)
            ))
        }

        let opponentRows: [CGFloat] = [74, 154, 244, 334, 424, 514, 604, 694]
        for (index, y) in opponentRows.enumerated() {
            regions.append(OCRRegion(
                id: "opponent_name_\(index)",
                kind: .opponentName,
                rect: CGRect(x: 1612, y: y - 2, width: 170, height: 34)
            ))
            regions.append(OCRRegion(
                id: "opponent_hp_\(index)",
                kind: .opponentHP,
                rect: CGRect(x: 1732, y: y + 28, width: 60, height: 40)
            ))
        }

        let traitRows: [CGFloat] = [96, 166, 236, 306, 376, 446, 516, 586]
        for (index, y) in traitRows.enumerated() {
            regions.append(OCRRegion(
                id: "active_trait_name_\(index)",
                kind: .activeTraitName,
                rect: CGRect(x: 72, y: y - 4, width: 175, height: 34)
            ))
            regions.append(OCRRegion(
                id: "active_trait_count_\(index)",
                kind: .activeTraitCount,
                rect: CGRect(x: 80, y: y + 28, width: 105, height: 30)
            ))
        }

        let augmentX: [CGFloat] = [400, 830, 1260]
        for (index, x) in augmentX.enumerated() {
            regions.append(OCRRegion(
                id: "augment_name_\(index)",
                kind: .augmentName,
                rect: CGRect(x: x, y: 465, width: 300, height: 58)
            ))
        }

        return regions
    }
}
