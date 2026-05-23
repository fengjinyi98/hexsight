//
//  TraitPanel.swift
//  HexSight
//
//  Created by Antigravity on 2026-05-24.
//  职责：展示羁绊图鉴，提供版本切换、分类筛选与模糊搜索，点击卡片后弹出全屏磨砂玻璃详情面板。
//

import SwiftUI

struct TraitPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selected: TraitModel?
    @State private var searchQuery: String = ""
    @State private var selectedType: String? = nil // nil = 全部, "race" = 种族, "job" = 职业

    /// 去重后的羁绊图鉴展示列表
    private var displayTraits: [TraitModel] {
        let grouped = Dictionary(grouping: data.traits) { $0.checkId }
        // 按 checkId 折叠，取最小 level 的记录作为基本图鉴
        let unique = grouped.values.compactMap { $0.min { $0.level < $1.level } }
        
        return unique.filter { trait in
            // 类别过滤 (traitType: 0 = 种族, 1 = 职业)
            if let type = selectedType {
                let isRace = trait.traitType == 0
                if type == "race" && !isRace { return false }
                if type == "job" && isRace { return false }
            }
            
            // 搜索框过滤
            if !searchQuery.isEmpty {
                let matchesName = trait.name.localizedCaseInsensitiveContains(searchQuery)
                let matchesDesc = trait.desc.localizedCaseInsensitiveContains(searchQuery) || trait.realDesc.localizedCaseInsensitiveContains(searchQuery)
                if !matchesName && !matchesDesc { return false }
            }
            return true
        }
        .sorted { $0.name < $1.name }
    }

    var body: some View {
        ZStack {
            VStack(spacing: 0) {
                // 顶部控制Header
                topHeaderView
                
                Divider().background(Color.white.opacity(0.1))

                // 宫格磁贴卡片区域
                ZStack {
                    if displayTraits.isEmpty {
                        emptyView
                    } else {
                        ScrollView {
                            LazyVGrid(
                                columns: [GridItem(.adaptive(minimum: 108, maximum: 108), spacing: 8)],
                                spacing: 8
                            ) {
                                ForEach(displayTraits) { trait in
                                    Button {
                                        selected = trait
                                    } label: {
                                        VStack(spacing: 10) {
                                            Spacer(minLength: 0)
                                            
                                            // 羁绊圆形勋章底座与图标
                                            ZStack {
                                                AsyncImage(url: URL(string: trait.picture)) { phase in
                                                    switch phase {
                                                    case .success(let image):
                                                        image.resizable()
                                                            .aspectRatio(contentMode: .fit)
                                                            .frame(width: 32, height: 32)
                                                    case .failure, .empty:
                                                        Image(systemName: "shield.fill")
                                                            .font(.system(size: 18))
                                                            .foregroundStyle(.white.opacity(0.3))
                                                    @unknown default:
                                                        Color.clear
                                                    }
                                                }
                                                .frame(width: 32, height: 32)
                                            }
                                            .frame(width: 44, height: 44)
                                            .background(Color.white.opacity(0.04))
                                            .clipShape(Circle())
                                            .overlay(Circle().stroke(Color.white.opacity(0.1), lineWidth: 1))
                                            
                                            // 羁绊名称
                                            Text(trait.name)
                                                .font(Theme.Font.caption.weight(.bold))
                                                .foregroundStyle(.white)
                                                .lineLimit(1)
                                                .multilineTextAlignment(.center)
                                            
                                            Spacer(minLength: 0)
                                        }
                                    }
                                    .buttonStyle(.plain)
                                    .frame(width: 108, height: 120)
                                    .background(
                                        selected?.checkId == trait.checkId
                                        ? Color(red: 0.28, green: 0.18, blue: 0.52)
                                        : Color(red: 0.18, green: 0.12, blue: 0.36)
                                    )
                                    .clipShape(RoundedRectangle(cornerRadius: 8))
                                    .overlay(
                                        RoundedRectangle(cornerRadius: 8)
                                            .stroke(selected?.checkId == trait.checkId ? Theme.Color.gold.opacity(0.8) : Color.white.opacity(0.06), lineWidth: 1)
                                    )
                                    .shadow(color: Color.black.opacity(0.2), radius: 3, y: 1)
                                }
                            }
                            .padding(.horizontal, 14)
                            .padding(.top, 10)
                            .padding(.bottom, 32)
                        }
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            }

            // 详情浮层
            if let trait = selected {
                traitDetailOverlay(trait)
                    .transition(.opacity.combined(with: .scale(scale: 0.95)))
            }
        }
        .animation(.easeInOut(duration: 0.2), value: selected != nil)
    }

    // MARK: - 顶栏 Header

    private var topHeaderView: some View {
        HStack(alignment: .center, spacing: Theme.Spacing.medium) {
            // 左侧：游戏版本药丸按钮切换组
            HStack(spacing: 8) {
                ForEach(data.availableModes, id: \.id) { mode in
                    Button {
                        data.switchMode(mode.id)
                        selected = nil
                        selectedType = nil
                    } label: {
                        Text(mode.name)
                            .font(Theme.Font.caption.weight(.bold))
                            .foregroundStyle(data.selectedMode == mode.id ? .white : Theme.Color.textSecondary)
                            .padding(.horizontal, 16)
                            .padding(.vertical, 8)
                            .background(
                                ZStack {
                                    if data.selectedMode == mode.id {
                                        if mode.id == "17" {
                                            LinearGradient(colors: [Color.gray.opacity(0.8), Color.black.opacity(0.6)], startPoint: .topLeading, endPoint: .bottomTrailing)
                                        } else if mode.id == "4" {
                                            LinearGradient(colors: [Color.purple.opacity(0.8), Color(red: 0.19, green: 0.12, blue: 0.37)], startPoint: .topLeading, endPoint: .bottomTrailing)
                                        } else {
                                            LinearGradient(colors: [Color(red: 0.2, green: 0.25, blue: 0.35), Color.black.opacity(0.7)], startPoint: .topLeading, endPoint: .bottomTrailing)
                                        }
                                    } else {
                                        Color.white.opacity(0.04)
                                    }
                                }
                            )
                            .clipShape(RoundedRectangle(cornerRadius: 6))
                            .overlay(
                                RoundedRectangle(cornerRadius: 6)
                                    .stroke(data.selectedMode == mode.id ? (mode.id == "4" ? Color.purple : Color.white.opacity(0.4)) : Color.white.opacity(0.08), lineWidth: 1)
                            )
                            .shadow(color: data.selectedMode == mode.id && mode.id == "4" ? Color.purple.opacity(0.4) : Color.clear, radius: 4)
                    }
                    .buttonStyle(.plain)
                }
            }

            Spacer()

            // 右侧：下拉筛选 + 搜索
            HStack(spacing: 8) {
                // 类别下拉筛选
                Menu {
                    Button("全部羁绊") { selectedType = nil }
                    Button("种族") { selectedType = "race" }
                    Button("职业") { selectedType = "job" }
                } label: {
                    dropdownLabel(text: selectedType == nil ? "类别" : (selectedType == "race" ? "种族" : "职业"))
                }
                .menuStyle(.borderlessButton)

                // 搜索框
                HStack(spacing: 6) {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundStyle(Theme.Color.textTertiary)
                    
                    TextField("搜索羁绊", text: $searchQuery)
                        .textFieldStyle(.plain)
                        .font(Theme.Font.caption)
                        .foregroundStyle(Theme.Color.textPrimary)
                        .frame(width: 100)
                }
                .padding(.horizontal, 8)
                .padding(.vertical, 5)
                .background(Color.white.opacity(0.06))
                .clipShape(RoundedRectangle(cornerRadius: 4))
                .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.white.opacity(0.12), lineWidth: 1))
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
    }

    private func dropdownLabel(text: String) -> some View {
        HStack(spacing: 4) {
            Text(text)
                .font(Theme.Font.caption)
                .foregroundStyle(Theme.Color.textPrimary)
            Image(systemName: "chevron.down")
                .font(.system(size: 8, weight: .bold))
                .foregroundStyle(Theme.Color.textSecondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(Color.white.opacity(0.06))
        .clipShape(RoundedRectangle(cornerRadius: 4))
        .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.white.opacity(0.12), lineWidth: 1))
    }

    // MARK: - 详情仪表盘浮层

    private func traitDetailOverlay(_ trait: TraitModel) -> some View {
        let levels = data.traitLevels(for: trait.checkId)
        let isRace = trait.traitType == 0
        let heroes = data.heroesForTrait(traitId: trait.checkId, isRace: isRace)

        return ZStack(alignment: .bottom) {
            HStack(alignment: .top, spacing: 32) {
                // 左侧栏：羁绊名、简介、人数分级表
                VStack(alignment: .leading, spacing: 18) {
                    // 头部基本信息
                    HStack(spacing: 12) {
                        ZStack {
                            AsyncImage(url: URL(string: trait.picture)) { phase in
                                switch phase {
                                case .success(let image):
                                    image.resizable()
                                        .aspectRatio(contentMode: .fit)
                                        .frame(width: 36, height: 36)
                                case .failure, .empty:
                                    Image(systemName: "shield.fill")
                                        .font(.system(size: 20))
                                        .foregroundStyle(.white.opacity(0.3))
                                @unknown default:
                                    Color.clear
                                }
                            }
                            .frame(width: 36, height: 36)
                        }
                        .frame(width: 52, height: 52)
                        .background(Color.white.opacity(0.05))
                        .clipShape(RoundedRectangle(cornerRadius: 8))
                        .overlay(RoundedRectangle(cornerRadius: 8).stroke(Theme.Color.gold.opacity(0.4), lineWidth: 1))

                        VStack(alignment: .leading, spacing: 4) {
                            Text(trait.name)
                                .font(Theme.Font.title2.weight(.black))
                                .foregroundStyle(Theme.Color.textPrimary)
                            
                            Text(isRace ? "种族" : "职业")
                                .font(Theme.Font.micro.weight(.semibold))
                                .foregroundStyle(Theme.Color.gold)
                                .padding(.horizontal, 8)
                                .padding(.vertical, 2)
                                .background(Theme.Color.gold.opacity(0.12))
                                .clipShape(Capsule())
                        }
                        Spacer()
                    }
                    
                    Divider().background(Color.white.opacity(0.08))

                    // 综合描述摘要
                    if let firstLevel = levels.first, !firstLevel.realDesc.isEmpty {
                        Text(firstLevel.realDesc)
                            .font(Theme.Font.body)
                            .foregroundStyle(Theme.Color.textSecondary)
                            .lineSpacing(4)
                            .fixedSize(horizontal: false, vertical: true)
                    } else if !trait.desc.isEmpty {
                        Text(trait.desc)
                            .font(Theme.Font.body)
                            .foregroundStyle(Theme.Color.textSecondary)
                            .lineSpacing(4)
                            .fixedSize(horizontal: false, vertical: true)
                    }

                    // 分级人数效果表
                    if !levels.isEmpty {
                        ScrollView {
                            VStack(spacing: 8) {
                                ForEach(levels) { lv in
                                    HStack(alignment: .top, spacing: 10) {
                                        Text("\(lv.num) 人")
                                            .font(.system(size: 11, weight: .bold, design: .monospaced))
                                            .foregroundStyle(Theme.Color.gold)
                                            .frame(width: 44, alignment: .leading)
                                            .padding(.top, 1)
                                        
                                        Text(lv.realDesc)
                                            .font(Theme.Font.caption)
                                            .foregroundStyle(Theme.Color.textSecondary)
                                            .lineSpacing(3)
                                            .fixedSize(horizontal: false, vertical: true)
                                    }
                                    .padding(10)
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                    .background(Color.white.opacity(0.02))
                                    .clipShape(RoundedRectangle(cornerRadius: 6))
                                    .overlay(RoundedRectangle(cornerRadius: 6).stroke(Color.white.opacity(0.04), lineWidth: 0.8))
                                }
                            }
                        }
                    }
                }
                .frame(width: 460)
                .padding(20)
                .background(Theme.Color.cardBackground)
                .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.08), lineWidth: 1))

                // 右侧栏：协同英雄平铺一览
                VStack(alignment: .leading, spacing: 14) {
                    Text("协同英雄 (\(heroes.count))")
                        .font(Theme.Font.title3)
                        .foregroundStyle(Theme.Color.textPrimary)
                    
                    Divider().background(Color.white.opacity(0.08))

                    if heroes.isEmpty {
                        VStack(spacing: 8) {
                            Spacer()
                            Image(systemName: "person.slash")
                                .font(.system(size: 24))
                                .foregroundStyle(Theme.Color.textTertiary)
                            Text("无协同英雄数据")
                                .font(Theme.Font.caption)
                                .foregroundStyle(Theme.Color.textTertiary)
                            Spacer()
                        }
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                    } else {
                        ScrollView {
                            LazyVGrid(
                                columns: [GridItem(.adaptive(minimum: 68), spacing: 12)],
                                spacing: 14
                            ) {
                                ForEach(heroes) { hero in
                                    VStack(spacing: 6) {
                                        // 费用着色描边圆形头像
                                        AsyncImage(url: URL(string: hero.picture)) { phase in
                                            switch phase {
                                            case .success(let image):
                                                image.resizable()
                                                    .aspectRatio(contentMode: .fill)
                                                    .frame(width: 44, height: 44)
                                                    .clipped()
                                            case .failure, .empty:
                                                Color.white.opacity(0.05)
                                                    .overlay(Image(systemName: "person.fill").font(.system(size: 16)).foregroundStyle(.secondary))
                                            @unknown default:
                                                Color.clear
                                            }
                                        }
                                        .frame(width: 44, height: 44)
                                        .clipShape(Circle())
                                        .overlay(Circle().stroke(costColor(for: hero.cost), lineWidth: 2))
                                        .shadow(color: costColor(for: hero.cost).opacity(0.3), radius: 3)
                                        
                                        // 名字
                                        Text(hero.name)
                                            .font(Theme.Font.micro.weight(.medium))
                                            .foregroundStyle(Theme.Color.textSecondary)
                                            .lineLimit(1)
                                            .multilineTextAlignment(.center)
                                    }
                                    .frame(width: 68)
                                }
                            }
                            .padding(.top, 4)
                        }
                    }
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .padding(20)
                .background(Theme.Color.cardBackground)
                .clipShape(RoundedRectangle(cornerRadius: Theme.CornerRadius.card))
                .overlay(RoundedRectangle(cornerRadius: Theme.CornerRadius.card).stroke(Color.white.opacity(0.08), lineWidth: 1))
            }
            .padding(.horizontal, 24)
            .padding(.top, 24)
            .padding(.bottom, 76)

            // 底部 X 关闭按钮
            Button {
                selected = nil
            } label: {
                Image(systemName: "xmark")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundStyle(Theme.Color.textSecondary)
                    .frame(width: 44, height: 44)
                    .background(Color.white.opacity(0.08))
                    .clipShape(Circle())
                    .overlay(Circle().stroke(Color.white.opacity(0.12), lineWidth: 1))
            }
            .buttonStyle(.plain)
            .padding(.bottom, 16)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .background(Color(red: 0.05, green: 0.05, blue: 0.12).opacity(0.85))
        .background(.ultraThinMaterial)
    }

    /// 费用描边着色
    private func costColor(for cost: Int) -> Color {
        switch cost {
        case 1: Color.gray
        case 2: Color(red: 0.12, green: 0.6, blue: 0.3)
        case 3: Color(red: 0.1, green: 0.45, blue: 0.9)
        case 4: Color.purple
        case 5: Color(red: 0.9, green: 0.65, blue: 0.0)
        default: Color.white
        }
    }

    private var emptyView: some View {
        VStack(spacing: 8) {
            Image(systemName: "link").font(.system(size: 32)).foregroundStyle(.tertiary)
            Text("没有找到符合条件的羁绊").font(Theme.Font.body).foregroundStyle(.tertiary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
