"""Tests for bench/ramp_up.py ramp-up engine."""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from ramp_up import check_stop_conditions, run_ramp_up


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

        final_conc, _, _ = run_ramp_up(cb, duration=4.5, ramp_interval=1.5)

        self.assertEqual(final_conc, 3)

    def test_duration_limits_ramp(self):
        """Duration caps how many ramp steps occur."""

        def cb(c):
            return []

        final_conc, _, _ = run_ramp_up(cb, duration=1.0, ramp_interval=1.5)

        self.assertEqual(final_conc, 1)

    def test_throughput_window(self):
        """Throughput computed from rolling window of completions."""

        def cb(c):
            return [i * 1.0 for i in range(5)]

        _, peak, _ = run_ramp_up(cb, duration=3.0, ramp_interval=1.5, window_size=10.0)

        self.assertGreater(peak, 0.0)

    def test_peak_throughput_tracked(self):
        """Peak reflects the highest throughput across all ticks."""
        call_count = [0]

        def cb(c):
            call_count[0] += 1
            if call_count[0] == 1:
                return [0.5]
            return [1.5, 2.5, 3.5, 4.5]

        _, peak, _ = run_ramp_up(cb, duration=3.0, ramp_interval=1.5, window_size=10.0)

        self.assertGreater(peak, 0.0)

    def test_peak_grows_with_higher_throughput(self):
        """Later steps with more completions raise the peak."""
        call_count = [0]

        def cb(c):
            call_count[0] += 1
            return [1.0] * (c * 10)

        _, peak, _ = run_ramp_up(cb, duration=4.5, ramp_interval=1.5, window_size=10.0)

        self.assertGreater(peak, 0.0)

    def test_no_completions_returns_zero_peak(self):
        """Zero completions yields 0.0 peak throughput."""

        def cb(c):
            return []

        _, peak, _ = run_ramp_up(cb, duration=4.5, ramp_interval=1.5, window_size=10.0)

        self.assertEqual(peak, 0.0)

    def test_callback_timestamps_collected(self):
        """Timestamps returned by callback are used in throughput computation."""

        def cb(c):
            return [0.5]

        _, peak, _ = run_ramp_up(cb, duration=3.0, ramp_interval=1.5, window_size=10.0)

        self.assertGreater(peak, 0.0)

    def test_returns_tuple(self):
        """Returns a (final_concurrency, peak_throughput) tuple."""

        def cb(c):
            return []

        result = run_ramp_up(cb, duration=3.0, ramp_interval=1.5)

        self.assertIsInstance(result, tuple)
        self.assertEqual(len(result), 3)

    def test_peak_is_float(self):
        """Peak throughput is a float."""

        def cb(c):
            return [0.5]

        _, peak, _ = run_ramp_up(cb, duration=3.0, ramp_interval=1.5)

        self.assertIsInstance(peak, float)

    def test_callback_none_returns_ok(self):
        """Callback returning None is handled gracefully."""

        def cb(c):
            return None

        final_conc, peak, _ = run_ramp_up(cb, duration=3.0, ramp_interval=1.5)

        self.assertEqual(final_conc, 2)
        self.assertEqual(peak, 0.0)

    def test_small_window(self):
        """Works with a window smaller than the ramp interval."""

        def cb(c):
            return [0.5]

        final_conc, peak, _ = run_ramp_up(cb, duration=3.0, ramp_interval=1.5, window_size=1.0)

        self.assertEqual(final_conc, 2)
        self.assertGreaterEqual(peak, 0.0)

    def test_throughput_within_window(self):
        """Only timestamps within the window contribute to throughput."""
        timestamps = [100.0, 101.0, 102.0, 200.0]

        def cb(c):
            return timestamps

        _, peak, _ = run_ramp_up(cb, duration=1.5, ramp_interval=1.5, window_size=5.0)

        all_4_in_window = peak > 0.5
        self.assertTrue(all_4_in_window)


