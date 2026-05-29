"""Tests for bench/run.py fixed-concurrency benchmark."""

import json
import os
import sys
import unittest
from unittest.mock import patch, MagicMock

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from run import _parse_args, _run_bench, _collect_stable_sample
from latency_tracker import LatencyTracker


def _make_response(status, body_bytes):
    mock_response = MagicMock()
    mock_response.status = status
    mock_response.read.return_value = body_bytes
    mock_response.__enter__.return_value = mock_response
    return mock_response


def _make_side_effect_fn(seed_and_cleanup_responses, bench_response):
    """Build a side_effect function that serves seed/cleanup responses first,
    then returns a default bench response for any additional generate calls.
    """
    seed_and_cleanup_iter = iter(seed_and_cleanup_responses)

    def side_effect(*args, **kwargs):
        try:
            return next(seed_and_cleanup_iter)
        except StopIteration:
            return bench_response

    return side_effect


def _default_bench_responses(scoped_key="arche_k_x"):
    client = _make_response(200, b'{"id": "c-1", "name": "bench-scratch"}')
    key = _make_response(
        200,
        json.dumps({
            "id": "k-1", "key": scoped_key, "name": "benchmark-key",
            "permissions": ["generate"],
        }).encode(),
    )
    bp = _make_response(200, b'{"id": "bp-x", "name": "x"}')
    gen = _make_response(200, b'{"ok": true}')
    del_key = _make_response(200, b'{"ok": true}')
    del_client = _make_response(200, b'{"ok": true}')
    return client, key, bp, gen, del_key, del_client


