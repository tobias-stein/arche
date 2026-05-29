"""Tests for bench/run.py HTML report generation."""

import json
import os
import sys
import tempfile
import unittest
from unittest.mock import patch, MagicMock

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from run import _generate_html_report, _run_sweep, SAMPLE_DURATION


SAMPLE_RESULTS = [
    {
        "blueprint_count": 10,
        "affix_count": 0,
        "attribute_count": 3,
        "concurrency": 4,
        "duration_s": "5.00",
        "total_requests": 500,
        "throughput": "100.00",
        "p50_ms": "1.20",
        "p95_ms": "2.80",
        "p99_ms": "4.10",
    },
    {
        "blueprint_count": 100,
        "affix_count": 2,
        "attribute_count": 10,
        "concurrency": 8,
        "duration_s": "5.00",
        "total_requests": 400,
        "throughput": "80.00",
        "p50_ms": "2.10",
        "p95_ms": "5.30",
        "p99_ms": "8.70",
    },
    {
        "blueprint_count": 1000,
        "affix_count": 4,
        "attribute_count": 25,
        "concurrency": 6,
        "duration_s": "5.00",
        "total_requests": 200,
        "throughput": "40.00",
        "p50_ms": "5.50",
        "p95_ms": "12.00",
        "p99_ms": "20.00",
    },
]


class TestGenerateHtmlReport(unittest.TestCase):
    def test_generates_html_file(self):
        with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
            html_path = f.name
        try:
            _generate_html_report(SAMPLE_RESULTS, html_path)
            self.assertTrue(os.path.exists(html_path))
            with open(html_path, "r") as f:
                content = f.read()
            self.assertGreater(len(content), 0)
        finally:
            os.unlink(html_path)

    def test_contains_chartjs_cdn(self):
        with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
            html_path = f.name
        try:
            _generate_html_report(SAMPLE_RESULTS, html_path)
            with open(html_path, "r") as f:
                content = f.read()
            self.assertIn("chart.js", content.lower())
            self.assertIn("cdn.jsdelivr.net", content)
        finally:
            os.unlink(html_path)

    def test_contains_embedded_json_data(self):
        with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
            html_path = f.name
        try:
            _generate_html_report(SAMPLE_RESULTS, html_path)
            with open(html_path, "r") as f:
                content = f.read()
            self.assertIn("BENCH_DATA", content)
            # Verify the JSON is valid
            start = content.index("BENCH_DATA = ")
            end = content.index(";", start)
            json_str = content[start + len("BENCH_DATA = "):end]
            data = json.loads(json_str)
            self.assertEqual(len(data), 3)
        finally:
            os.unlink(html_path)

    def test_contains_all_expected_chart_elements(self):
        with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
            html_path = f.name
        try:
            _generate_html_report(SAMPLE_RESULTS, html_path)
            with open(html_path, "r") as f:
                content = f.read()
            self.assertIn("throughputChart", content)
            self.assertIn("peakChart", content)
            self.assertIn("latencyChart", content)
            self.assertIn("new Chart", content)
        finally:
            os.unlink(html_path)

    def test_contains_filter_controls(self):
        with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
            html_path = f.name
        try:
            _generate_html_report(SAMPLE_RESULTS, html_path)
            with open(html_path, "r") as f:
                content = f.read()
            self.assertIn("checkbox", content.lower())
            self.assertIn("filter", content.lower())
        finally:
            os.unlink(html_path)

    def test_axes_labeled_with_units(self):
        with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
            html_path = f.name
        try:
            _generate_html_report(SAMPLE_RESULTS, html_path)
            with open(html_path, "r") as f:
                content = f.read()
            self.assertIn("gen/s", content.lower())
        finally:
            os.unlink(html_path)

    def test_empty_results_generates_skeleton(self):
        with tempfile.NamedTemporaryFile(suffix=".html", delete=False) as f:
            html_path = f.name
        try:
            _generate_html_report([], html_path)
            with open(html_path, "r") as f:
                content = f.read()
            self.assertIn("BENCH_DATA = []", content)
        finally:
            os.unlink(html_path)


def _make_response(status, body_bytes):
    mock_response = MagicMock()
    mock_response.status = status
    mock_response.read.return_value = body_bytes
    mock_response.__enter__.return_value = mock_response
    return mock_response


CLIENT_RESP = _make_response(200, b'{"id": "c-xxx", "name": "bench-test"}')
KEY_RESP = _make_response(
    200,
    json.dumps({
        "id": "k-xxx", "key": "arche_k_scoped",
        "name": "benchmark-key", "permissions": ["generate"],
    }).encode(),
)
BP_RESP = _make_response(200, b'{"id": "bp-x", "name": "x"}')
GEN_RESP = _make_response(200, b'{"ok": true}')
DELETE_RESP = _make_response(200, b'{"ok": true}')


def _smart_side_effect(*args, **kwargs):
    req = args[0]
    url = req.full_url
    method = req.method

    if "/keys" in url:
        if method == "POST":
            return KEY_RESP
        if method == "DELETE":
            return DELETE_RESP

    if "/api/clients" in url:
        if method == "POST":
            return CLIENT_RESP
        if method == "DELETE":
            return DELETE_RESP

    if "/api/blueprints" in url and method == "POST":
        return BP_RESP

    if "/api/generate" in url:
        return GEN_RESP

    return _make_response(200, b'{"ok": true}')


class TestSweepGeneratesReport(unittest.TestCase):
    def test_sweep_generates_html_report(self):
        """Verify _run_sweep produces an HTML report file."""
        scenarios = [
            {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3},
        ]

        with patch("run.load_config", return_value=scenarios):
            with patch("urllib.request.urlopen", side_effect=_smart_side_effect):
                with patch("builtins.print"):
                    with patch("run.os.makedirs"):
                        with patch("builtins.open") as mock_open:
                            mock_csv_file = MagicMock()
                            mock_html_file = MagicMock()
                            mock_open.side_effect = [mock_csv_file, mock_html_file]

                            _run_sweep(
                                api_key="super-key",
                                target_url="http://localhost:8080",
                                config_path="dummy.yaml",
                                concurrency=1,
                                duration=0.1,
                                sample_duration=0.05,
                            )

        # Verify HTML file was opened for writing
        html_write_calls = [
            call for call in mock_open.call_args_list
            if "report.html" in str(call)
        ]
        self.assertGreaterEqual(len(html_write_calls), 1,
                                msg="Expected report.html to be opened for writing")


if __name__ == "__main__":
    unittest.main()
