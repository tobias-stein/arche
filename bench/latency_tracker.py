"""
Latency tracker for benchmark suite.

Records per-request durations and computes percentiles.
"""

import bisect


class LatencyTracker:
    """Records per-request durations and computes percentiles.

    Durations are stored in a sorted list for O(n) compute of
    arbitrary percentiles.
    """

    def __init__(self):
        self._durations = []

    def record(self, duration_ms: float) -> None:
        """Record a single request duration in milliseconds."""
        bisect.insort(self._durations, duration_ms)

    @property
    def count(self) -> int:
        """Number of recorded durations."""
        return len(self._durations)

    def p50(self) -> float:
        """50th percentile (median) of recorded durations."""
        return self._percentile(50)

    def p95(self) -> float:
        """95th percentile of recorded durations."""
        return self._percentile(95)

    def p99(self) -> float:
        """99th percentile of recorded durations."""
        return self._percentile(99)

    def _percentile(self, p: int) -> float:
        """Compute the p-th percentile of recorded durations.

        Uses the nearest-rank method.
        Raises ValueError if no durations have been recorded.
        """
        if not self._durations:
            raise ValueError("no durations recorded")
        n = len(self._durations)
        rank = max(1, int(p / 100.0 * n + 0.5))
        rank = min(rank, n)
        return self._durations[rank - 1]
