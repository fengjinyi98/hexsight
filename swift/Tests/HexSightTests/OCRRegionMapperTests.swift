import XCTest
@testable import HexSight

/// OCRRegionMapperTests OCR 区域映射测试
/// 核心职责：
/// - 校验应用宝 Mac 窗口截图能映射到标准游戏画面区域
/// - 校验商店、玩家栏、羁绊栏等 OCR 区域随截图尺寸缩放
final class OCRRegionMapperTests: XCTestCase {
    func testGameContentRectRemovesMacTitleBar() {
        let mapper = OCRRegionMapper()

        let rect = mapper.gameContentRect(imageSize: CGSize(width: 3840, height: 2414))

        XCTAssertEqual(rect.origin.x, 0, accuracy: 0.1)
        XCTAssertEqual(rect.origin.y, 68, accuracy: 0.1)
        XCTAssertEqual(rect.width, 3840, accuracy: 0.1)
        XCTAssertEqual(rect.height, 2346, accuracy: 0.1)
    }

    func testGameContentRectDetectsLightMacTitleBarFromPixels() {
        let mapper = OCRRegionMapper()

        let rect = mapper.gameContentRect(cgImage: makeImage(width: 3840, height: 2414, topColor: (241, 242, 246)))

        XCTAssertEqual(rect.origin.y, 68, accuracy: 0.1)
        XCTAssertEqual(rect.height, 2346, accuracy: 0.1)
    }

    func testGameContentRectKeepsFullHeightWhenScreenshotHasNoTitleBar() {
        let mapper = OCRRegionMapper()

        let rect = mapper.gameContentRect(cgImage: makeImage(width: 3796, height: 2322, topColor: (75, 84, 68)))

        XCTAssertEqual(rect.origin.y, 0, accuracy: 0.1)
        XCTAssertEqual(rect.height, 2322, accuracy: 0.1)
    }

    func testShopNameRegionsScaleFromStandardGameCoordinates() {
        let mapper = OCRRegionMapper()

        let regions = mapper.regions(imageSize: CGSize(width: 3840, height: 2414), profile: .appBaoMac)
        let shopNames = regions.filter { $0.kind == .shopHeroName }

        XCTAssertEqual(shopNames.count, 5)
        XCTAssertEqual(shopNames[0].rect.origin.x, 810, accuracy: 3)
        XCTAssertEqual(shopNames[0].rect.origin.y, 2270, accuracy: 3)
        XCTAssertEqual(shopNames[0].rect.width, 290, accuracy: 3)
        XCTAssertEqual(shopNames[0].rect.height, 71, accuracy: 3)
        XCTAssertEqual(shopNames[4].rect.origin.x, 3290, accuracy: 3)
    }

    func testOpponentRegionsExistForVisibleSidebarRows() {
        let mapper = OCRRegionMapper()

        let regions = mapper.regions(imageSize: CGSize(width: 3796, height: 2322), profile: .appBaoMac)
        let opponentNames = regions.filter { $0.kind == .opponentName }
        let opponentHp = regions.filter { $0.kind == .opponentHP }

        XCTAssertEqual(opponentNames.count, 8)
        XCTAssertEqual(opponentHp.count, 8)
        XCTAssertTrue(opponentNames.allSatisfy { $0.rect.maxX <= 3796 })
        XCTAssertTrue(opponentHp.allSatisfy { $0.rect.maxX <= 3796 })
    }

    func testInGameHudRegionsTargetVisibleTextInRealSamples() {
        let mapper = OCRRegionMapper()

        let regions = mapper.regions(imageSize: CGSize(width: 3796, height: 2322), profile: .appBaoMac)
        let firstShopName = regions.first { $0.id == "shop_hero_name_0" }!.rect
        let firstTraitName = regions.first { $0.id == "active_trait_name_0" }!.rect
        let firstOpponentName = regions.first { $0.id == "opponent_name_0" }!.rect
        let firstOpponentHP = regions.first { $0.id == "opponent_hp_0" }!.rect

        XCTAssertEqual(firstShopName.origin.x, 914, accuracy: 12)
        XCTAssertEqual(firstShopName.origin.y, 2235, accuracy: 12)
        XCTAssertLessThan(firstTraitName.origin.x, 190)
        XCTAssertGreaterThan(firstTraitName.width, 330)
        XCTAssertGreaterThan(firstOpponentName.origin.x, 3180)
        XCTAssertGreaterThan(firstOpponentHP.origin.x, 3420)
    }

    func testAugmentNameRegionsTargetCardTitleBandInRealSamples() {
        let mapper = OCRRegionMapper()
        let image = makeImage(width: 4064, height: 2640, topColor: (40, 60, 80))

        let regions = mapper.regions(cgImage: image, profile: .appBaoMac)
        let augmentNames = regions.filter { $0.kind == .augmentName }

        XCTAssertEqual(augmentNames.count, 3)
        XCTAssertEqual(augmentNames[0].rect.origin.x, 910, accuracy: 8)
        XCTAssertEqual(augmentNames[0].rect.origin.y, 1137, accuracy: 16)
        XCTAssertEqual(augmentNames[0].rect.width, 551, accuracy: 8)
        XCTAssertLessThan(augmentNames[0].rect.maxY, 1270)
    }

    private func makeImage(width: Int, height: Int, topColor: (UInt8, UInt8, UInt8)) -> CGImage {
        let colorSpace = CGColorSpaceCreateDeviceRGB()
        let bytesPerRow = width * 4
        var pixels = Data(count: bytesPerRow * height)
        pixels.withUnsafeMutableBytes { rawBuffer in
            guard let base = rawBuffer.baseAddress?.assumingMemoryBound(to: UInt8.self) else { return }
            for y in 0..<height {
                for x in 0..<width {
                    let offset = y * bytesPerRow + x * 4
                    if y < 68 {
                        base[offset] = topColor.0
                        base[offset + 1] = topColor.1
                        base[offset + 2] = topColor.2
                    } else {
                        base[offset] = 40
                        base[offset + 1] = 60
                        base[offset + 2] = 80
                    }
                    base[offset + 3] = 255
                }
            }
        }
        let provider = CGDataProvider(data: pixels as CFData)!
        return CGImage(
            width: width,
            height: height,
            bitsPerComponent: 8,
            bitsPerPixel: 32,
            bytesPerRow: bytesPerRow,
            space: colorSpace,
            bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue),
            provider: provider,
            decode: nil,
            shouldInterpolate: false,
            intent: .defaultIntent
        )!
    }
}
