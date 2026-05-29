"""
Ramp-up benchmark engine.

Starts with 1 concurrent task, adds 1 every `ramp_interval` seconds,
maintains a rolling `window_size`-second throughput window,
and tracks peak throughput.

The `run_ramp_up` function accepts a callback for testability with
synthetic timestamps.
"""


def run_ramp_up(callback, *, duration=30, ramp_interval=1.5, window_size=10.0):
    """Run a ramp-up benchmark.

    Args:
        callback: called with (concurrency) for each ramp step. Returns
                  an iterable of completion timestamps (seconds since
                  benchmark start).
        duration: total benchmark duration in seconds.
        ramp_interval: seconds between concurrency increases.
        window_size: rolling window for throughput computation in seconds.

    Returns:
        (final_concurrency, peak_throughput)
    """
    all_ts = []
    concurrency = 1
    peak = 0.0
    elapsed = 0.0

    while elapsed < duration:
        step_ts = callback(concurrency)
        if step_ts:
            all_ts.extend(step_ts)
        elapsed += ramp_interval
        concurrency += 1

    final_concurrency = concurrency - 1

    for tick in range(1, int(elapsed) + 1):
        cutoff = tick - window_size
        count = sum(1 for t in all_ts if t > cutoff)
        throughput = count / window_size
        if throughput > peak:
            peak = throughput

    return final_concurrency, peak
