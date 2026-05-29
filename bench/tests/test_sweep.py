"""Tests for bench/run.py multi-scenario sweep."""

import csv
import io
import json
import os
import sys
import unittest
from unittest.mock import patch, MagicMock

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from run import _parse_args, _run_sweep, _print_summary_table


def _make_mock_response(status, body_bytes):
    mock_response = MagicMock()
    mock_response.status = status
    mock_response.read.return_value = body_bytes
    mock_response.__enter__.return_value = mock_response
    return mock_response


CLIENT_RESP = _make_mock_response(
    200, json.dumps({"id": "c-xxx", "name": "bench-test"}).encode(),
)
KEY_RESP = _make_mock_response(
    200,
    json.dumps({
        "id": "k-xxx", "key": "arche_k_scoped",
        "name": "benchmark-key", "permissions": ["generate"],
    }).encode(),
)
BP_RESP = _make_mock_response(200, b'{"id": "bp-x", "name": "x"}')
GEN_RESP = _make_mock_response(200, b'{"ok": true}')
DELETE_RESP = _make_mock_response(200, b'{"ok": true}')


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

    return _make_mock_response(200, b'{"ok": true}')


class TestParseArgsSweep(unittest.TestCase):
    def test_sweep_flag(self):
        args = _parse_args(["--api-key", "super-key", "--sweep"])
        self.assertTrue(args.sweep)

    def test_sweep_with_concurrency(self):
        args = _parse_args(["--api-key", "k", "--sweep", "--concurrency", "5"])
        self.assertEqual(args.concurrency, 5)

    def test_sweep_with_duration(self):
        args = _parse_args(["--api-key", "k", "--sweep", "--duration", "30"])
        self.assertEqual(args.duration, 30)

    def test_sweep_not_set_by_default(self):
        args = _parse_args(["--api-key", "k"])
        self.assertFalse(args.sweep)

    def test_sweep_with_config(self):
        args = _parse_args(["--api-key", "k", "--sweep", "--config", "/tmp/my.yaml"])
        self.assertEqual(args.config, "/tmp/my.yaml")


def _make_mock_tracker(count=100, p50=1.2, p95=2.8, p99=4.1):
    tracker = MagicMock()
    tracker.count = count
    tracker.p50.return_value = p50
    tracker.p95.return_value = p95
    tracker.p99.return_value = p99
    return tracker


