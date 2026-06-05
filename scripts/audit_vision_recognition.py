#!/usr/bin/env python3
import argparse
import json
from pathlib import Path


DEBUG_SUFFIXES = (
    "_bottom",
    "_opponent_roi",
    "_roi_sheet",
    "_trait_roi",
)


def audit(root: Path) -> dict:
    """
    audit 画面识别目标审计入口
    核心职责：
    - 检查 VISION_RECOGNITION_TODO 中可机审的完成证据
    - 将真实样本缺口输出为明确失败项
    """
    checks = [
        check_digit_templates(root),
        check_template_directory(root, "heroes", 60, {".png"}),
        check_template_directory(root, "equipment", 2, {".png", ".jpg", ".jpeg", ".txt"}),
        check_template_directory(root, "traits", 1, {".png", ".jpg", ".jpeg", ".txt"}),
        check_region_slots(root, "board_grid", 28),
        check_region_slots(root, "bench_slots", 9),
        check_region_slots(root, "opponent_board_grid", 14),
        check_region_slots(root, "carousel_slots", 8),
        check_main_sample_count(root, 80),
        check_augment_annotation_count(root, 15),
    ]
    return {
        "passed": all(check["status"] == "passed" for check in checks),
        "checks": checks,
    }


def check_digit_templates(root: Path) -> dict:
    """
    check_digit_templates 数字模板完整性检查
    核心职责：
    - 确认 0-9 模板文件存在
    - 确认模板文件包含可解析内容
    """
    directory = root / "config" / "digit_templates"
    missing = []
    empty = []
    for digit in range(10):
        path = directory / f"{digit}.txt"
        if not path.exists():
            missing.append(path.name)
        elif not path.read_text(encoding="utf-8").strip():
            empty.append(path.name)

    return make_check(
        "templates.digits",
        not missing and not empty,
        f"数字模板目录完整: {directory}",
        f"数字模板缺失或为空: missing={missing}, empty={empty}",
    )


def check_template_directory(root: Path, name: str, minimum: int, extensions: set[str]) -> dict:
    """
    check_template_directory 图标模板目录检查
    核心职责：
    - 统计指定模板目录中的有效文件
    - 对照当前阶段要求输出通过或失败
    """
    directory = root / "config" / "vision_templates" / name
    files = [
        path
        for path in directory.glob("*")
        if path.is_file() and path.suffix.lower() in extensions
    ]
    return make_check(
        f"templates.{name}",
        len(files) >= minimum,
        f"{name} 模板数量 {len(files)}，要求 >= {minimum}",
        f"{name} 模板数量 {len(files)}，要求 >= {minimum}",
        observed=len(files),
        required=minimum,
    )


def check_region_slots(root: Path, key: str, minimum: int) -> dict:
    """
    check_region_slots ROI 配置检查
    核心职责：
    - 读取区域配置中的指定槽位数组
    - 验证棋盘、备战席、对手棋盘和选秀槽位具备入口
    """
    path = root / "config" / "regions.json"
    try:
        regions = json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        regions = {}
    slots = regions.get(key, [])
    count = len(slots) if isinstance(slots, list) else 0
    return make_check(
        f"regions.{key}",
        count >= minimum,
        f"{key} 槽位数量 {count}，要求 >= {minimum}",
        f"{key} 槽位数量 {count}，要求 >= {minimum}",
        observed=count,
        required=minimum,
    )


def check_main_sample_count(root: Path, minimum: int) -> dict:
    """
    check_main_sample_count OCR 主样本数量检查
    核心职责：
    - 排除 ROI 裁剪调试图
    - 验证内测回归集是否达到文档目标下限
    """
    samples = main_png_samples(root)
    return make_check(
        "ocr.main_png_count",
        len(samples) >= minimum,
        f"OCR 主截图 {len(samples)} 张，要求 >= {minimum}",
        f"OCR 主截图 {len(samples)} 张，要求 >= {minimum}",
        observed=len(samples),
        required=minimum,
    )


def check_augment_annotation_count(root: Path, minimum: int) -> dict:
    """
    check_augment_annotation_count 海克斯标注数量检查
    核心职责：
    - 读取批量或同名 JSON 标注
    - 统计包含海克斯候选答案的真实样本数量
    """
    annotations = load_annotations(root / "docs" / "ocr_samples")
    count = sum(1 for value in annotations.values() if value.get("augments"))
    return make_check(
        "ocr.augment_annotation_count",
        count >= minimum,
        f"海克斯标注样本 {count} 张，要求 >= {minimum}",
        f"海克斯标注样本 {count} 张，要求 >= {minimum}",
        observed=count,
        required=minimum,
    )


def main_png_samples(root: Path) -> list[Path]:
    """
    main_png_samples OCR 主截图枚举
    核心职责：
    - 读取 docs/ocr_samples 下的 PNG
    - 过滤调试裁剪产物
    """
    directory = root / "docs" / "ocr_samples"
    if not directory.exists():
        return []
    return sorted(
        path
        for path in directory.glob("*.png")
        if not any(path.stem.endswith(suffix) for suffix in DEBUG_SUFFIXES)
    )


def load_annotations(directory: Path) -> dict[str, dict]:
    """
    load_annotations OCR 标注加载
    核心职责：
    - 支持 ocr_annotations.json 批量标注
    - 支持与截图同名的单文件 JSON 标注
    """
    annotations: dict[str, dict] = {}
    batch = directory / "ocr_annotations.json"
    if batch.exists():
        annotations.update(json.loads(batch.read_text(encoding="utf-8")))

    if not directory.exists():
        return annotations

    for path in sorted(directory.glob("*.json")):
        if path.name in {"ocr_annotations.json", "ocr_eval_output.json"}:
            continue
        if any(path.stem.endswith(suffix) for suffix in DEBUG_SUFFIXES):
            continue
        annotations[f"{path.stem}.png"] = json.loads(path.read_text(encoding="utf-8"))
    return annotations


def make_check(
    check_id: str,
    passed: bool,
    passed_message: str,
    failed_message: str,
    observed: int | None = None,
    required: int | None = None,
) -> dict:
    """
    make_check 审计结果构造
    核心职责：
    - 统一检查项 JSON 字段
    - 保留观测值和要求值用于命令行报告
    """
    check = {
        "id": check_id,
        "status": "passed" if passed else "failed",
        "message": passed_message if passed else failed_message,
    }
    if observed is not None:
        check["observed"] = observed
    if required is not None:
        check["required"] = required
    return check


def main() -> int:
    """
    main 命令行入口
    核心职责：
    - 解析仓库根目录参数
    - 输出机器可读 JSON 并用退出码表达审计结果
    """
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        default=Path.cwd(),
        type=Path,
        help="HexSight 仓库根目录",
    )
    args = parser.parse_args()
    report = audit(args.root)
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