class TestRunBench(unittest.TestCase):
    maxDiff = None

    def test_seeds_data_and_runs_bench_then_cleans_up(self):
        """Verify _run_bench seeds, fires generate requests, and cleans up."""
        client_resp, key_resp, bp_resp, gen_resp, del_key_resp, del_client_resp = (
            _default_bench_responses("arche_k_secret")
        )

        seed_responses = [client_resp, key_resp] + [bp_resp] * 10
        cleanup_responses = [del_key_resp, del_client_resp]

        side_effect = _make_side_effect_fn(
            seed_responses + cleanup_responses, gen_resp
        )

        with patch("urllib.request.urlopen", side_effect=side_effect) as mock_urlopen:
            with patch("builtins.print") as mock_print:
                _run_bench(
                    api_key="super-key",
                    target_url="http://localhost:8080",
                    concurrency=1,
                    duration=0.5,
                )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]

        # Verify seed calls
        client_calls = [
            c for c in calls
            if c.full_url == "http://localhost:8080/api/clients" and c.method == "POST"
        ]
        self.assertEqual(len(client_calls), 1, msg="Expected 1 POST to /api/clients")

        key_calls = [
            c for c in calls
            if "keys" in c.full_url and c.method == "POST"
        ]
        self.assertEqual(len(key_calls), 1, msg="Expected 1 POST to .../keys")

        bp_calls = [
            c for c in calls
            if "blueprints" in c.full_url and c.method == "POST"
        ]
        self.assertEqual(len(bp_calls), 10, msg="Expected 10 POST to .../blueprints")

        # Verify generate requests were made
        gen_calls = [
            c for c in calls
            if c.full_url == "http://localhost:8080/api/generate"
        ]
        self.assertGreaterEqual(
            len(gen_calls), 1,
            msg="Expected at least 1 generate request",
        )

        # Verify cleanup: DELETE keys and DELETE client
        delete_key_calls = [
            c for c in calls
            if c.method == "DELETE" and "keys" in c.full_url
        ]
        self.assertEqual(len(delete_key_calls), 1, msg="Expected 1 DELETE call for the API key")

        delete_client_calls = [
            c for c in calls
            if c.method == "DELETE" and "clients" in c.full_url
            and "keys" not in c.full_url
        ]
        self.assertEqual(len(delete_client_calls), 1, msg="Expected 1 DELETE call for the client")

        # Verify summary was printed
        printed_texts = [call[0][0] for call in mock_print.call_args_list]
        combined = " ".join(printed_texts)
        self.assertIn("generates/s", combined)
        self.assertIn("total requests", combined)

    def test_no_cleanup_flag_skips_deletion(self):
        """Verify --no-cleanup skips DELETE calls."""
        client_resp, key_resp, bp_resp, gen_resp, _, _ = _default_bench_responses()

        seed_responses = [client_resp, key_resp] + [bp_resp] * 10
        side_effect = _make_side_effect_fn(seed_responses, gen_resp)

        with patch("urllib.request.urlopen", side_effect=side_effect) as mock_urlopen:
            with patch("builtins.print"):
                _run_bench(
                    api_key="super-key",
                    target_url="http://localhost:8080",
                    concurrency=1,
                    duration=0.5,
                    no_cleanup=True,
                )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]
        delete_calls = [c for c in calls if c.method == "DELETE"]
        self.assertEqual(len(delete_calls), 0, msg="Expected no DELETE calls with --no-cleanup")

    def test_concurrency_5_spawns_multiple_workers(self):
        """Verify concurrency=5 uses multiple concurrent workers."""
        client_resp, key_resp, bp_resp, gen_resp, del_key_resp, del_client_resp = (
            _default_bench_responses()
        )

        seed_responses = [client_resp, key_resp] + [bp_resp] * 10
        cleanup_responses = [del_key_resp, del_client_resp]

        side_effect = _make_side_effect_fn(
            seed_responses + cleanup_responses, gen_resp
        )

        with patch("urllib.request.urlopen", side_effect=side_effect) as mock_urlopen:
            with patch("builtins.print"):
                _run_bench(
                    api_key="super-key",
                    target_url="http://localhost:8080",
                    concurrency=5,
                    duration=0.5,
                )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]
        gen_calls = [
            c for c in calls
            if c.full_url == "http://localhost:8080/api/generate"
        ]
        self.assertGreaterEqual(
            len(gen_calls), 5,
            msg="Expected at least 5 generate requests with concurrency=5",
        )

    def test_generates_per_second_is_printed(self):
        """Verify generates/s is printed in the output."""
        client_resp, key_resp, bp_resp, gen_resp, del_key_resp, del_client_resp = (
            _default_bench_responses()
        )

        seed_responses = [client_resp, key_resp] + [bp_resp] * 10
        cleanup_responses = [del_key_resp, del_client_resp]

        side_effect = _make_side_effect_fn(
            seed_responses + cleanup_responses, gen_resp
        )

        with patch("urllib.request.urlopen", side_effect=side_effect):
            with patch("builtins.print") as mock_print:
                _run_bench(
                    api_key="super-key",
                    target_url="http://localhost:8080",
                    concurrency=2,
                    duration=0.5,
                )

        printed_texts = [call[0][0] for call in mock_print.call_args_list]
        combined = " ".join(printed_texts)
        self.assertIn("generates/s", combined)
        self.assertIn("total requests", combined)
        self.assertIn("elapsed", combined)

    def test_bench_fires_generate_requests_with_scoped_key(self):
        """Verify generate requests use the scoped API key, not the super key."""
        client_resp, key_resp, bp_resp, gen_resp, del_key_resp, del_client_resp = (
            _default_bench_responses("arche_k_scoped")
        )

        seed_responses = [client_resp, key_resp] + [bp_resp] * 10
        cleanup_responses = [del_key_resp, del_client_resp]

        side_effect = _make_side_effect_fn(
            seed_responses + cleanup_responses, gen_resp
        )

        with patch("urllib.request.urlopen", side_effect=side_effect) as mock_urlopen:
            with patch("builtins.print"):
                _run_bench(
                    api_key="super-key",
                    target_url="http://localhost:8080",
                    concurrency=1,
                    duration=0.5,
                )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]
        gen_calls = [
            c for c in calls
            if c.full_url == "http://localhost:8080/api/generate"
        ]
        self.assertGreater(len(gen_calls), 0, msg="Expected at least 1 generate call")
        for call in gen_calls[:3]:
            headers = dict(call.headers)
            self.assertEqual(
                headers.get("X-api-key"),
                "arche_k_scoped",
                msg="Generate requests should use scoped key, not super key",
            )

    def test_cleanup_uses_super_key(self):
        """Verify cleanup DELETE calls use the super admin key."""
        client_resp, key_resp, bp_resp, gen_resp, del_key_resp, del_client_resp = (
            _default_bench_responses()
        )

        seed_responses = [client_resp, key_resp] + [bp_resp] * 10
        cleanup_responses = [del_key_resp, del_client_resp]

        side_effect = _make_side_effect_fn(
            seed_responses + cleanup_responses, gen_resp
        )

        with patch("urllib.request.urlopen", side_effect=side_effect) as mock_urlopen:
            with patch("builtins.print"):
                _run_bench(
                    api_key="super-key",
                    target_url="http://localhost:8080",
                    concurrency=1,
                    duration=0.5,
                )

        calls = [call[0][0] for call in mock_urlopen.call_args_list]
        delete_calls = [c for c in calls if c.method == "DELETE"]
        for call in delete_calls:
            headers = dict(call.headers)
            self.assertEqual(
                headers.get("X-api-key"),
                "super-key",
                msg="Cleanup DELETE calls should use the super admin key",
            )


