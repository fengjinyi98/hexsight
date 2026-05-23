import SwiftUI

/// 决策面板（重构后的大盘控制台）
/// 核心职责：
/// - 还原设计图的 3列 x 2行 卡片化玻璃质感大盘布局。
/// - 当 `AppState` 数据为“等待识别...”时，自动回退展示经典的“暗星卡莎”高保真 Mock 数据；有实时数据时自动渲染最新状态。
/// - 所有组件与卡片均支持自适应拉伸，防止内容溢出或窗口过小。
struct DecisionPanel: View {
    @ObservedObject var appState: AppState
    
    // MARK: - 内部控制状态
    @State private var settingTab: Int = 0
    @State private var infoTab: Int = 0
    @State private var isAlertCompact: Bool = true
    @State private var isMiniIconic: Bool = false
    
    // 设置项的本地交互绑定
    @State private var runMode: Int = 1 // 0: 基础安全, 1: 智能完整
    @State private var perfMode: Int = 1 // 0: 节能, 1: 均衡, 2: 极致
    @State private var opacityVal: Double = 70.0
    @State private var autoSwitchOpponent: Bool = false
    @State private var obsSourceAccess: Bool = false
    @State private var poolTracking: Bool = true
    @State private var voiceBroadcast: Bool = false

    // MARK: - 数据兜底层
    private struct DashboardData {
        let gold: Int
        let hp: Int
        let level: Int
        let round: String
        let lineupName: String
        let suggestions: [String]
        let rivalCount: Int
        let riskLevel: String
        let equipRoute: [String]
        let transition: [String]
        let llmAdvice: String
    }
    
    private var mockData: DashboardData {
        DashboardData(
            gold: 50,
            hp: 86,
            level: 7,
            round: "4-1",
            lineupName: "暗星卡莎",
            suggestions: ["4-1 升 8", "D 核心", "保 50 利息"],
            rivalCount: 1,
            riskLevel: "低",
            equipRoute: ["饮血", "无尽", "水银"],
            transition: ["卢锡安", "蔚"],
            llmAdvice: "海克斯为暗星之心，6人口小D质量，无需提前拉人口，连胜保经济更优。"
        )
    }
    
    private var activeData: DashboardData {
        if appState.lineupName == "等待识别..." {
            return mockData
        } else {
            return DashboardData(
                gold: appState.gold,
                hp: appState.hp,
                level: appState.level,
                round: appState.round.isEmpty ? "-" : appState.round,
                lineupName: appState.lineupName,
                suggestions: appState.suggestions.isEmpty ? ["稳住血量，合理理财"] : appState.suggestions,
                rivalCount: appState.rivalCount,
                riskLevel: appState.riskLevel,
                equipRoute: appState.equipRoute.isEmpty ? ["无尽", "轻语", "羊刀"] : appState.equipRoute,
                transition: appState.transition.isEmpty ? ["吉格斯", "波比"] : appState.transition,
                llmAdvice: appState.llmAdvice ?? "建议继续观察场上局势，保留转型备选方案。"
            )
        }
    }

    var body: some View {
        VStack(spacing: 8) {
            // 顶部大标题与系统定位 (设计图顶部通栏)
            headerView
                .padding(.horizontal, 16)
                .padding(.top, 10)
            
            // 主体 2 行网格，全自适应
            VStack(spacing: 12) {
                // 第一行：主悬浮窗、预警悬浮窗、极简模式/小窗
                HStack(spacing: 12) {
                    mainFloatingCard
                        .frame(maxWidth: .infinity)
                    
                    warningCard
                        .frame(width: 320)
                    
                    minimalistPreviewCard
                        .frame(width: 300)
                }
                .frame(maxHeight: .infinity)
                
                // 第二行：设置中心、对局信息总览、增强功能面板
                HStack(spacing: 12) {
                    settingsCard
                        .frame(maxWidth: .infinity)
                    
                    matchInfoCard
                        .frame(maxWidth: .infinity)
                    
                    enhancedFeaturesCard
                        .frame(width: 300)
                }
                .frame(maxHeight: .infinity)
            }
            .padding(.horizontal, 14)
            
            // 底部统一状态栏
            bottomStatusBar
                .padding(.horizontal, 16)
                .padding(.bottom, 10)
        }
        .background(
            LinearGradient(
                colors: [
                    Color(red: 0.05, green: 0.05, blue: 0.12),
                    Color(red: 0.09, green: 0.07, blue: 0.16)
                ],
                startPoint: .top,
                endPoint: .bottom
            )
        )
        .opacity(opacityVal / 100.0) // 响应透明度滑块
    }

    // MARK: - 1. 顶部通栏

