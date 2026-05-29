"""Tests for bench/latency_tracker.py latency tracker."""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from latency_tracker import LatencyTracker


class TestLatencyTracker(unittest.TestCase):

    def test_records_durations(self):
        tracker = LatencyTracker()
        tracker.record(1.0)
        self.assertEqual(tracker.count, 1)

    def test_count_multiple_records(self):
        tracker = LatencyTracker()
        for d in [1.0, 2.0, 3.0]:
            tracker.record(d)
        self.assertEqual(tracker.count, 3)

    def test_p50_with_odd_count(self):
        tracker = LatencyTracker()
        for d in [1.0, 2.0, 3.0, 4.0, 5.0]:
            tracker.record(d)
        self.assertEqual(tracker.p50(), 3.0)

    def test_p50_with_even_count(self):
        tracker = LatencyTracker()
        for d in [1.0, 2.0, 3.0, 4.0]:
            tracker.record(d)
        self.assertEqual(tracker.p50(), 2.0)

    def test_p50_single_element(self):
        tracker = LatencyTracker()
        tracker.record(42.0)
        self.assertEqual(tracker.p50(), 42.0)

    def test_p50_two_elements(self):
        tracker = LatencyTracker()
        tracker.record(1.0)
        tracker.record(2.0)
        self.assertEqual(tracker.p50(), 1.0)

    def test_p95(self):
        tracker = LatencyTracker()
        durations = list(range(1, 101))
        for d in durations:
            tracker.record(float(d))
        self.assertEqual(tracker.p95(), 95.0)

    def test_p99(self):
        tracker = LatencyTracker()
        durations = list(range(1, 101))
        for d in durations:
            tracker.record(float(d))
        self.assertEqual(tracker.p99(), 99.0)

    def test_percentiles_with_unsorted_input(self):
        tracker = LatencyTracker()
        for d in [5.0, 1.0, 3.0, 2.0, 4.0]:
            tracker.record(d)
        self.assertEqual(tracker.p50(), 3.0)

    def test_known_percentiles_10_elements(self):
        tracker = LatencyTracker()
        for d in [10, 20, 30, 40, 50, 60, 70, 80, 90, 100]:
            tracker.record(float(d))
        self.assertEqual(tracker.p50(), 50.0)
        self.assertEqual(tracker.p95(), 100.0)
        self.assertEqual(tracker.p99(), 100.0)

    def test_empty_tracker_raises_on_p50(self):
        tracker = LatencyTracker()
        with self.assertRaises(ValueError):
            tracker.p50()

    def test_empty_tracker_raises_on_p95(self):
        tracker = LatencyTracker()
        with self.assertRaises(ValueError):
            tracker.p95()

    def test_empty_tracker_raises_on_p99(self):
        tracker = LatencyTracker()
        with self.assertRaises(ValueError):
            tracker.p99()

    def test_empty_tracker_count_zero(self):
        tracker = LatencyTracker()
        self.assertEqual(tracker.count, 0)

    def test_known_values_p50(self):
        durations = [2.5, 7.1, 3.8, 5.0, 9.2]
        tracker = LatencyTracker()
        for d in durations:
            tracker.record(d)
        self.assertAlmostEqual(tracker.p50(), 5.0)

    def test_known_values_p95(self):
        durations = [2.5, 7.1, 3.8, 5.0, 9.2]
        tracker = LatencyTracker()
        for d in durations:
            tracker.record(d)
        self.assertAlmostEqual(tracker.p95(), 9.2)

    def test_known_values_p99(self):
        durations = [2.5, 7.1, 3.8, 5.0, 9.2]
        tracker = LatencyTracker()
        for d in durations:
            tracker.record(d)
        self.assertAlmostEqual(tracker.p99(), 9.2)

    def test_large_sample_p99(self):
        tracker = LatencyTracker()
        n = 1000
        for i in range(1, n + 1):
            tracker.record(float(i))
        self.assertEqual(tracker.p99(), 990.0)

    def test_large_sample_p95(self):
        tracker = LatencyTracker()
        n = 1000
        for i in range(1, n + 1):
            tracker.record(float(i))
        self.assertEqual(tracker.p95(), 950.0)


if __name__ == "__main__":
    unittest.main()
