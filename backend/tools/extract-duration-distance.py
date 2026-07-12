#!/usr/bin/env python3
import sys
import os
import xml.etree.ElementTree as ET
import math
from datetime import datetime
import subprocess


def haversine(lat1, lon1, lat2, lon2):
    R = 6371000
    phi1 = math.radians(lat1)
    phi2 = math.radians(lat2)
    dphi = math.radians(lat2 - lat1)
    dlambda = math.radians(lon2 - lon1)
    a = math.sin(dphi / 2) ** 2 + math.cos(phi1) * math.cos(phi2) * math.sin(dlambda / 2) ** 2
    return R * 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))


def local_tag(elem):
    return elem.tag.split("}")[1] if "}" in elem.tag else elem.tag


def parse_gpx(filepath):
    tree = ET.parse(filepath)
    root = tree.getroot()
    points = []
    for trk in root.iter():
        if local_tag(trk) != "trk":
            continue
        for seg in trk.iter():
            if local_tag(seg) != "trkseg":
                continue
            for pt in seg.iter():
                if local_tag(pt) != "trkpt":
                    continue
                lat = float(pt.attrib["lat"])
                lon = float(pt.attrib["lon"])
                for child in pt:
                    if local_tag(child) == "time" and child.text:
                        t = datetime.fromisoformat(child.text.replace("Z", "+00:00"))
                        points.append((t, lat, lon))
                        break
    return points


def compute_duration_distance(points):
    if not points:
        return []
    start_time = points[0][0]
    result = []
    cum_dist = 0.0
    prev_lat, prev_lon = points[0][1], points[0][2]
    for i, (t, lat, lon) in enumerate(points):
        duration = (t - start_time).total_seconds()
        if i > 0:
            cum_dist += haversine(prev_lat, prev_lon, lat, lon)
        result.append((duration, cum_dist))
        prev_lat, prev_lon = lat, lon
    return result


def format_duration(seconds):
    hours = int(seconds // 3600)
    minutes = int((seconds % 3600) // 60)
    secs = int(seconds % 60)
    return f"{hours:02d}:{minutes:02d}:{secs:02d}"


def main():
    if len(sys.argv) < 2:
        print("Usage: extract-duration-distance.py file1.gpx [file2.gpx ...]", file=sys.stderr)
        sys.exit(1)

    outdir = "/tmp/duration-distance"
    os.makedirs(outdir, exist_ok=True)
    csv_path = os.path.join(outdir, "duration-distance.csv")

    all_series = []
    labels = []
    for filepath in sys.argv[1:]:
        points = parse_gpx(filepath)
        if not points:
            print(f"error: no track points with timestamps found in {filepath}", file=sys.stderr)
            sys.exit(1)
        series = compute_duration_distance(points)
        all_series.append(series)
        basename = os.path.splitext(os.path.basename(filepath))[0]
        labels.append(basename)

    all_times = sorted({t for series in all_series for t, _ in series})
    if len(all_times) < 2:
        print("error: need at least 2 data points with distinct timestamps to plot", file=sys.stderr)
        sys.exit(1)

    with open(csv_path, "w") as f:
        header = "time" + "".join(f"|{label}" for label in labels)
        f.write(header + "\n")
        for t in all_times:
            row = [format_duration(t)]
            for series in all_series:
                dists = [d for tt, d in series if tt == t]
                if dists:
                    row.append(f"{dists[0]:.1f}")
                else:
                    row.append("")
            f.write("|".join(row) + "\n")

    script_dir = os.path.dirname(os.path.abspath(__file__))
    gnuplot_script = os.path.join(script_dir, "duration-distance.gnuplot")
    subprocess.run(["gnuplot", gnuplot_script], check=True)


if __name__ == "__main__":
    main()