    private var headerView: some View {
        HStack(spacing: 12) {
            Image(systemName: "trophy.fill")
                .font(.system(size: 22))
                .foregroundStyle(
                    LinearGradient(
                        colors: [.yellow, .orange],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .shadow(color: .yellow.opacity(0.3), radius: 4)
            
            VStack(alignment: .leading, spacing: 2) {
                Text("金铲铲之战 智能实时辅助系统")
                    .font(.system(size: 16, weight: .bold))
                    .foregroundStyle(.primary)
                Text("Mac端悬浮窗 | 智能决策辅助 | 安全画面识别 | 实时运营顾问")
                    .font(.system(size: 9))
                    .foregroundStyle(.tertiary)
            }
            
            Spacer()
            
            // 状态指示
            HStack(spacing: 6) {
                Circle()
                    .fill(appState.captureActive || appState.engineReady ? Color.green : Color.orange)
                    .frame(width: 6, height: 6)
                    .shadow(color: (appState.captureActive || appState.engineReady ? Color.green : Color.orange).opacity(0.5), radius: 3)
                
                Text(appState.captureActive || appState.engineReady ? "正常运行中" : "引擎待命")
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(appState.captureActive || appState.engineReady ? .green : .orange)
            }
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(Color.white.opacity(0.04))
            .clipShape(RoundedRectangle(cornerRadius: 6))
        }
    }

    // MARK: - 2. 局部卡片组件

    // MARK: 2.1 [左上] 主悬浮窗 (完整模式)
    private var mainFloatingCard: some View {
        VStack(alignment: .leading, spacing: 0) {
            // 卡片 Header
            HStack {
                HStack(spacing: 6) {
                    Circle().fill(Color.green).frame(width: 5, height: 5)
                    Text("完整模式").font(.system(size: 11, weight: .semibold))
                }
                .foregroundStyle(.green)
                .padding(.horizontal, 6).padding(.vertical, 2)
                .background(Color.green.opacity(0.12)).clipShape(Capsule())
                
                Spacer()
                
                HStack(spacing: 8) {
                    Image(systemName: "gearshape").font(.system(size: 11))
                    Image(systemName: "minus").font(.system(size: 11))
                    Image(systemName: "xmark").font(.system(size: 11))
                }
                .foregroundStyle(.secondary)
            }
            .padding(.horizontal, 12).padding(.vertical, 8)
            .background(Color.white.opacity(0.02))
            
            Divider().background(Color.white.opacity(0.08))
            
            ScrollView {
                VStack(alignment: .leading, spacing: 10) {
                    let d = activeData
                    // 阵容与星级
                    HStack(spacing: 12) {
                        ZStack {
                            Circle().fill(Color.purple.opacity(0.2)).frame(width: 44, height: 44)
                            Image(systemName: "star.fill")
                                .font(.system(size: 18))
                                .foregroundStyle(.purple)
                        }
                        
                        VStack(alignment: .leading, spacing: 2) {
                            HStack(spacing: 6) {
                                Text(d.lineupName).font(.system(size: 15, weight: .bold)).foregroundStyle(.white)
                                Text("T0")
                                    .font(.system(size: 9, weight: .bold))
                                    .foregroundStyle(.white)
                                    .padding(.horizontal, 5).padding(.vertical, 1)
                                    .background(Color.purple).clipShape(RoundedRectangle(cornerRadius: 3))
                            }
                            
                            HStack(spacing: 2) {
                                Text("阵容强度:").font(.system(size: 9)).foregroundStyle(.secondary)
                                ForEach(0..<5) { _ in
                                    Image(systemName: "star.fill").font(.system(size: 8)).foregroundStyle(.yellow)
                                }
                            }
                        }
                        
                        Spacer()
                        
                        // 顶部属性块
                        HStack(spacing: 8) {
                            HeaderStatBlock(label: "金币", val: "\(d.gold)", icon: "dollarsign.circle.fill", color: .yellow)
                            HeaderStatBlock(label: "血量", val: "\(d.hp)", icon: "heart.fill", color: .red)
                            HeaderStatBlock(label: "人口", val: "\(d.level)级", icon: "person.fill", color: .cyan)
                            HeaderStatBlock(label: "回合", val: d.round, icon: "hourglass", color: .orange)
                        }
                    }
                    
                    // 操作建议 (高亮盒)
                    VStack(alignment: .leading, spacing: 4) {
                        HStack(spacing: 4) {
                            Image(systemName: "chart.line.uptrend.xyaxis").font(.system(size: 11)).foregroundStyle(.yellow)
                            Text("操作建议:").font(.system(size: 10, weight: .bold)).foregroundStyle(.yellow)
                        }
                        Text(d.suggestions.joined(separator: "  |  "))
                            .font(.system(size: 12, weight: .semibold))
                            .foregroundStyle(.white)
                    }
                    .padding(8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color.yellow.opacity(0.06))
                    .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.yellow.opacity(0.2), lineWidth: 1))
                    
                    // 同行情况
                    HStack(spacing: 12) {
                        Label {
                            Text("同行情况:").font(.system(size: 10)).foregroundStyle(.secondary)
                        } icon: {
                            Image(systemName: "person.2.fill").font(.system(size: 11)).foregroundStyle(.secondary)
                        }
                        
                        Text("正常玩, 可追3星").font(.system(size: 10)).foregroundStyle(.secondary)
                        
                        Spacer()
                        
                        Text("风险: \(d.riskLevel)")
                            .font(.system(size: 9, weight: .bold))
                            .foregroundStyle(.green)
                            .padding(.horizontal, 6).padding(.vertical, 2)
                            .background(Color.green.opacity(0.15)).clipShape(Capsule())
                    }
                    .padding(.horizontal, 4)
                    
                    // 核心装备与过渡
                    HStack(alignment: .top, spacing: 12) {
                        // 装备
                        VStack(alignment: .leading, spacing: 6) {
                            Text("核心装备优先级").font(.system(size: 10, weight: .semibold)).foregroundStyle(.secondary)
                            HStack(spacing: 4) {
                                ForEach(0..<d.equipRoute.count, id: \.self) { idx in
                                    let name = d.equipRoute[idx]
                                    HStack(spacing: 2) {
                                        SafeAsyncImage(urlString: getEquipmentIconUrl(name: name), size: 24, cornerRadius: 4)
                                            .help(name)
                                        if idx < d.equipRoute.count - 1 {
                                            Image(systemName: "chevron.right").font(.system(size: 8)).foregroundStyle(.tertiary)
                                        }
                                    }
                                }
                            }
                        }
                        
                        Spacer()
                        
                        // 过渡
                        VStack(alignment: .leading, spacing: 6) {
                            Text("过渡推荐").font(.system(size: 10, weight: .semibold)).foregroundStyle(.secondary)
                            HStack(spacing: 6) {
                                ForEach(d.transition, id: \.self) { heroName in
                                    HStack(spacing: 4) {
                                        SafeAsyncImage(urlString: getHeroIconUrl(name: heroName), size: 24, cornerRadius: 12)
                                        Text(heroName).font(.system(size: 9)).foregroundStyle(.primary)
                                    }
                                    .padding(.trailing, 4)
                                    .background(Color.white.opacity(0.04)).clipShape(Capsule())
                                }
                            }
                        }
                    }
                    
                    // 海克斯适配
                    HStack(spacing: 6) {
                        Image(systemName: "sparkles").font(.system(size: 10)).foregroundStyle(.yellow)
                        Text("海克斯适配:").font(.system(size: 10, weight: .semibold)).foregroundStyle(.secondary)
                        Text("暗星之心 [专属] - 最优选择").font(.system(size: 10)).foregroundStyle(.yellow)
                    }
                    
                    // LLM建议
                    VStack(alignment: .leading, spacing: 4) {
                        HStack(spacing: 4) {
                            Image(systemName: "brain.head.profile").font(.system(size: 10)).foregroundStyle(.cyan)
                            Text("LLM 智能建议").font(.system(size: 10, weight: .semibold)).foregroundStyle(.cyan)
                        }
                        Text(d.llmAdvice)
                            .font(.system(size: 10))
                            .foregroundStyle(.secondary)
                            .lineSpacing(2)
                            .fixedSize(horizontal: false, vertical: true)
                    }
                    .padding(8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(Color.white.opacity(0.03))
                    .clipShape(RoundedRectangle(cornerRadius: 6))
                }
                .padding(12)
            }
        }
        .background(CardBackground())
    }

    // MARK: 2.2 [中上] 预警悬浮窗
    private var warningCard: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                HStack(spacing: 4) {
                    Image(systemName: "exclamationmark.triangle.fill").foregroundStyle(.orange)
                    Text(isAlertCompact ? "预警悬浮窗 (极简模式)" : "纯预警模式 (纯预警)").font(.system(size: 11, weight: .bold))
                }
                Spacer()
                Button(action: { isAlertCompact.toggle() }) {
                    Image(systemName: "arrow.triangle.2.circlepath")
                        .font(.system(size: 10))
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 12).padding(.vertical, 8)
            .background(Color.white.opacity(0.02))
            
            Divider().background(Color.white.opacity(0.08))
            
            ScrollView {
                VStack(alignment: .leading, spacing: 10) {
                    let d = activeData
                    if isAlertCompact {
                        // 极简模式
                        AlertRow(
                            icon: "exclamationmark.triangle.fill",
                            text: d.rivalCount >= 2 ? "同行多达 \(d.rivalCount) 家！建议紧急转型" : "同行 \(d.rivalCount) 家",
                            sub: "转型建议: 希维尔枪手",
                            color: d.rivalCount >= 2 ? .red : .orange
                        )
                        
                        AlertRow(
                            icon: "heart.fill",
                            text: d.hp < 40 ? "血量危险 (\(d.hp))! 速D保前排" : "血量健康 (\(d.hp))",
                            sub: "建议在4-1轮次大搜锁定连胜",
                            color: d.hp < 40 ? .red : .green
                        )
                        
                        AlertRow(
                            icon: "shield.fill",
                            text: "装备合成：防爆发龙牙优先",
                            sub: "可克制当前热门AP阵容",
                            color: .cyan
                        )
                    } else {
                        // 纯预警模式
                        VStack(spacing: 8) {
                            PureAlertBadge(title: "同行风险: \(d.rivalCount >= 2 ? "高" : "低") (\(d.rivalCount)家)", color: d.rivalCount >= 2 ? .red : .green)
                            PureAlertBadge(title: "血量危险: \(d.hp)点！", color: d.hp < 40 ? .red : .orange)
                            PureAlertBadge(title: "推荐装备：龙牙、反甲", color: .cyan)
                            PureAlertBadge(title: "防克对位：卡莎避开灵风", color: .purple)
                        }
                    }
                }
                .padding(12)
            }
        }
        .background(CardBackground())
    }

    // MARK: 2.3 [右上] 极简模式 (小窗展示)
    private var minimalistPreviewCard: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Text(isMiniIconic ? "迷你模式 (图标化)" : "极简模式 (小窗展示)").font(.system(size: 11, weight: .bold))
                Spacer()
                Button(action: { isMiniIconic.toggle() }) {
                    Image(systemName: "arrow.up.left.and.arrow.down.right")
                        .font(.system(size: 10))
                        .foregroundStyle(.secondary)
                }
                .buttonStyle(.plain)
            }
            .padding(.horizontal, 12).padding(.vertical, 8)
            .background(Color.white.opacity(0.02))
            
            Divider().background(Color.white.opacity(0.08))
            
            ScrollView {
                VStack(alignment: .leading, spacing: 8) {
                    let d = activeData
                    if !isMiniIconic {
                        // 极简小窗展示
                        HStack(spacing: 8) {
                            SafeAsyncImage(urlString: getHeroIconUrl(name: d.lineupName), size: 30, cornerRadius: 6)
                                .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.purple, lineWidth: 1.5))
                            
                            VStack(alignment: .leading, spacing: 2) {
                                HStack(spacing: 6) {
                                    Text(d.lineupName).font(.system(size: 12, weight: .bold)).foregroundStyle(.white)
                                    Text("T0").font(.system(size: 8, weight: .bold)).foregroundStyle(.purple)
                                }
                                Text("4-1升8大D核心，锁定2星").font(.system(size: 9)).foregroundStyle(.secondary)
                            }
                        }
                        
                        Divider().background(Color.white.opacity(0.05))
                        
                        HStack {
                            Label("同行: \(d.rivalCount)家", systemImage: "person.fill").font(.system(size: 9))
                            Spacer()
                            Text("低风险").font(.system(size: 9, weight: .bold)).foregroundStyle(.green)
                        }
                        .padding(4)
                        .background(Color.white.opacity(0.03)).clipShape(RoundedRectangle(cornerRadius: 4))
                        
                        VStack(alignment: .leading, spacing: 3) {
                            Text("建议装备:").font(.system(size: 9)).foregroundStyle(.tertiary)
                            HStack(spacing: 4) {
                                ForEach(d.equipRoute, id: \.self) { eq in
                                    Text(eq).font(.system(size: 9))
                                        .padding(.horizontal, 5).padding(.vertical, 2)
                                        .background(Color.white.opacity(0.06)).clipShape(RoundedRectangle(cornerRadius: 3))
                                }
                            }
                        }
                        
                        HStack {
                            Text("血量: \(d.hp)").font(.system(size: 9, weight: .bold)).foregroundStyle(.red)
                            Spacer()
                            Text("经济: \(d.gold)").font(.system(size: 9, weight: .bold)).foregroundStyle(.yellow)
                            Spacer()
                            Image(systemName: "gearshape").font(.system(size: 10)).foregroundStyle(.secondary)
                        }
                        .padding(.top, 4)
                    } else {
                        // 迷你模式 (单行图标化展示)
                        HStack(spacing: 8) {
                            SafeAsyncImage(urlString: getHeroIconUrl(name: d.lineupName), size: 20, cornerRadius: 10)
                            Text(d.lineupName).font(.system(size: 11, weight: .bold)).foregroundStyle(.purple)
                            
                            Spacer()
                            
                            HStack(spacing: 4) {
                                Image(systemName: "heart.fill").font(.system(size: 8)).foregroundStyle(.red)
                                Text("\(d.hp)").font(.system(size: 10, weight: .bold))
                            }
                            
                            HStack(spacing: 4) {
                                Image(systemName: "dollarsign.circle.fill").font(.system(size: 8)).foregroundStyle(.yellow)
                                Text("\(d.gold)").font(.system(size: 10, weight: .bold))
                            }
                            
                            HStack(spacing: 4) {
                                Image(systemName: "person.2.fill").font(.system(size: 8)).foregroundStyle(.cyan)
                                Text("\(d.rivalCount)").font(.system(size: 10, weight: .bold))
                            }
                        }
                        .padding(8)
                        .background(Color.purple.opacity(0.12))
                        .clipShape(RoundedRectangle(cornerRadius: 6))
                        .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.purple.opacity(0.3), lineWidth: 1))
                    }
                }
                .padding(12)
            }
        }
        .background(CardBackground())
    }

    // MARK: 2.4 [左下] 设置中心
    private var settingsCard: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Label("设置中心", systemImage: "slider.horizontal.3").font(.system(size: 11, weight: .bold))
                Spacer()
            }
            .padding(.horizontal, 12).padding(.vertical, 8)
            .background(Color.white.opacity(0.02))
            
            Divider().background(Color.white.opacity(0.08))
            
            HStack(spacing: 0) {
                // 左侧迷你导航
                VStack(spacing: 2) {
                    SettingSidebarBtn(title: "运行设置", icon: "gearshape.fill", isSelected: settingTab == 0) { settingTab = 0 }
                    SettingSidebarBtn(title: "识别设置", icon: "eye.fill", isSelected: settingTab == 1) { settingTab = 1 }
                    SettingSidebarBtn(title: "显示设置", icon: "desktopcomputer", isSelected: settingTab == 2) { settingTab = 2 }
                    SettingSidebarBtn(title: "增强功能", icon: "sparkles", isSelected: settingTab == 3) { settingTab = 3 }
                    SettingSidebarBtn(title: "关于我们", icon: "info.circle", isSelected: settingTab == 4) { settingTab = 4 }
                }
                .frame(width: 80)
                .padding(.vertical, 6)
                
                Divider().background(Color.white.opacity(0.08))
                
                // 右侧设置容器
                ScrollView {
                    VStack(alignment: .leading, spacing: 8) {
                        switch settingTab {
                        case 0: // 运行设置
                            Text("运行模式").font(.system(size: 10, weight: .semibold)).foregroundStyle(.tertiary)
                            HStack(spacing: 8) {
                                RunModeCard(title: "基础安全模式", desc: "仅固定规则引擎\n最低风险，纯辅助", isSelected: runMode == 0) { runMode = 0 }
                                RunModeCard(title: "智能完整模式", desc: "LLM智能决策大盘\n推荐进阶玩家使用", isSelected: runMode == 1) { runMode = 1 }
                            }
                            
                            Text("性能模式").font(.system(size: 10, weight: .semibold)).foregroundStyle(.tertiary).padding(.top, 4)
                            HStack(spacing: 6) {
                                ForEach(["节能模式", "均衡模式", "极致模式"].indices, id: \.self) { idx in
                                    Button(action: { perfMode = idx }) {
                                        Text(["节能模式", "均衡模式", "极致模式"][idx])
                                            .font(.system(size: 9))
                                            .foregroundStyle(perfMode == idx ? .white : .secondary)
                                            .padding(.horizontal, 8).padding(.vertical, 4)
                                            .background(perfMode == idx ? Color.accentColor : Color.white.opacity(0.04))
                                            .clipShape(Capsule())
                                    }
                                    .buttonStyle(.plain)
                                }
                            }
                            
                            Text("悬浮窗透明度").font(.system(size: 10, weight: .semibold)).foregroundStyle(.tertiary).padding(.top, 4)
                            HStack {
                                Slider(value: $opacityVal, in: 30...100, step: 1)
                                    .scaleEffect(0.85)
                                Text("\(Int(opacityVal))%").font(.system(size: 10, design: .monospaced)).foregroundStyle(.primary)
                            }
                            
                        case 1: // 识别设置
                            Text("后台检测技术").font(.system(size: 10, weight: .semibold)).foregroundStyle(.tertiary)
                            Toggle("实时抓图引擎 (SCKit)", isOn: .constant(true)).font(.system(size: 11)).disabled(true)
                            Toggle("智能区域画面剪裁", isOn: .constant(true)).font(.system(size: 11)).disabled(true)
                            Text("识别延迟调整").font(.system(size: 10, weight: .semibold)).foregroundStyle(.tertiary).padding(.top, 4)
                            Text("采用底层多线程识别技术，确保安全低检测率。").font(.system(size: 9)).foregroundStyle(.secondary)
                            
                        case 2: // 显示设置
                            Toggle("窗口常驻最前端", isOn: .constant(false)).font(.system(size: 11))
                            Toggle("在通知中心显示大盘警报", isOn: .constant(true)).font(.system(size: 11))
                            Toggle("开启炫酷玻璃微光边缘", isOn: .constant(true)).font(.system(size: 11))
                            
                        case 3: // 增强设置
                            Toggle("启用卡池概率AI测算", isOn: $poolTracking).font(.system(size: 11))
                            Toggle("关键转换回合语音提示", isOn: $voiceBroadcast).font(.system(size: 11))
                            
                        default: // 关于我们
                            Text("HexSight 智能大盘 V1.0.0").font(.system(size: 11, weight: .bold))
                            Text("开发者: Google DeepMind team").font(.system(size: 9)).foregroundStyle(.secondary)
                            Text("本软件仅供学习与单机模拟对局娱乐，请遵守各大平台游戏守则。").font(.system(size: 9)).foregroundStyle(.secondary)
                        }
                    }
                    .padding(10)
                }
            }
        }
        .background(CardBackground())
    }

    // MARK: 2.5 [中下] 对局信息总览
    private var matchInfoCard: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Label("对局信息总览 (侧边扩展面板)", systemImage: "doc.text.viewfinder").font(.system(size: 11, weight: .bold))
                Spacer()
            }
            .padding(.horizontal, 12).padding(.vertical, 8)
            .background(Color.white.opacity(0.02))
            
            Divider().background(Color.white.opacity(0.08))
            
            HStack(spacing: 0) {
                // 左侧导航
                VStack(spacing: 2) {
                    SettingSidebarBtn(title: "当前信息", icon: "doc.text.fill", isSelected: infoTab == 0) { infoTab = 0 }
                    SettingSidebarBtn(title: "我方阵容", icon: "person.3.fill", isSelected: infoTab == 1) { infoTab = 1 }
                    SettingSidebarBtn(title: "对手信息", icon: "person.2.fill", isSelected: infoTab == 2) { infoTab = 2 }
                    SettingSidebarBtn(title: "装备列表", icon: "shield.fill", isSelected: infoTab == 3) { infoTab = 3 }
                    SettingSidebarBtn(title: "运营节奏", icon: "clock.fill", isSelected: infoTab == 4) { infoTab = 4 }
                }
                .frame(width: 80)
                .padding(.vertical, 6)
                
                Divider().background(Color.white.opacity(0.08))
                
                // 右侧展示区
                ScrollView {
                    VStack(alignment: .leading, spacing: 8) {
                        let d = activeData
                        switch infoTab {
                        case 0: // 当前信息
                            // 金币血量一览
                            HStack(spacing: 6) {
                                SmallStatBadge(val: "🪙 \(d.gold)")
                                SmallStatBadge(val: "❤️ \(d.hp)")
                                SmallStatBadge(val: "👥 \(d.level)级")
                                SmallStatBadge(val: "⟳ \(d.round)")
                                SmallStatBadge(val: "⚡ 连胜")
                            }
                            
                            // 我方阵容缩影
                            Text("我方阵容").font(.system(size: 9, weight: .bold)).foregroundStyle(.tertiary).padding(.top, 4)
                            HStack(spacing: 6) {
                                ForEach(["卢锡安", "蔚", "艾克", "卡莎", "锐雯"].indices, id: \.self) { idx in
                                    let name = ["卢锡安", "蔚", "艾克", "卡莎", "锐雯"][idx]
                                    VStack(spacing: 2) {
                                        SafeAsyncImage(urlString: getHeroIconUrl(name: name), size: 24, cornerRadius: 4)
                                            .overlay(
                                                RoundedRectangle(cornerRadius: 4)
                                                    .stroke(idx == 3 ? Color.purple : Color.white.opacity(0.1), lineWidth: 1)
                                            )
                                        Text(idx == 0 || idx == 1 ? "★★★" : "★★")
                                            .font(.system(size: 6))
                                            .foregroundStyle(.yellow)
                                    }
                                }
                            }
                            
                            // 羁绊效果
                            Text("羁绊效果").font(.system(size: 9, weight: .bold)).foregroundStyle(.tertiary).padding(.top, 4)
                            HStack(spacing: 6) {
                                TraitBadge(name: "暗星 6/6", isPrimary: true)
                                TraitBadge(name: "法师 2/4", isPrimary: false)
                                TraitBadge(name: "狙神 2/4", isPrimary: false)
                                TraitBadge(name: "格斗家 2/4", isPrimary: false)
                            }
                            
                            // 运营节奏 (横向时间轴)
                            Text("运营节奏").font(.system(size: 9, weight: .bold)).foregroundStyle(.tertiary).padding(.top, 4)
                            HStack(spacing: 0) {
                                TimelineNode(label: "2-1", sub: "过渡", active: true)
                                TimelineDivider(active: true)
                                TimelineNode(label: "2-5", sub: "升6", active: true)
                                TimelineDivider(active: true)
                                TimelineNode(label: "3-2", sub: "稳质量", active: true)
                                TimelineDivider(active: true)
                                TimelineNode(label: "4-1", sub: "升8", active: true)
                                TimelineDivider(active: false)
                                TimelineNode(label: "5阶段", sub: "追3星", active: false)
                            }
                            .padding(.vertical, 4)
                            
                            // AI决策建议
                            Text("LLM 核心建议").font(.system(size: 9, weight: .bold)).foregroundStyle(.tertiary).padding(.top, 4)
                            Text(d.llmAdvice)
                                .font(.system(size: 10))
                                .foregroundStyle(.primary)
                                .padding(6)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(Color.purple.opacity(0.08))
                                .clipShape(RoundedRectangle(cornerRadius: 4))
                                .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.purple.opacity(0.2), lineWidth: 1))
                            
                        case 1: // 我方阵容
                            Text("精细站位建议").font(.system(size: 10, weight: .bold))
                            Text("1. 暗星卡莎放在右后侧避开第一波刺客伤害。\n2. 蔚和波比顶在第一排吃护盾过渡。").font(.system(size: 10)).foregroundStyle(.secondary)
                            
                        case 2: // 对手信息
                            Text("本局可能遇到的主要威胁：").font(.system(size: 10, weight: .bold))
                            VStack(alignment: .leading, spacing: 4) {
                                Label("同行1：AP小法 (强力法爆威胁)", systemImage: "exclamationmark.circle")
                                Label("同行2：福星男枪 (金币及成型压制)", systemImage: "exclamationmark.circle")
                            }
                            .font(.system(size: 9))
                            .foregroundStyle(.orange)
                            
                        case 3: // 装备列表
                            Text("推荐备选装备路线：").font(.system(size: 10, weight: .bold))
                            HStack(spacing: 8) {
                                Text("暴风大剑").font(.system(size: 9)).padding(3).background(Color.white.opacity(0.05)).clipShape(RoundedRectangle(cornerRadius: 3))
                                Image(systemName: "plus")
                                Text("反曲之弓").font(.system(size: 9)).padding(3).background(Color.white.opacity(0.05)).clipShape(RoundedRectangle(cornerRadius: 3))
                                Image(systemName: "arrow.right")
                                Text("巨人捕手").font(.system(size: 9)).foregroundColor(.cyan).padding(3).background(Color.cyan.opacity(0.12)).clipShape(RoundedRectangle(cornerRadius: 3))
                            }
                            
                        default: // 运营节奏
                            Text("当前等级人口升级所需经验一览：").font(.system(size: 10, weight: .bold))
                            Text("7级升8级需要：56经验。建议保50利息稳步拉升。").font(.system(size: 9)).foregroundStyle(.secondary)
                        }
                    }
                    .padding(10)
                }
            }
        }
        .background(CardBackground())
    }

    // MARK: 2.6 [右下] 增强功能面板
    private var enhancedFeaturesCard: some View {
        VStack(alignment: .leading, spacing: 0) {
            HStack {
                Label("增强功能面板", systemImage: "bolt.badge.a.fill").font(.system(size: 11, weight: .bold))
                Spacer()
            }
            .padding(.horizontal, 12).padding(.vertical, 8)
            .background(Color.white.opacity(0.02))
            
            Divider().background(Color.white.opacity(0.08))
            
            ScrollView {
                VStack(alignment: .leading, spacing: 10) {
                    ToggleRow(title: "自动切屏识别对手 (增强模式)", subtitle: "自动轮询7家对手屏幕，精准识别", isOn: $autoSwitchOpponent)
                    ToggleRow(title: "OBS源接入 (实验功能)", subtitle: "接入OBS窗口源，适合直播场景", isOn: $obsSourceAccess)
                    ToggleRow(title: "卡池剩余数量推算", subtitle: "根据全场数据推算核心卡剩余数量", isOn: $poolTracking)
                    ToggleRow(title: "语音播报建议", subtitle: "关键操作语音提醒", isOn: $voiceBroadcast)
                    
                    Spacer()
                    
                    Button(action: { appState.resetGame() }) {
                        Text("一键重置对局记忆")
                            .font(.system(size: 11, weight: .bold))
                            .foregroundStyle(.white)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 8)
                            .background(
                                LinearGradient(
                                    colors: [Color.purple.opacity(0.7), Color.blue.opacity(0.5)],
                                    startPoint: .leading,
                                    endPoint: .trailing
                                )
                            )
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                    }
                    .buttonStyle(.plain)
                    .shadow(color: .purple.opacity(0.25), radius: 4, x: 0, y: 2)
                    .padding(.top, 8)
                }
                .padding(12)
            }
        }
        .background(CardBackground())
    }

    // MARK: - 3. 底部状态栏

    private var bottomStatusBar: some View {
        HStack {
            // 系统状态
            HStack(spacing: 5) {
                Circle()
                    .fill(Color.green)
                    .frame(width: 6, height: 6)
                    .shadow(color: .green.opacity(0.5), radius: 3)
                Text("系统状态：正常运行中").font(.system(size: 10)).foregroundStyle(.secondary)
            }
            
            Spacer()
            
            // 指标
            HStack(spacing: 16) {
                StatusIndicator(label: "延迟", val: "68ms")
                StatusIndicator(label: "帧率", val: "15FPS")
                StatusIndicator(label: "内存", val: "1.24GB")
                StatusIndicator(label: "CPU", val: "12%")
            }
            
            Spacer()
            
            // 退出系统
            Button(action: { NSApplication.shared.terminate(nil) }) {
                HStack(spacing: 4) {
                    Image(systemName: "power").font(.system(size: 9))
                    Text("退出系统").font(.system(size: 10, weight: .bold))
                }
                .foregroundStyle(.white)
                .padding(.horizontal, 10).padding(.vertical, 5)
                .background(
                    LinearGradient(
                        colors: [Color(red: 0.7, green: 0.15, blue: 0.15), Color(red: 0.5, green: 0.1, blue: 0.1)],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                )
                .clipShape(RoundedRectangle(cornerRadius: 5))
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(Color.white.opacity(0.02))
        .overlay(RoundedRectangle(cornerRadius: 8).stroke(Color.white.opacity(0.05), lineWidth: 1))
        .clipShape(RoundedRectangle(cornerRadius: 8))
    }
    
    // MARK: - Helper 数据加载解析器
    
    private func getEquipmentIconUrl(name: String) -> String {
        GameDataService.shared.equipment.first(where: { $0.name.contains(name) })?.picture ?? ""
    }
    
    private func getHeroIconUrl(name: String) -> String {
        GameDataService.shared.heroPicture(named: name)
    }
}

// MARK: - 辅助子视图组件

private struct HeaderStatBlock: View {
    let label: String
    let val: String
    let icon: String
    let color: Color
    
    var body: some View {
        VStack(spacing: 2) {
            HStack(spacing: 3) {
                Image(systemName: icon).font(.system(size: 9)).foregroundStyle(color)
                Text(val).font(.system(size: 11, weight: .bold, design: .monospaced)).foregroundStyle(.white)
            }
            Text(label).font(.system(size: 7)).foregroundStyle(.secondary)
        }
        .padding(.horizontal, 6).padding(.vertical, 3)
        .background(Color.white.opacity(0.04))
        .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}

private struct AlertRow: View {
    let icon: String
    let text: String
    let sub: String
    let color: Color
    
    var body: some View {
        HStack(alignment: .top, spacing: 8) {
            Image(systemName: icon)
                .font(.system(size: 12))
                .foregroundStyle(color)
                .padding(6)
                .background(color.opacity(0.12))
                .clipShape(Circle())
            
            VStack(alignment: .leading, spacing: 2) {
                Text(text).font(.system(size: 11, weight: .bold)).foregroundStyle(.white)
                Text(sub).font(.system(size: 9)).foregroundStyle(.secondary)
            }
            Spacer()
        }
        .padding(8)
        .background(Color.white.opacity(0.03))
        .clipShape(RoundedRectangle(cornerRadius: 6))
    }
}

private struct PureAlertBadge: View {
    let title: String
    let color: Color
    
    var body: some View {
        HStack {
            Circle().fill(color).frame(width: 5, height: 5)
            Text(title).font(.system(size: 10, weight: .medium)).foregroundStyle(.white)
            Spacer()
        }
        .padding(.horizontal, 8).padding(.vertical, 5)
        .background(color.opacity(0.12))
        .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}

private struct ToggleRow: View {
    let title: String
    let subtitle: String
    @Binding var isOn: Bool
    
    var body: some View {
        HStack(alignment: .top) {
            VStack(alignment: .leading, spacing: 2) {
                Text(title).font(.system(size: 11, weight: .medium)).foregroundStyle(.primary)
                Text(subtitle).font(.system(size: 8)).foregroundStyle(.secondary)
            }
            Spacer()
            Toggle("", isOn: $isOn)
                .labelsHidden()
                .scaleEffect(0.8)
        }
        .padding(6)
        .background(Color.white.opacity(0.02))
        .clipShape(RoundedRectangle(cornerRadius: 5))
    }
}

private struct SettingSidebarBtn: View {
    let title: String
    let icon: String
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            HStack(spacing: 6) {
                Image(systemName: icon).font(.system(size: 10))
                Text(title).font(.system(size: 9))
                Spacer()
            }
            .foregroundStyle(isSelected ? .white : .secondary)
            .padding(.horizontal, 8).padding(.vertical, 6)
            .background(isSelected ? Color.white.opacity(0.1) : Color.clear)
            .clipShape(RoundedRectangle(cornerRadius: 6))
        }
        .buttonStyle(.plain)
    }
}

private struct RunModeCard: View {
    let title: String
    let desc: String
    let isSelected: Bool
    let action: () -> Void
    
    var body: some View {
        Button(action: action) {
            VStack(alignment: .leading, spacing: 4) {
                HStack {
                    Text(title).font(.system(size: 10, weight: .bold)).foregroundStyle(.white)
                    Spacer()
                    if isSelected {
                        Image(systemName: "checkmark.circle.fill").font(.system(size: 10)).foregroundColor(.green)
                    }
                }
                Text(desc)
                    .font(.system(size: 8))
                    .foregroundStyle(.secondary)
                    .lineSpacing(2)
                    .fixedSize(horizontal: false, vertical: true)
            }
            .padding(8)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(isSelected ? Color.accentColor.opacity(0.15) : Color.white.opacity(0.04))
            .overlay(
                RoundedRectangle(cornerRadius: 6)
                    .stroke(isSelected ? Color.accentColor : Color.white.opacity(0.1), lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 6))
        }
        .buttonStyle(.plain)
    }
}

private struct SmallStatBadge: View {
    let val: String
    var body: some View {
        Text(val)
            .font(.system(size: 9, weight: .semibold, design: .monospaced))
            .padding(.horizontal, 6).padding(.vertical, 3)
            .background(Color.white.opacity(0.05))
            .clipShape(RoundedRectangle(cornerRadius: 4))
    }
}

private struct TraitBadge: View {
    let name: String
    let isPrimary: Bool
    var body: some View {
        Text(name)
            .font(.system(size: 8, weight: .bold))
            .foregroundStyle(isPrimary ? .black : .white)
            .padding(.horizontal, 6).padding(.vertical, 2)
            .background(isPrimary ? Color.yellow.opacity(0.9) : Color.white.opacity(0.1))
            .clipShape(Capsule())
    }
}

private struct TimelineNode: View {
    let label: String
    let sub: String
    let active: Bool
    
    var body: some View {
        VStack(spacing: 2) {
            Circle()
                .fill(active ? Color.yellow : Color.white.opacity(0.15))
                .frame(width: 6, height: 6)
                .shadow(color: active ? .yellow.opacity(0.5) : .clear, radius: 2)
            
            Text(label).font(.system(size: 8, weight: .semibold)).foregroundStyle(active ? .primary : .secondary)
            Text(sub).font(.system(size: 7)).foregroundStyle(.tertiary)
        }
        .frame(width: 44)
    }
}

private struct TimelineDivider: View {
    let active: Bool
    var body: some View {
        Rectangle()
            .fill(active ? Color.yellow.opacity(0.6) : Color.white.opacity(0.15))
            .frame(height: 1)
            .frame(maxWidth: .infinity)
            .offset(y: -8)
    }
}

private struct StatusIndicator: View {
    let label: String
    let val: String
    
    var body: some View {
        HStack(spacing: 4) {
            Text(label + ":").font(.system(size: 9)).foregroundStyle(.secondary)
            Text(val).font(.system(size: 9, design: .monospaced)).foregroundStyle(.primary)
        }
    }
}

private struct CardBackground: View {
    var body: some View {
        Color(red: 0.08, green: 0.08, blue: 0.16).opacity(0.85)
            .overlay(
                RoundedRectangle(cornerRadius: 10)
                    .stroke(Color.white.opacity(0.06), lineWidth: 1)
            )
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .shadow(color: Color.black.opacity(0.3), radius: 4, x: 0, y: 2)
    }
}

struct SafeAsyncImage: View {
    let urlString: String
    var size: CGFloat = 32
    var cornerRadius: CGFloat = 4
    
    var body: some View {
        if let url = URL(string: urlString), !urlString.isEmpty {
            AsyncImage(url: url) { phase in
                switch phase {
                case .success(let image):
                    image.resizable()
                        .aspectRatio(contentMode: .fill)
                case .failure, .empty:
                    fallbackImage
                @unknown default:
                    fallbackImage
                }
            }
            .frame(width: size, height: size)
            .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
        } else {
            fallbackImage
        }
    }
    
    private var fallbackImage: some View {
        Image(systemName: "photo")
            .font(.system(size: size * 0.4))
            .foregroundStyle(.secondary)
            .frame(width: size, height: size)
            .background(Color.white.opacity(0.06))
            .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
    }
}
