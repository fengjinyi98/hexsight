#!/usr/bin/env python3
"""
download_hero_templates 用途说明
核心职责：
- 从本地 chess.json 读取英雄名称与头像 URL
- 下载去重后的英雄头像模板到视觉识别模板目录
"""

from __future__ import annotations

import argparse
import json
import re
import sys
import urllib.request
from pathlib import Path

def safe_filename(name: str) -> str:
    """
    safe_filename 用途说明
    核心职责：
    - 将英雄名称转换为可落盘文件名
    - 保留中文英雄名，移除路径分隔符等危险字符
    """
    cleaned = re.sub(r'[\\/:*?"<>|]+', "_", name.strip())
    return cleaned or "unknown"


def load_hero_pictures(chess_json: Path) -> dict[str, str]:
    """
    load_hero_pictures 用途说明
    核心职责：
    - 读取静态英雄数据
    - 按英雄名折叠多星级与重复变体
    """
    payload = json.loads(chess_json.read_text(encoding="utf-8"))
    heroes: dict[str, str] = {}
    for item in payload.get("data", {}).values():
        name = str(item.get("name") or "").strip()
        picture = str(item.get("picture") or "").strip()
        price = str(item.get("price") or "").strip()
        hero_type = str(item.get("heroType") or "").strip()
        if (
            not name
            or not picture
            or price == "0"
            or hero_type != "0"
            or name == "木桩假人"
        ):
            continue
        heroes.setdefault(name, picture)
    return dict(sorted(heroes.items()))


def download(url: str, output: Path, timeout: float) -> None:
    """
    download 用途说明
    核心职责：
    - 下载单个远程头像资源
    - 原子写入目标文件，避免半成品污染模板库
    """
    request = urllib.request.Request(url, headers={"User-Agent": "HexSight/1.0"})
    with urllib.request.urlopen(request, timeout=timeout) as response:
        data = response.read()
    tmp = output.with_suffix(output.suffix + ".tmp")
    tmp.write_bytes(data)
    tmp.replace(output)


def main() -> int:
    """
    main 用途说明
    核心职责：
    - 解析命令行参数
    - 批量下载并报告成功/失败数量
    """
    parser = argparse.ArgumentParser()
    parser.add_argument("--chess-json", type=Path, default=Path("config/game_data/mode17/chess.json"))
    parser.add_argument("--output-dir", type=Path, default=Path("config/vision_templates/heroes"))
    parser.add_argument("--timeout", type=float, default=20.0)
    args = parser.parse_args()

    heroes = load_hero_pictures(args.chess_json)
    args.output_dir.mkdir(parents=True, exist_ok=True)

    failures: list[tuple[str, str]] = []
    for name, url in heroes.items():
        output = args.output_dir / f"{safe_filename(name)}.png"
        if output.exists() and output.stat().st_size > 0:
            continue
        try:
            download(url, output, args.timeout)
        except Exception as exc:
            failures.append((name, str(exc)))

    print(f"heroes={len(heroes)} downloaded={len(heroes) - len(failures)} failed={len(failures)}")
    for name, error in failures:
        print(f"[failed] {name}: {error}", file=sys.stderr)
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