class TestRunSweep(unittest.TestCase):
    maxDiff = None

    def setUp(self):
        self.cs_patcher = patch("run._collect_stable_sample", return_value=_make_mock_tracker())
        self.cs_patcher.start()

    def tearDown(self):
        self.cs_patcher.stop()

    def test_sweep_processes_all_scenarios(self):
        """Verify _run_sweep processes all scenarios, creates named clients, and cleans up."""
        scenarios = [
            {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3},
            {"blueprint_count": 10, "affix_count": 2, "attribute_count": 3},
        ]

        with patch("run.load_config", return_value=scenarios):
            with patch("urllib.request.urlopen", side_effect=_smart_side_effect) as mock_urlopen:
                with patch("builtins.print"):
                    with patch("run.os.makedirs"):
                        with patch("builtins.open"):
                            _run_sweep(
                                api_key="super-key",
                                target_url="http://localhost:8080",
                                config_path="dummy.yaml",
                                concurrency=1,
                                duration=0.1,
                            )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]

        post_client_calls = [
            c for c in calls
            if c.full_url == "http://localhost:8080/api/clients" and c.method == "POST"
        ]
        self.assertEqual(len(post_client_calls), 2, "Expected 2 client creations")

        client_names = set()
        for c in post_client_calls:
            body = json.loads(c.data)
            client_names.add(body["name"])
        self.assertIn("bench-bp10-aff0-attr3", client_names)
        self.assertIn("bench-bp10-aff2-attr3", client_names)

        delete_key_calls = [
            c for c in calls
            if c.method == "DELETE" and "keys" in c.full_url
        ]
        self.assertEqual(len(delete_key_calls), 2, "Expected 2 DELETE key calls")

        delete_client_calls = [
            c for c in calls
            if c.method == "DELETE" and "clients" in c.full_url and "keys" not in c.full_url
        ]
        self.assertEqual(len(delete_client_calls), 2, "Expected 2 DELETE client calls")

    def test_sweep_writes_csv(self):
        """Verify CSV output has correct columns and rows."""
        scenarios = [
            {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3},
        ]

        with patch("run.load_config", return_value=scenarios):
            with patch("urllib.request.urlopen", side_effect=_smart_side_effect):
                with patch("builtins.print"):
                    with patch("run.os.makedirs"):
                        with patch("builtins.open") as mock_open:
                            mock_file = io.StringIO()
                            mock_open.return_value.__enter__.return_value = mock_file

                            _run_sweep(
                                api_key="super-key",
                                target_url="http://localhost:8080",
                                config_path="dummy.yaml",
                                concurrency=1,
                                duration=0.1,
                            )

                            mock_file.seek(0)
                            content = mock_file.read()

        self.assertIn("blueprint_count", content)
        self.assertIn("affix_count", content)
        self.assertIn("attribute_count", content)
        self.assertIn("concurrency", content)
        self.assertIn("duration_s", content)
        self.assertIn("total_requests", content)
        self.assertIn("throughput", content)
        self.assertIn("10", content)
        self.assertIn("0", content)
        self.assertIn("3", content)
        self.assertIn("1", content)

        mock_file.seek(0)
        reader = csv.DictReader(io.StringIO(content))
        rows = list(reader)
        self.assertEqual(len(rows), 1)

    def test_sweep_prints_progress(self):
        """Verify progress is printed for each scenario."""
        scenarios = [
            {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3},
            {"blueprint_count": 100, "affix_count": 2, "attribute_count": 10},
        ]

        with patch("run.load_config", return_value=scenarios):
            with patch("urllib.request.urlopen", side_effect=_smart_side_effect):
                with patch("run.os.makedirs"):
                    with patch("builtins.open"):
                        with patch("builtins.print") as mock_print:
                            _run_sweep(
                                api_key="super-key",
                                target_url="http://localhost:8080",
                                config_path="dummy.yaml",
                                concurrency=1,
                                duration=0.1,
                            )

        printed = []
        for call in mock_print.call_args_list:
            args, _ = call
            if args:
                printed.append(str(args[0]))
        combined = "\n".join(printed)
        self.assertIn("[1/2]", combined)
        self.assertIn("[2/2]", combined)
        self.assertIn("bp=10", combined)
        self.assertIn("bp=100", combined)

    def test_sweep_cleans_up_all_clients(self):
        """Verify all test clients are cleaned up."""
        scenarios = [
            {"blueprint_count": 1, "affix_count": 0, "attribute_count": 3},
            {"blueprint_count": 1, "affix_count": 2, "attribute_count": 3},
            {"blueprint_count": 1, "affix_count": 4, "attribute_count": 3},
        ]

        with patch("run.load_config", return_value=scenarios):
            with patch("urllib.request.urlopen", side_effect=_smart_side_effect) as mock_urlopen:
                with patch("builtins.print"):
                    with patch("run.os.makedirs"):
                        with patch("builtins.open"):
                            _run_sweep(
                                api_key="super-key",
                                target_url="http://localhost:8080",
                                config_path="dummy.yaml",
                                concurrency=1,
                                duration=0.1,
                            )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]
        delete_key_calls = [
            c for c in calls
            if c.method == "DELETE" and "keys" in c.full_url
        ]
        self.assertEqual(len(delete_key_calls), 3)

        delete_client_calls = [
            c for c in calls
            if c.method == "DELETE" and "clients" in c.full_url and "keys" not in c.full_url
        ]
        self.assertEqual(len(delete_client_calls), 3)

    def test_sweep_uses_scoped_key_for_generate(self):
        """Verify generate requests use scoped keys, not super key."""
        scenarios = [
            {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3},
        ]

        with patch("run.load_config", return_value=scenarios):
            with patch("urllib.request.urlopen", side_effect=_smart_side_effect) as mock_urlopen:
                with patch("builtins.print"):
                    with patch("run.os.makedirs"):
                        with patch("builtins.open"):
                            _run_sweep(
                                api_key="super-key",
                                target_url="http://localhost:8080",
                                config_path="dummy.yaml",
                                concurrency=1,
                                duration=0.1,
                            )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]
        gen_calls = [
            c for c in calls
            if c.full_url == "http://localhost:8080/api/generate"
        ]
        self.assertGreater(len(gen_calls), 0)
        for c in gen_calls[:3]:
            headers = dict(c.headers)
            self.assertEqual(headers.get("X-api-key"), "arche_k_scoped",
                             "Generate should use scoped key")

    def test_sweep_uses_super_key_for_cleanup(self):
        """Verify cleanup uses super admin key."""
        scenarios = [
            {"blueprint_count": 1, "affix_count": 0, "attribute_count": 3},
        ]

        with patch("run.load_config", return_value=scenarios):
            with patch("urllib.request.urlopen", side_effect=_smart_side_effect) as mock_urlopen:
                with patch("builtins.print"):
                    with patch("run.os.makedirs"):
                        with patch("builtins.open"):
                            _run_sweep(
                                api_key="super-key",
                                target_url="http://localhost:8080",
                                config_path="dummy.yaml",
                                concurrency=1,
                                duration=0.1,
                            )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]
        delete_calls = [c for c in calls if c.method == "DELETE"]
        self.assertGreater(len(delete_calls), 0)
        for c in delete_calls:
            headers = dict(c.headers)
            self.assertEqual(headers.get("X-api-key"), "super-key",
                             "Cleanup should use super key")


