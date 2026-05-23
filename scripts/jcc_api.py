"""
金铲铲之战 官方数据拉取工具

数据源（完整版）：
  1. 阵容推荐:   POST mlol.qt.qq.com/go/jgame/get_lineup_recomm
  2. 阵容详情:   POST mlol.qt.qq.com/go/jgame/get_lineup_detail
  3. 游戏数据库: CDN game.gtimg.cn/images/lol/act/jkzlk/js/
     - chess.js    英雄/棋子数据（含头像URL、属性、费用）
     - equip.js    装备数据（含图标URL、合成、类型）
     - trait.js    羁绊数据（含图标URL、等级、人数要求）
     - hex.js      强化符文（含图标URL、效果描述）
     - race.js     种族
     - job.js      职业
     - god.js      星神赐福
     - galaxy.js   星系/国度
     - mission.js  任务
     - goop.js     异常突变
     - tinyhero.js 小小英雄皮肤

使用方式：
  python scripts/jcc_api.py fetch-lineups              # 拉取阵容
  python scripts/jcc_api.py fetch-gamedata             # 拉取游戏数据库
  python scripts/jcc_api.py fetch-all                  # 拉取全部
"""

from __future__ import annotations

import asyncio
import json
import os
import re
from dataclasses import dataclass, field
from datetime import datetime
from pathlib import Path
from typing import Any

from anansi import HTTPFetcher

# CDN 基础路径
CDN_GAME_DATA = "https://game.gtimg.cn/images/lol/act/jkzlk/js"

# API 端点
API_LINEUP_RECOMM = "https://mlol.qt.qq.com/go/jgame/get_lineup_recomm"
API_LINEUP_DETAIL = "https://mlol.qt.qq.com/go/jgame/get_lineup_detail"
BASIC_CONFIG_URL = "https://jcc.qq.com/data-js/basicConfig.js"

# URL key → 文件名 映射
DATA_TYPES: dict[str, str] = {
    "herourl": "chess",
    "equipurl": "equip",
    "traiturl": "trait",
    "hexurl": "hex",
    "raceurl": "race",
    "joburl": "job",
    "godurl": "god",
    "galaxyurl": "galaxy",
    "missionurl": "mission",
    "goopurl": "goop",
    "legendurl": "legend",
    "adventureurl": "adventure",
    "tinyherourl": "tinyhero",
}


@dataclass
class GameDataFetcher:
    """游戏数据库拉取器"""

    season: str = "S18"
    output_dir: str = "config/game_data"
    _fetcher: HTTPFetcher | None = field(default=None, repr=False)
    lineup_channel: str = "11"

    async def __aenter__(self) -> "GameDataFetcher":
        self._fetcher = HTTPFetcher()
        await self._fetcher.__aenter__()
        return self

    async def __aexit__(self, *args: Any) -> None:
        if self._fetcher:
            await self._fetcher.close()

    async def _fetch_json(self, url: str) -> dict:
        assert self._fetcher is not None
        r = await self._fetcher.fetch(url)
        if r.status != 200:
            raise RuntimeError(f"请求失败: {url}, status={r.status}")
        return json.loads(r.html)

    async def get_version_config(self) -> list[dict]:
        """获取版本配置（含所有赛季模式的 URL 映射）"""
        return await self._fetch_json(f"{CDN_GAME_DATA}/config/versiondataconfig.js")

    async def get_basic_config_text(self) -> str:
        assert self._fetcher is not None
        r = await self._fetcher.fetch(BASIC_CONFIG_URL)
        if r.status != 200:
            raise RuntimeError(f"请求失败: {BASIC_CONFIG_URL}, status={r.status}")
        return r.html

    async def fetch_all_gamedata(self, modes: list[str] | None = None) -> dict[str, dict]:
        """拉取指定模式的所有游戏数据"""
        config = await self.get_version_config()

        # 筛选最新版本
        latest: dict[str, dict] = {}
        for m in config:
            mode = m.get("mode", "")
            season = m.get("season", "")
            ver = m.get("version", "")
            if season != self.season:
                continue
            if modes and mode not in modes:
                continue
            key = mode
            if key not in latest or ver > latest[key].get("version", ""):
                latest[key] = m

        results: dict[str, dict] = {}
        for mode, m in sorted(latest.items()):
            name = m.get("name", mode)
            print(f"\n{'='*50}")
            print(f"模式 {mode} ({name}) — version {m.get('version')}")
            results[mode] = {"config": m, "data": {}}

            for url_key, file_type in DATA_TYPES.items():
                path = m.get(url_key, "")
                if not path:
                    continue
                url = f"{CDN_GAME_DATA}//{path}"
                try:
                    data = await self._fetch_json(url)
                    out_dir = Path(self.output_dir) / f"mode{mode}"
                    out_dir.mkdir(parents=True, exist_ok=True)
                    out_path = out_dir / f"{file_type}.json"
                    out_path.write_text(json.dumps(data, ensure_ascii=False, indent=2),
                                       encoding="utf-8")

                    count = len(data.get("data", {}))
                    size_kb = out_path.stat().st_size // 1024
                    print(f"  {file_type}: {count} 条 ({size_kb}KB)")
                    results[mode]["data"][file_type] = data
                except Exception as e:
                    print(f"  {file_type}: 跳过 ({e})")

        return results

    async def fetch_lineups(self, modes: list[str] | None = None) -> dict[str, dict]:
        """拉取官网阵容总表 JSON"""
        config_text = await self.get_basic_config_text()
        requested_modes = modes or ["17", "16", "4"]

        id_to_url: dict[str, str] = {}
        for mode in requested_modes:
            key = f"{mode}_{self.season}"
            pattern = rf"'{re.escape(key)}':`([^`]+)`"
            match = re.search(pattern, config_text)
            if not match:
                print(f"未找到阵容地址: {key}")
                continue
            id_to_url[mode] = match.group(1).replace("${channel}", self.lineup_channel)

        results: dict[str, dict] = {}
        out_dir = Path("config/lineups")
        out_dir.mkdir(parents=True, exist_ok=True)

        for mode, url in id_to_url.items():
            data = await self._fetch_json(url)
            lineup_list = data.get("lineup_list", [])
            out_path = out_dir / f"mode{mode}_{self.season}.json"
            out_path.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
            print(f"mode {mode}: 保存 {len(lineup_list)} 个阵容 -> {out_path.name}")
            results[mode] = data

        return results


async def cmd_fetch_gamedata():
    """命令行：拉取游戏数据库"""
    async with GameDataFetcher() as fetcher:
        await fetcher.fetch_all_gamedata()


async def cmd_fetch_lineups():
    """命令行：拉取阵容"""
    async with GameDataFetcher() as fetcher:
        await fetcher.fetch_lineups()


async def cmd_fetch_all():
    """命令行：拉取全部"""
    async with GameDataFetcher() as fetcher:
        await fetcher.fetch_all_gamedata()
        await fetcher.fetch_lineups()


if __name__ == "__main__":
    import sys
    cmd = sys.argv[1] if len(sys.argv) > 1 else "fetch-all"
    if cmd == "fetch-gamedata":
        asyncio.run(cmd_fetch_gamedata())
    elif cmd == "fetch-lineups":
        asyncio.run(cmd_fetch_lineups())
    else:
        asyncio.run(cmd_fetch_all())
