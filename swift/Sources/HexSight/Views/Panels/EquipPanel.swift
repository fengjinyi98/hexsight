//
//  EquipPanel.swift
//  HexSight
//
//  Created by Antigravity on 2026-05-24.
//  职责：装备图鉴面板，复刻云顶官网大公式平铺排版布局，彻底废除左右对半均分的多窗限制，支持大类横向按钮、基础件高亮横条多维过滤和成品平铺。
//

import SwiftUI

struct EquipPanel: View {
    @StateObject private var data = GameDataService.shared
    @State private var selectedCategory: EquipCategory = .component
    @State private var selectedComponentId: String? = nil
    @State private var searchQuery: String = ""

    /// 装备大分类定义
    enum EquipCategory: String, CaseIterable {
        case component = "基础装备"
        case completed = "成型装备"
        case radiant = "光明装备"
        case support = "辅助装备"
        case artifact = "神器装备"
        case emblem = "纹章"
        case special = "特殊装备"
    }

    /// 提取当前模式下的核心基础装备（用来做横向小图标选择器）
    private var componentItems: [EquipmentModel] {
        data.equipment.filter { $0.type.contains("基础") }
            .sorted { $0.id < $1.id }
    }

    /// 根据分类、选中的基础小件筛选出在列表中平铺展示的装备
    private var displayEquipment: [EquipmentModel] {
        let all = data.equipment
        let initialList: [EquipmentModel]
        
        switch selectedCategory {
        case .component:
            // 基础分类下：实际展示由该基础件所能合成的所有成品公式列表
            if let compId = selectedComponentId {
                initialList = all.filter { $0.type.contains("成型") && ($0.synthesis1 == compId || $0.synthesis2 == compId) }
            } else {
                initialList = all.filter { $0.type.contains("成型") }
            }
            
        case .completed:
            // 成型分类下：展示成型装备。如果点击了具体的小件，则对其进行配方筛选；否则全部展示。
            let completedList = all.filter { $0.type.contains("成型") }
            if let compId = selectedComponentId {
                initialList = completedList.filter { $0.synthesis1 == compId || $0.synthesis2 == compId }
            } else {
                initialList = completedList
            }
            
        default:
            // 其他没有合成公式的分类，直接展示其所有对应成员
            initialList = all.filter { belongsTo(eq: $0, category: selectedCategory) }
        }
        
        // 模糊搜索过滤
        if !searchQuery.isEmpty {
            return initialList.filter { eq in
                let nameMatch = eq.name.localizedCaseInsensitiveContains(searchQuery)
                let descMatch = eq.basicDesc.localizedCaseInsensitiveContains(searchQuery) || eq.desc.localizedCaseInsensitiveContains(searchQuery)
                return nameMatch || descMatch
            }
        }
        return initialList
    }