class TestSweepTerminalTable(unittest.TestCase):
    """Tests for terminal summary table at end of sweep."""

    def test_summary_table_has_all_columns(self):
        """Verify table includes Scenario, Peak gen/s, Concurrency, p50/p95/p99."""
        results = [
            {
                "blueprint_count": 10, "affix_count": 0, "attribute_count": 3,
                "concurrency": 1, "duration_s": 0.1, "total_requests": 500,
                "throughput": 854.0, "p50": 1.2, "p95": 2.8, "p99": 4.1,
            },
        ]

        with patch("builtins.print") as mock_print:
            _print_summary_table(results)

        printed = []
        for call in mock_print.call_args_list:
            args, _ = call
            if args:
                printed.append(str(args[0]))
        combined = "\n".join(printed)
        self.assertIn("Scenario", combined)
        self.assertIn("Peak gen/s", combined)
        self.assertIn("Concurrency", combined)
        self.assertIn("p50(ms)", combined)
        self.assertIn("p95(ms)", combined)
        self.assertIn("p99(ms)", combined)
        self.assertIn("854", combined)
        self.assertIn("bp=10", combined)
        self.assertIn("1.2", combined)
        self.assertIn("2.8", combined)
        self.assertIn("4.1", combined)

    def test_summary_table_footer_shows_overall_peak(self):
        """Verify footer shows overall peak throughput and best scenario."""
        results = [
            {
                "blueprint_count": 10, "affix_count": 0, "attribute_count": 3,
                "concurrency": 1, "duration_s": 0.1, "total_requests": 500,
                "throughput": 1000.0, "p50": 1.2, "p95": 2.8, "p99": 4.1,
            },
            {
                "blueprint_count": 100, "affix_count": 2, "attribute_count": 10,
                "concurrency": 1, "duration_s": 0.1, "total_requests": 200,
                "throughput": 500.0, "p50": 2.1, "p95": 5.3, "p99": 8.7,
            },
        ]

        with patch("builtins.print") as mock_print:
            _print_summary_table(results)

        printed = []
        for call in mock_print.call_args_list:
            args, _ = call
            if args:
                printed.append(str(args[0]))
        combined = "\n".join(printed)
        self.assertIn("Peak gen/s: 1000", combined)
        self.assertIn("bp=10 aff=0 attr=3", combined)

    def test_summary_table_handles_missing_latency(self):
        """Verify — is shown when latency data is absent."""
        results = [
            {
                "blueprint_count": 10, "affix_count": 0, "attribute_count": 3,
                "concurrency": 1, "duration_s": 0.1, "total_requests": 500,
                "throughput": 854.0, "p50": None, "p95": None, "p99": None,
            },
        ]

        with patch("builtins.print") as mock_print:
            _print_summary_table(results)

        printed = []
        for call in mock_print.call_args_list:
            args, _ = call
            if args:
                printed.append(str(args[0]))
        combined = "\n".join(printed)
        self.assertIn("\u2014", combined)

    def test_summary_table_empty_results_no_error(self):
        """Verify no error when there are no results."""
        with patch("builtins.print") as mock_print:
            _print_summary_table([])
        self.assertEqual(mock_print.call_count, 0)

    def test_summary_table_printed_at_end_of_sweep(self):
        """Verify _run_sweep prints summary table after all scenarios."""
        scenarios = [
            {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3},
            {"blueprint_count": 100, "affix_count": 2, "attribute_count": 10},
        ]

        mock_tracker = MagicMock()
        mock_tracker.count = 100
        mock_tracker.p50.return_value = 1.2
        mock_tracker.p95.return_value = 2.8
        mock_tracker.p99.return_value = 4.1

        with patch("run.load_config", return_value=scenarios):
            with patch("urllib.request.urlopen", side_effect=_smart_side_effect):
                with patch("run._collect_stable_sample", return_value=mock_tracker):
                    with patch("builtins.print") as mock_print:
                        with patch("run.os.makedirs"):
                            with patch("builtins.open"):
                                _run_sweep(
                                    api_key="super-key",
                                    target_url="http://localhost:8080",
                                    config_path="dummy.yaml",
                                    concurrency=1,
                                    duration=0.1,
                                )

        printed = []
        for call in mock_print.call_args_list:
            args, _ = call
            if args:
                printed.append(str(args[0]))
        combined = "\n".join(printed)
        self.assertIn("Scenario", combined)
        self.assertIn("Peak gen/s", combined)
        self.assertIn("Concurrency", combined)
        self.assertIn("p50(ms)", combined)
        self.assertIn("p95(ms)", combined)
        self.assertIn("p99(ms)", combined)

    def test_summary_table_auto_adjusts_column_widths(self):
        """Verify columns are wide enough for all content."""
        results = [
            {
                "blueprint_count": 10, "affix_count": 0, "attribute_count": 3,
                "concurrency": 1, "duration_s": 0.1, "total_requests": 500,
                "throughput": 100.5, "p50": 1.2, "p95": 2.8, "p99": 4.1,
            },
            {
                "blueprint_count": 1000, "affix_count": 4, "attribute_count": 25,
                "concurrency": 12, "duration_s": 0.1, "total_requests": 200,
                "throughput": 2100.0, "p50": 4.1, "p95": 11.2, "p99": 18.3,
            },
        ]

        with patch("builtins.print") as mock_print:
            _print_summary_table(results)

        printed = []
        for call in mock_print.call_args_list:
            args, _ = call
            if args:
                printed.append(str(args[0]))
        combined = "\n".join(printed)
        self.assertIn("bp=1000", combined)
        self.assertIn("2100", combined)


if __name__ == "__main__":
    unittest.main()