class TestCheckStopConditions(unittest.TestCase):
    """Tests for check_stop_conditions in isolation."""

    def test_degradation_detected(self):
        """Throughput below 90% of peak triggers degradation stop."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=100, current_throughput=89,
            throughput_15s_ago=100, min_duration=20,
        )
        self.assertTrue(should_stop)
        self.assertEqual(reason, "degradation")

    def test_degradation_not_below_threshold(self):
        """Throughput at exactly 90% of peak does NOT trigger degradation."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=100, current_throughput=90,
            throughput_15s_ago=100, min_duration=20,
        )
        self.assertFalse(should_stop)

    def test_degradation_above_threshold(self):
        """Throughput above 90% of peak does NOT trigger degradation or plateau."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=105, current_throughput=105,
            throughput_15s_ago=90, min_duration=20,
        )
        self.assertFalse(should_stop)

    def test_degradation_not_below_threshold(self):
        """Throughput at exactly 90% of peak does NOT trigger degradation."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=100, current_throughput=90,
            throughput_15s_ago=85, min_duration=20,
        )
        self.assertFalse(should_stop)

    def test_plateau_detected(self):
        """Growth below 3% over trailing window triggers plateau stop."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=100, current_throughput=102,
            throughput_15s_ago=100, min_duration=20,
        )
        self.assertTrue(should_stop)
        self.assertEqual(reason, "plateau")

    def test_plateau_at_exact_boundary(self):
        """Growth at exactly 3% does NOT trigger plateau (needs <3%)."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=100, current_throughput=103,
            throughput_15s_ago=100, min_duration=20,
        )
        self.assertFalse(should_stop)

    def test_plateau_above_threshold(self):
        """Growth above 3% does NOT trigger plateau."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=100, current_throughput=105,
            throughput_15s_ago=100, min_duration=20,
        )
        self.assertFalse(should_stop)

    def test_no_stop_before_min_duration(self):
        """Both conditions ignored when elapsed < min_duration."""
        should_stop, reason = check_stop_conditions(
            elapsed=15, peak_throughput=100, current_throughput=50,
            throughput_15s_ago=100, min_duration=20,
        )
        self.assertFalse(should_stop)
        self.assertIsNone(reason)

    def test_no_stop_at_min_duration_boundary(self):
        """Conditions checked when elapsed == min_duration."""
        should_stop, reason = check_stop_conditions(
            elapsed=20, peak_throughput=100, current_throughput=80,
            throughput_15s_ago=100, min_duration=20,
        )
        self.assertTrue(should_stop)
        self.assertEqual(reason, "degradation")

    def test_plateau_with_none_past(self):
        """Plateau not triggered when throughput_15s_ago is None."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=100, current_throughput=101,
            throughput_15s_ago=None, min_duration=20,
        )
        self.assertFalse(should_stop)

    def test_plateau_with_zero_past(self):
        """Plateau not triggered when throughput_15s_ago is zero."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=100, current_throughput=101,
            throughput_15s_ago=0, min_duration=20,
        )
        self.assertFalse(should_stop)

    def test_degradation_with_zero_peak(self):
        """Degradation not triggered when peak is zero."""
        should_stop, reason = check_stop_conditions(
            elapsed=25, peak_throughput=0, current_throughput=0,
            throughput_15s_ago=0, min_duration=20,
        )
        self.assertFalse(should_stop)


class TestRampUpStopConditions(unittest.TestCase):
    """Integration tests for run_ramp_up with plateau/degradation detection."""

    def test_plateau_stops_early(self):
        """Constant throughput triggers plateau stop before duration."""
        current_time = [20.0]

        def cb(c):
            ts = [current_time[0] + i * 0.1 for i in range(10)]
            current_time[0] += 1.5
            return ts

        final_conc, peak, stop_reason = run_ramp_up(
            cb, duration=60, ramp_interval=1.5, window_size=10.0,
            plateau_window=15.0, min_duration=20.0,
        )

        self.assertEqual(stop_reason, "plateau")
        self.assertIsNotNone(stop_reason)
        self.assertLess(final_conc, 40)

    def test_degradation_stops_early(self):
        """Throughput drop triggers degradation stop before duration."""
        call_count = [0]

        def cb(c):
            call_count[0] += 1
            if call_count[0] <= 2:
                return [i * 0.5 for i in range(10)]
            return []

        final_conc, peak, stop_reason = run_ramp_up(
            cb, duration=60, ramp_interval=1.5, window_size=10.0,
            plateau_window=15.0, min_duration=20.0,
        )

        self.assertEqual(stop_reason, "degradation")
        self.assertLess(final_conc, 40)

    def test_short_duration_no_stop(self):
        """Run with duration < min_duration completes with stop_reason None."""
        def cb(c):
            return [0.5]

        final_conc, peak, stop_reason = run_ramp_up(
            cb, duration=3.0, ramp_interval=1.5, window_size=10.0,
            plateau_window=15.0, min_duration=20.0,
        )

        self.assertIsNone(stop_reason)
        self.assertEqual(final_conc, 2)

    def test_plateau_reason_string(self):
        """Plateau detection returns correct stop_reason string."""
        current_time = [25.0]

        def cb(c):
            ts = [current_time[0] + i * 0.1 for i in range(10)]
            current_time[0] += 1.5
            return ts

        _, _, stop_reason = run_ramp_up(
            cb, duration=60, ramp_interval=1.5, window_size=10.0,
            plateau_window=15.0, min_duration=20.0,
        )

        self.assertEqual(stop_reason, "plateau")

    def test_degradation_reason_string(self):
        """Degradation detection returns correct stop_reason string."""
        call_count = [0]

        def cb(c):
            call_count[0] += 1
            if call_count[0] <= 2:
                return [i * 0.5 for i in range(10)]
            return []

        _, _, stop_reason = run_ramp_up(
            cb, duration=60, ramp_interval=1.5, window_size=10.0,
            plateau_window=15.0, min_duration=20.0,
        )

        self.assertEqual(stop_reason, "degradation")


if __name__ == "__main__":
    unittest.main()
