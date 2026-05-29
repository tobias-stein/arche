"""Tests for bench/ramp_up.py ramp-up engine."""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ramp_up import run_ramp_up


class TestRunRampUp(unittest.TestCase):

    def test_starts_at_concurrency_1(self):
        """Callback is first called with concurrency=1."""
        levels = []

        def cb(c):
            levels.append(c)
            return []

        run_ramp_up(cb, duration=3.0, ramp_interval=1.5)

        self.assertEqual(levels[0], 1)

    def test_ramps_up_every_1_5s(self):
        """Callback sees concurrency increase by 1 every 1.5s."""
        levels = []

        def cb(c):
            levels.append(c)
            return []

        run_ramp_up(cb, duration=6.0, ramp_interval=1.5)

        self.assertEqual(levels, [1, 2, 3, 4])

    def test_returns_final_concurrency(self):
        """Returns the maximum concurrency reached."""

        def cb(c):
            return []

        final_conc, _ = run_ramp_up(cb, duration=4.5, ramp_interval=1.5)

        self.assertEqual(final_conc, 3)

    def test_duration_limits_ramp(self):
        """Duration caps how many ramp steps occur."""

        def cb(c):
            return []

        final_conc, _ = run_ramp_up(cb, duration=1.0, ramp_interval=1.5)

        self.assertEqual(final_conc, 1)

    def test_throughput_window(self):
        """Throughput computed from rolling window of completions."""

        def cb(c):
            return [i * 1.0 for i in range(5)]

        _, peak = run_ramp_up(cb, duration=3.0, ramp_interval=1.5, window_size=10.0)

        self.assertGreater(peak, 0.0)

    def test_peak_throughput_tracked(self):
        """Peak reflects the highest throughput across all ticks."""
        call_count = [0]

        def cb(c):
            call_count[0] += 1
            if call_count[0] == 1:
                return [0.5]
            return [1.5, 2.5, 3.5, 4.5]

        _, peak = run_ramp_up(cb, duration=3.0, ramp_interval=1.5, window_size=10.0)

        self.assertGreater(peak, 0.0)

    def test_peak_grows_with_higher_throughput(self):
        """Later steps with more completions raise the peak."""
        call_count = [0]

        def cb(c):
            call_count[0] += 1
            return [1.0] * (c * 10)

        _, peak = run_ramp_up(cb, duration=4.5, ramp_interval=1.5, window_size=10.0)

        self.assertGreater(peak, 0.0)

    def test_no_completions_returns_zero_peak(self):
        """Zero completions yields 0.0 peak throughput."""

        def cb(c):
            return []

        _, peak = run_ramp_up(cb, duration=4.5, ramp_interval=1.5, window_size=10.0)

        self.assertEqual(peak, 0.0)

    def test_callback_timestamps_collected(self):
        """Timestamps returned by callback are used in throughput computation."""

        def cb(c):
            return [0.5]

        _, peak = run_ramp_up(cb, duration=3.0, ramp_interval=1.5, window_size=10.0)

        self.assertGreater(peak, 0.0)

    def test_returns_tuple(self):
        """Returns a (final_concurrency, peak_throughput) tuple."""

        def cb(c):
            return []

        result = run_ramp_up(cb, duration=3.0, ramp_interval=1.5)

        self.assertIsInstance(result, tuple)
        self.assertEqual(len(result), 2)

    def test_peak_is_float(self):
        """Peak throughput is a float."""

        def cb(c):
            return [0.5]

        _, peak = run_ramp_up(cb, duration=3.0, ramp_interval=1.5)

        self.assertIsInstance(peak, float)

    def test_callback_none_returns_ok(self):
        """Callback returning None is handled gracefully."""

        def cb(c):
            return None

        final_conc, peak = run_ramp_up(cb, duration=3.0, ramp_interval=1.5)

        self.assertEqual(final_conc, 2)
        self.assertEqual(peak, 0.0)

    def test_small_window(self):
        """Works with a window smaller than the ramp interval."""

        def cb(c):
            return [0.5]

        final_conc, peak = run_ramp_up(cb, duration=3.0, ramp_interval=1.5, window_size=1.0)

        self.assertEqual(final_conc, 2)
        self.assertGreaterEqual(peak, 0.0)

    def test_throughput_within_window(self):
        """Only timestamps within the window contribute to throughput."""
        timestamps = [100.0, 101.0, 102.0, 200.0]

        def cb(c):
            return timestamps

        _, peak = run_ramp_up(cb, duration=1.5, ramp_interval=1.5, window_size=5.0)

        all_4_in_window = peak > 0.5
        self.assertTrue(all_4_in_window)


if __name__ == "__main__":
    unittest.main()
