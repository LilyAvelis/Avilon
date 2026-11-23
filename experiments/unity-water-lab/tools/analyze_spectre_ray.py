"""Summarize finite ray prefixes written by SpectreRayExperiment. Standard library only."""

import argparse
import csv
import math
import statistics
from collections import Counter, defaultdict
from pathlib import Path


def period(values, tolerance):
    """Look for a period shared by the entire prefix, with at least two cycles."""
    return next(
        (lag for lag in range(1, len(values) // 2 + 1)
         if all(abs(a - b) <= tolerance for a, b in zip(values, values[lag:]))),
        None,
    )


def correlation(values, lag):
    left, right = values[:-lag], values[lag:]
    if len(left) < 3 or min(len(set(left)), len(set(right))) < 2:
        return float("nan")
    return statistics.correlation(left, right)


def summarize(rows):
    values = []
    previous = rows[0]
    scale = float(previous["tile_scale"])
    for row in rows[1:]:
        interval = float(row["interval"])
        gap = float(row["distance"]) - float(previous["distance"])
        if not interval > 0 or not math.isclose(interval, gap, abs_tol=1e-12):
            raise ValueError("Invalid distance interval in " + row["run"])
        if previous["to_tile"] != row["from_tile"] or row["from_tile"] == row["to_tile"]:
            raise ValueError("Broken tile sequence in " + row["run"])
        values.append(interval / scale)
        previous = row

    # Read one decimal digit from a distance measured in prototile edges.
    digits = [math.floor(value * 10) % 10 for value in values]
    counts = Counter(digits)
    hundredths = [round(value * 100) for value in values]
    motifs = Counter(zip(hundredths, hundredths[1:], hundredths[2:], hundredths[3:]))
    digit_motifs = Counter(zip(digits, digits[1:], digits[2:], digits[3:]))
    autocorrelations = {
        lag: correlation(values, lag) for lag in range(1, min(32, len(values) // 3) + 1)
    }
    peak = max(autocorrelations, key=lambda lag: abs(autocorrelations[lag]))
    return {
        "run": rows[0]["run"],
        "angle_deg": float(rows[0]["angle_deg"]),
        "intervals": len(values),
        "interval_min": min(values),
        "interval_median": statistics.median(values),
        "interval_max": max(values),
        "period_e1e-8": period(values, 1e-8),
        "period_e1e-2": period(values, 1e-2),
        "corr_lag1": autocorrelations[1],
        "peak_corr_lag": peak,
        "peak_corr": autocorrelations[peak],
        "raw_repeated_4_motifs_at_0.01": sum(count - 1 for count in motifs.values()),
        "digit_repeated_4_motifs": sum(count - 1 for count in digit_motifs.values()),
        "digit_period": period(digits, 0),
        "digit_corr_lag1": correlation(digits, 1),
        "digit_min_count": min(counts[i] for i in range(10)),
        "digit_max_count": max(counts.values()),
        "digits_0_to_9": " ".join(str(counts[i]) for i in range(10)),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output", type=Path, help="Directory containing crossings.csv")
    output = parser.parse_args().output
    runs = defaultdict(list)
    with (output / "crossings.csv").open(newline="", encoding="utf-8") as file:
        for row in csv.DictReader(file):
            runs[row["run"]].append(row)
    repeated = [{key: value for key, value in row.items() if key != "run"} for row in runs.pop("water-repeat")]
    original = [{key: value for key, value in row.items() if key != "run"} for row in runs["water"]]
    if not repeated or original != repeated:
        raise ValueError("The same seed and configuration did not reproduce the same crossings")
    summary = [summarize(rows) for rows in runs.values()]
    with (output / "statistics.csv").open("w", newline="", encoding="utf-8") as file:
        writer = csv.DictWriter(file, fieldnames=list(summary[0]))
        writer.writeheader()
        writer.writerows(summary)
    axis, perturbed = runs["angle-30"], runs["angle-30-near"]
    changed = len(axis) != len(perturbed) or any(a["edge"] != b["edge"] for a, b in zip(axis, perturbed))
    max_delta = max(abs(float(a["interval"]) - float(b["interval"])) for a, b in zip(axis[1:], perturbed[1:]))
    print(f"Same seed reproduced {len(original)} crossings. Summarized {len(summary)} rays.")
    print(f"30° + 0.00001°: changed edge sequence {changed}, maximum paired interval difference {max_delta:.6g} m.")


if __name__ == "__main__":
    main()