    var body: some View {
        VStack(spacing: 0) {
            // 1. 顶部 Header 控制栏 (版本药丸 + 大类选项卡)
            topHeaderView
            
            Divider().background(Color.white.opacity(0.1))

            // 2. 基础件小图标选择横条 (仅基础件和成型件大类显示)
            if selectedCategory == .component || selectedCategory == .completed {
                componentSelectorView
            }

            // 3. 图鉴列表区域 (大公式行或普通行平铺展示)
            ZStack {
                if displayEquipment.isEmpty {
                    emptyView
                } else {
                    ScrollView {
                        LazyVStack(spacing: 8) {
                            ForEach(displayEquipment.sorted(by: { $0.name < $1.name })) { eq in
                                if selectedCategory == .component || selectedCategory == .completed {
                                    formulaRow(for: eq)
                                } else {
                                    normalRow(for: eq)
                                }
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
        .onAppear {
            initializeComponentFilter()
        }
        .onChange(of: data.selectedMode) { _, _ in
            initializeComponentFilter()
        }
        .onChange(of: selectedCategory) { _, newCat in
            if newCat == .component && selectedComponentId == nil {
                selectedComponentId = componentItems.first?.id
            }
        }
    }

    private func initializeComponentFilter() {
        // 模式切换或初次进入时，默认高亮第一个基础件
        if selectedCategory == .component || selectedComponentId == nil {
            selectedComponentId = componentItems.first?.id
        }
    }

    // MARK: - 顶部 Header 视图

    private var topHeaderView: some View {
        HStack(alignment: .center, spacing: Theme.Spacing.medium) {
            // 左侧：版本药丸组
            HStack(spacing: 8) {
                ForEach(data.availableModes, id: \.id) { mode in
                    Button {
                        data.switchMode(mode.id)
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

            // 右侧：大类水平选项卡组 (高亮亮黄背景配黑字) + 模糊搜索框
            HStack(spacing: 8) {
                HStack(spacing: 6) {
                    ForEach(EquipCategory.allCases, id: \.self) { cat in
                        Button {
                            selectedCategory = cat
                            if cat == .component && selectedComponentId == nil {
                                selectedComponentId = componentItems.first?.id
                            }
                        } label: {
                            Text(cat.rawValue)
                                .font(.system(size: 11, weight: selectedCategory == cat ? .bold : .medium))
                                .foregroundStyle(selectedCategory == cat ? Color(red: 0.05, green: 0.05, blue: 0.1) : Theme.Color.textSecondary)
                                .padding(.horizontal, 12)
                                .padding(.vertical, 6)
                                .background(
                                    selectedCategory == cat
                                    ? Theme.Color.gold
                                    : Color.white.opacity(0.04)
                                )
                                .clipShape(RoundedRectangle(cornerRadius: 4))
                                .overlay(
                                    RoundedRectangle(cornerRadius: 4)
                                        .stroke(selectedCategory == cat ? Theme.Color.gold : Color.white.opacity(0.08), lineWidth: 1)
                                )
                        }
                        .buttonStyle(.plain)
                    }
                }
                
                // 搜索输入框
                HStack(spacing: 6) {
                    Image(systemName: "magnifyingglass")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundStyle(Theme.Color.textTertiary)
                    
                    TextField("搜索装备", text: $searchQuery)
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

    // MARK: - 基础件小图标选择横条

    private var componentSelectorView: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                // 如果是成型装备，提供“全部”选项
                if selectedCategory == .completed {
                    Button {
                        selectedComponentId = nil
                    } label: {
                        Text("全部")
                            .font(Theme.Font.micro.bold())
                            .foregroundStyle(selectedComponentId == nil ? Color(red: 0.05, green: 0.05, blue: 0.1) : Theme.Color.textSecondary)
                            .frame(width: 32, height: 32)
                            .background(selectedComponentId == nil ? Theme.Color.gold : Color.white.opacity(0.04))
                            .clipShape(RoundedRectangle(cornerRadius: 3))
                            .overlay(
                                RoundedRectangle(cornerRadius: 3)
                                    .stroke(selectedComponentId == nil ? Theme.Color.gold : Color.white.opacity(0.08), lineWidth: 1)
                            )
                    }
                    .buttonStyle(.plain)
                }
                
                // 9 核心基础件图标
                ForEach(componentItems) { eq in
                    Button {
                        selectedComponentId = eq.id
                    } label: {
                        AsyncImage(url: URL(string: eq.picture)) { phase in
                            switch phase {
                            case .success(let image):
                                image.resizable()
                                    .aspectRatio(contentMode: .fit)
                                    .frame(width: 32, height: 32)
                                    .clipped()
                            case .failure, .empty:
                                Color.white.opacity(0.06)
                                    .overlay(Image(systemName: "questionmark").font(.system(size: 10)))
                            @unknown default:
                                Color.clear
                            }
                        }
                        .frame(width: 32, height: 32)
                        .clipShape(RoundedRectangle(cornerRadius: 3))
                        .overlay(
                            RoundedRectangle(cornerRadius: 3)
                                .stroke(selectedComponentId == eq.id ? Theme.Color.gold : Color.white.opacity(0.08), lineWidth: 1.5)
                        )
                        .shadow(color: selectedComponentId == eq.id ? Theme.Color.gold.opacity(0.4) : Color.clear, radius: 3)
                    }
                    .buttonStyle(.plain)
                    .help(eq.name)
                }
                
                // 当前选中基础件属性描述挂载
                if let compId = selectedComponentId, let compEq = data.getEquip(compId) {
                    HStack(spacing: 6) {
                        Text(compEq.name)
                            .font(Theme.Font.caption.bold())
                            .foregroundStyle(Theme.Color.gold)
                        
                        Text(compEq.basicDesc)
                            .font(Theme.Font.micro)
                            .foregroundStyle(Theme.Color.textSecondary)
                    }
                    .padding(.horizontal, 10)
                    .padding(.vertical, 5)
                    .background(Color.white.opacity(0.04))
                    .clipShape(RoundedRectangle(cornerRadius: 4))
                    .overlay(RoundedRectangle(cornerRadius: 4).stroke(Color.white.opacity(0.06), lineWidth: 0.8))
                    .padding(.leading, 12)
                }
            }
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 8)
        .background(Color.white.opacity(0.02))
        .clipShape(RoundedRectangle(cornerRadius: 6))
        .padding(.horizontal, 14)
        .padding(.top, 8)
    }

    // MARK: - 配方大公式行组件 (Recipe Formula Row)

    private func formulaRow(for eq: EquipmentModel) -> some View {
        HStack(spacing: 16) {
            // 1. 配方区 (A + B =)
            HStack(spacing: 8) {
                let compA = data.getEquip(eq.synthesis1)
                let compB = data.getEquip(eq.synthesis2)
                
                recipeIcon(compA, id: eq.synthesis1)
                
                Text("+")
                    .font(.system(size: 14, weight: .bold))
                    .foregroundStyle(Theme.Color.textSecondary)
                
                recipeIcon(compB, id: eq.synthesis2)
                
                Text("=")
                    .font(.system(size: 14, weight: .bold))
                    .foregroundStyle(Theme.Color.textSecondary)
            }
            .frame(width: 120, alignment: .leading)
            
            // 2. 成品图标
            AsyncImage(url: URL(string: eq.picture)) { phase in
                switch phase {
                case .success(let image):
                    image.resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(width: 44, height: 44)
                        .clipped()
                case .failure, .empty:
                    Color.white.opacity(0.05)
                        .overlay(Image(systemName: "photo").font(.system(size: 14)))
                @unknown default:
                    Color.clear
                }
            }
            .frame(width: 44, height: 44)
            .clipShape(RoundedRectangle(cornerRadius: 4))
            .overlay(RoundedRectangle(cornerRadius: 4).stroke(Theme.Color.gold.opacity(0.4), lineWidth: 1))
            
            // 3. 成品描述
            VStack(alignment: .leading, spacing: 4) {
                Text(eq.name)
                    .font(Theme.Font.body.bold())
                    .foregroundStyle(Theme.Color.textPrimary)
                
                let fullDesc = [eq.basicDesc, eq.desc].filter { !$0.isEmpty }.joined(separator: "  ")
                Text(fullDesc)
                    .font(Theme.Font.caption)
                    .foregroundStyle(Theme.Color.textSecondary)
                    .lineSpacing(3)
                    .lineLimit(2)
                    .fixedSize(horizontal: false, vertical: true)
            }
            
            Spacer()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(Color(red: 0.18, green: 0.12, blue: 0.36).opacity(0.4))
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.white.opacity(0.06), lineWidth: 1)
        )
    }

    private func recipeIcon(_ eq: EquipmentModel?, id: String) -> some View {
        AsyncImage(url: URL(string: eq?.picture ?? "")) { phase in
            switch phase {
            case .success(let image):
                image.resizable()
                    .aspectRatio(contentMode: .fit)
                    .frame(width: 32, height: 32)
                    .clipped()
            case .failure, .empty:
                Color.white.opacity(0.05)
                    .overlay(Image(systemName: "questionmark").font(.system(size: 12)))
            @unknown default:
                Color.clear
            }
        }
        .frame(width: 32, height: 32)
        .clipShape(RoundedRectangle(cornerRadius: 3))
        .overlay(RoundedRectangle(cornerRadius: 3).stroke(Color.white.opacity(0.1), lineWidth: 0.8))
        .help(eq?.name ?? id)
    }

    // MARK: - 无合成公式大类的展示行 (Normal Row)

    private func normalRow(for eq: EquipmentModel) -> some View {
        HStack(spacing: 16) {
            // 1. 成品图标
            AsyncImage(url: URL(string: eq.picture)) { phase in
                switch phase {
                case .success(let image):
                    image.resizable()
                        .aspectRatio(contentMode: .fit)
                        .frame(width: 44, height: 44)
                        .clipped()
                case .failure, .empty:
                    Color.white.opacity(0.05)
                        .overlay(Image(systemName: "photo").font(.system(size: 14)))
                @unknown default:
                    Color.clear
                }
            }
            .frame(width: 44, height: 44)
            .clipShape(RoundedRectangle(cornerRadius: 4))
            .overlay(RoundedRectangle(cornerRadius: 4).stroke(Theme.Color.gold.opacity(0.4), lineWidth: 1))
            
            // 2. 装备描述
            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 8) {
                    Text(eq.name)
                        .font(Theme.Font.body.bold())
                        .foregroundStyle(Theme.Color.textPrimary)
                    
                    Text(eq.type)
                        .font(Theme.Font.micro.bold())
                        .foregroundStyle(Theme.Color.gold)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Theme.Color.gold.opacity(0.12))
                        .clipShape(Capsule())
                }
                
                let fullDesc = [eq.basicDesc, eq.desc].filter { !$0.isEmpty }.joined(separator: "  ")
                Text(fullDesc)
                    .font(Theme.Font.caption)
                    .foregroundStyle(Theme.Color.textSecondary)
                    .lineSpacing(3)
                    .lineLimit(2)
                    .fixedSize(horizontal: false, vertical: true)
            }
            
            Spacer()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 10)
        .background(Color(red: 0.18, green: 0.12, blue: 0.36).opacity(0.4))
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.white.opacity(0.06), lineWidth: 1)
        )
    }

    // MARK: - 数据分类工具方法

    private func belongsTo(eq: EquipmentModel, category: EquipCategory) -> Bool {
        let type = eq.type
        switch category {
        case .component:
            return type.contains("基础")
        case .completed:
            return type.contains("成型")
        case .radiant:
            return type.contains("光明")
        case .support:
            return type.contains("辅助")
        case .artifact:
            return type.contains("奥恩") || type.contains("神器")
        case .emblem:
            return type.contains("纹章") || type.contains("转职")
        case .special:
            // 排除其他已知类型以防漏掉
            let isKnown = type.contains("基础") || type.contains("成型") || type.contains("光明") || type.contains("辅助") || type.contains("奥恩") || type.contains("神器") || type.contains("纹章") || type.contains("转职")
            return !isKnown
        }
    }

    private var emptyView: some View {
        VStack(spacing: 8) {
            Image(systemName: "shield.fill").font(.system(size: 32)).foregroundStyle(.tertiary)
            Text("没有找到符合条件的装备").font(Theme.Font.body).foregroundStyle(.tertiary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