class TestRunBenchRampUp(unittest.TestCase):
    """Tests for _run_bench with ramp_up=True."""

    def setUp(self):
        self.client_resp, self.key_resp, self.bp_resp, self.gen_resp, \
            self.del_key_resp, self.del_client_resp = _default_bench_responses()

    def test_ramp_up_collects_stable_sample_and_prints_percentiles(self):
        """Verify ramp-up path collects stable sample and prints percentiles."""
        seed_responses = [self.client_resp, self.key_resp] + [self.bp_resp] * 10
        cleanup_responses = [self.del_key_resp, self.del_client_resp]
        side_effect = _make_side_effect_fn(
            seed_responses + cleanup_responses, self.gen_resp
        )

        with patch("urllib.request.urlopen", side_effect=side_effect):
            with patch("run.run_ramp_up", return_value=(5, 100.0, "plateau")):
                with patch("run.SAMPLE_DURATION", 0.2):
                    with patch("builtins.print") as mock_print:
                        _run_bench(
                            api_key="super-key",
                            target_url="http://localhost:8080",
                            concurrency=1,
                            duration=0.5,
                            ramp_up=True,
                        )

        printed_texts = [call[0][0] for call in mock_print.call_args_list]
        combined = " ".join(printed_texts)
        self.assertIn("peak throughput", combined)
        self.assertIn("final concurrency", combined)
        self.assertIn("stable sample", combined)
        self.assertIn("p50:", combined)
        self.assertIn("p95:", combined)
        self.assertIn("p99:", combined)

    def test_ramp_up_warns_when_stable_sample_too_small(self):
        """Verify ramp-up path warns when stable sample has < 50 requests."""
        seed_responses = [self.client_resp, self.key_resp] + [self.bp_resp] * 10
        cleanup_responses = [self.del_key_resp, self.del_client_resp]
        side_effect = _make_side_effect_fn(
            seed_responses + cleanup_responses, self.gen_resp
        )

        with patch("urllib.request.urlopen", side_effect=side_effect):
            with patch("run.run_ramp_up", return_value=(0, 0.0, "plateau")):
                with patch("run.SAMPLE_DURATION", 0.2):
                    with patch("builtins.print") as mock_print:
                        _run_bench(
                            api_key="super-key",
                            target_url="http://localhost:8080",
                            concurrency=1,
                            duration=0.5,
                            ramp_up=True,
                        )

        printed_texts = [call[0][0] for call in mock_print.call_args_list]
        combined = " ".join(printed_texts)
        self.assertIn("stable sample too small", combined)
        self.assertIn("skipping percentiles", combined)


