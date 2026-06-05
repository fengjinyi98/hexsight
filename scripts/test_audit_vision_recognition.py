import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import audit_vision_recognition


class VisionRecognitionAuditTests(unittest.TestCase):
    def test_audit_reports_external_sample_gaps(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_base_project(root, main_png_count=30, augment_annotation_count=0)

            report = audit_vision_recognition.audit(root)

            failed = {check["id"] for check in report["checks"] if check["status"] == "failed"}
            self.assertIn("ocr.main_png_count", failed)
            self.assertIn("ocr.augment_annotation_count", failed)
            self.assertFalse(report["passed"])

    def test_audit_passes_when_repository_evidence_meets_thresholds(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self._write_base_project(root, main_png_count=80, augment_annotation_count=15)

            report = audit_vision_recognition.audit(root)

            self.assertTrue(report["passed"])
            self.assertEqual(
                {check["status"] for check in report["checks"]},
                {"passed"},
            )

    def _write_base_project(
        self,
        root: Path,
        main_png_count: int,
        augment_annotation_count: int,
    ) -> None:
        digit_dir = root / "config" / "digit_templates"
        digit_dir.mkdir(parents=True)
        for digit in range(10):
            (digit_dir / f"{digit}.txt").write_text("111\n101\n111\n", encoding="utf-8")

        template_root = root / "config" / "vision_templates"
        for name, count, suffix in [
            ("heroes", 65, ".png"),
            ("equipment", 2, ".txt"),
            ("traits", 1, ".txt"),
        ]:
            directory = template_root / name
            directory.mkdir(parents=True)
            for index in range(count):
                (directory / f"template_{index}{suffix}").write_text("11\n11\n", encoding="utf-8")

        region_config = {
            "board_grid": [{} for _ in range(28)],
            "bench_slots": [{} for _ in range(9)],
            "opponent_board_grid": [{} for _ in range(14)],
            "carousel_slots": [{} for _ in range(8)],
        }
        region_path = root / "config" / "regions.json"
        region_path.parent.mkdir(parents=True, exist_ok=True)
        region_path.write_text(json.dumps(region_config), encoding="utf-8")

        sample_dir = root / "docs" / "ocr_samples"
        sample_dir.mkdir(parents=True)
        annotations = {}
        for index in range(main_png_count):
            filename = f"sample_{index:03}.png"
            (sample_dir / filename).write_bytes(b"png")
            annotations[filename] = {
                "round": "2-1",
                "shop": [],
                "traits": [],
                "opponents": [],
                "augments": ["潘朵拉的装备"] if index < augment_annotation_count else [],
            }
        (sample_dir / "sample_000_opponent_roi.png").write_bytes(b"png")
        (sample_dir / "ocr_annotations.json").write_text(
            json.dumps(annotations),
            encoding="utf-8",
        )


if __name__ == "__main__":
    unittest.main()
