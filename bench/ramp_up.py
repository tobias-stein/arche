"""
Ramp-up benchmark engine.

Starts with 1 concurrent task, adds 1 every `ramp_interval` seconds,
maintains a rolling `window_size`-second throughput window,
and tracks peak throughput.

The `run_ramp_up` function accepts a callback for testability with
synthetic timestamps.  Plateau / degradation detection stops the run
early when throughput plateaus (growth <3% over `plateau_window`) or
degrades (current <90% of peak), subject to a `min_duration` floor.
"""


def check_stop_conditions(*, elapsed, peak_throughput, current_throughput,
                          throughput_15s_ago=None, min_duration=20.0):
    """Check whether the benchmark should stop.

    Two conditions are evaluated (only when ``elapsed >= min_duration``):

    1. **Degradation** — current throughput < 90 % of observed peak.
    2. **Plateau** — throughput has grown by less than 3 % compared to
       the throughput from 15 seconds ago.

    Args:
        elapsed: elapsed time in seconds.
        peak_throughput: highest throughput observed so far.
        current_throughput: throughput at the current tick.
        throughput_15s_ago: throughput from 15 seconds ago (None if
            not available yet).
        min_duration: minimum run duration in seconds before stop
            conditions are checked.

    Returns:
        (should_stop, reason) where *reason* is ``"plateau"``,
        ``"degradation"``, or ``None``.
    """
    if elapsed < min_duration:
        return False, None

    if peak_throughput > 0 and current_throughput < 0.9 * peak_throughput:
        return True, "degradation"

    if (throughput_15s_ago is not None and throughput_15s_ago > 0
            and current_throughput < 1.03 * throughput_15s_ago):
        return True, "plateau"

    return False, None


def run_ramp_up(callback, *, duration=30, ramp_interval=1.5, window_size=10.0,
                plateau_window=15.0, min_duration=20.0):
    """Run a ramp-up benchmark.

    Args:
        callback: called with (concurrency) for each ramp step. Returns
                  an iterable of completion timestamps (seconds since
                  benchmark start).
        duration: maximum benchmark duration in seconds.
        ramp_interval: seconds between concurrency increases.
        window_size: rolling window for throughput computation in seconds.
        plateau_window: window for plateau comparison in seconds.
        min_duration: minimum run duration before stop conditions are
            checked.

    Returns:
        (final_concurrency, peak_throughput, stop_reason)
    """
    all_ts = []
    concurrency = 1
    peak = 0.0
    elapsed = 0.0
    stop_reason = None
    last_tick = 0

    while elapsed < duration and stop_reason is None:
        step_ts = callback(concurrency)
        if step_ts:
            all_ts.extend(step_ts)
        elapsed += ramp_interval

        new_max = int(elapsed)
        for tick in range(last_tick + 1, new_max + 1):
            cutoff = tick - window_size
            count = sum(1 for t in all_ts if t > cutoff)
            throughput = count / window_size
            if throughput > peak:
                peak = throughput

            if tick >= min_duration:
                past_tick = tick - int(plateau_window)
                past_throughput = None
                if past_tick >= 1:
                    past_cutoff = past_tick - window_size
                    past_count = sum(1 for t in all_ts if t > past_cutoff)
                    past_throughput = past_count / window_size

                should_stop, reason = check_stop_conditions(
                    elapsed=float(tick),
                    peak_throughput=peak,
                    current_throughput=throughput,
                    throughput_15s_ago=past_throughput,
                    min_duration=min_duration,
                )
                if should_stop:
                    stop_reason = reason
                    break

        last_tick = new_max
        if stop_reason:
            break
        concurrency += 1

    final_concurrency = concurrency - 1
    return final_concurrency, peak, stop_reason