class TestCollectStableSample(unittest.TestCase):
    """Tests for _collect_stable_sample in isolation."""

    def test_collects_latencies(self):
        """Verify latencies are recorded in the tracker."""
        gen_resp = _make_response(200, b'{"ok": true}')

        with patch("urllib.request.urlopen", return_value=gen_resp):
            tracker = _collect_stable_sample(
                "key", "http://localhost:8080", concurrency=2, sample_duration=0.3,
            )

        self.assertGreaterEqual(tracker.count, 1)

    def test_nonzero_concurrency_produces_requests(self):
        """Any positive concurrency produces at least 1 request."""
        gen_resp = _make_response(200, b'{"ok": true}')

        with patch("urllib.request.urlopen", return_value=gen_resp):
            tracker = _collect_stable_sample(
                "key", "http://localhost:8080", concurrency=3, sample_duration=0.2,
            )

        self.assertGreaterEqual(tracker.count, 1)
        self.assertGreater(tracker.p50(), 0.0)

    def test_records_elapsed_ms(self):
        """Verify the durations recorded are reasonable (positive floats)."""
        gen_resp = _make_response(200, b'{"ok": true}')

        with patch("urllib.request.urlopen", return_value=gen_resp):
            tracker = _collect_stable_sample(
                "key", "http://localhost:8080", concurrency=1, sample_duration=0.3,
            )

        if tracker.count > 0:
            self.assertGreater(tracker.p50(), 0.0)

    def test_empty_sample_with_zero_concurrency(self):
        """Zero concurrency yields empty tracker."""
        gen_resp = _make_response(200, b'{"ok": true}')

        with patch("urllib.request.urlopen", return_value=gen_resp):
            tracker = _collect_stable_sample(
                "key", "http://localhost:8080", concurrency=0, sample_duration=0.2,
            )

        self.assertEqual(tracker.count, 0)


class TestParseArgsBench(unittest.TestCase):
    def test_bench_flag(self):
        args = _parse_args(["--api-key", "super-key", "--bench"])
        self.assertTrue(args.bench)

    def test_bench_default_concurrency(self):
        args = _parse_args(["--api-key", "super-key", "--bench"])
        self.assertEqual(args.concurrency, 1)

    def test_bench_custom_concurrency(self):
        args = _parse_args(["--api-key", "super-key", "--bench", "--concurrency", "5"])
        self.assertEqual(args.concurrency, 5)

    def test_bench_default_duration(self):
        args = _parse_args(["--api-key", "super-key", "--bench"])
        self.assertEqual(args.duration, 10)

    def test_bench_custom_duration(self):
        args = _parse_args(["--api-key", "super-key", "--bench", "--duration", "30"])
        self.assertEqual(args.duration, 30)

    def test_bench_no_cleanup(self):
        args = _parse_args(["--api-key", "super-key", "--bench", "--no-cleanup"])
        self.assertTrue(args.no_cleanup)

    def test_bench_no_cleanup_default_false(self):
        args = _parse_args(["--api-key", "super-key", "--bench"])
        self.assertFalse(args.no_cleanup)

    def test_bench_with_target_url(self):
        args = _parse_args([
            "--api-key", "super-key", "--bench",
            "--target-url", "http://other:8080",
        ])
        self.assertEqual(args.target_url, "http://other:8080")

    def test_ramp_up_flag(self):
        args = _parse_args(["--api-key", "super-key", "--bench", "--ramp-up"])
        self.assertTrue(args.ramp_up)

    def test_ramp_up_default_false(self):
        args = _parse_args(["--api-key", "super-key", "--bench"])
        self.assertFalse(args.ramp_up)

    def test_ramp_up_with_duration(self):
        args = _parse_args([
            "--api-key", "super-key", "--bench", "--ramp-up", "--duration", "20",
        ])
        self.assertTrue(args.ramp_up)
        self.assertEqual(args.duration, 20)


if __name__ == "__main__":
    unittest.main()
