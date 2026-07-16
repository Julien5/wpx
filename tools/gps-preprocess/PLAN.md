# GPS Preprocess — Implementation Plan

## Project structure

```
gps-preprocess/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI (clap), pipeline orchestration
│   ├── data.rs          # Input, Smooth structs + haversine + interpolate_nans
│   ├── read.rs          # read_csv
│   ├── oversample.rs    # oversample
│   ├── filter.rs        # filter_outliers
│   ├── smooth.rs        # smooth_speed, slope (centered windows)
│   └── output.rs        # write processing.csv + 5 gnuplot scripts
└── gnuplot/             # generated scripts directory
```

## Dependencies

- `clap` — CLI argument parsing
- `csv` — reading/writing CSV
- `serde` + `serde_derive` — optional CSV deserialization

## Pipeline

```
CLI args → read_csv → oversample → filter_outliers → smooth_speed + slope → write CSV + gnuplot
```

## Structs (`data.rs`)

```rust
struct Input {
    time: f64,           // seconds from start
    distance: f64,       // meters
    elevation: f64,      // meters
    speed: f64,          // m/s
    vertical_speed: f64, // m/s
}

struct Smooth {
    time: f64,
    distance: f64,
    elevation: f64,
    speed: f64,
    smooth_speed: f64,
    vertical_speed: f64,
    slope: f64,
}
```

`data.rs` also contains:
- `haversine(lat1, lon1, lat2, lon2) -> f64` — distance in meters
- `interpolate_nans(data: &mut [Input], field: fn(&mut Input) -> &mut f64)` — linear interpolation of NaN runs

## Step 1: `read.rs` — `read_csv(path: &str) -> Vec<Input>`

- Parse CSV with `csv` crate
- Filter rows where time delta == 0 (skip duplicate timestamps)
- Accumulate Haversine distance
- Compute speed = Δdistance / Δtime
- Compute vertical_speed = Δelevation / Δtime

## Step 2: `oversample.rs` — `oversample(input: &[Input]) -> Vec<Input>`

- Determine time range: `t_start` to `t_end` (ceil/floor)
- Create vector with one entry per second
  - If input point exists at that second → copy
  - Otherwise → copy time, rest fields = NaN
- Second pass: interpolate each NaN field using `interpolate_nans`

## Step 3: `filter.rs` — `filter_outliers(input: &[Input], max_speed: f64, max_vert_speed: f64) -> Vec<Input>`

- First pass: set fields (except time) to NaN where:
  - `speed > max_speed` (convert max_speed from km/h to m/s: `/ 3.6`)
  - `vertical_speed > max_vert_speed`
- Second pass: interpolate each NaN field using `interpolate_nans`

## Step 4: `smooth.rs`

### `smooth_speed(input: &[Input], window_sec: f64) -> Vec<f64>`

For each index `i`:
- `half = (window_sec / 2.0) as usize`
- `start = i.saturating_sub(half)`, `end = min(i + half, len - 1)`
- Average `input[j].speed` for `j in start..=end`

### `slope(input: &[Input], window_sec: f64) -> Vec<f64>`

For each index `i`:
- Gather points in centered window (same as above)
- Linear regression of `elevation ~ distance`:
  - `slope = (n * Σ(d*e) - Σd * Σe) / (n * Σ(d²) - (Σd)²)`
  - If denominator is 0 (all same distance), slope = 0

## Step 5: `output.rs`

### `write_csv(smooth: &[Smooth], raw: &[Input], oversampled: &[Input], filtered: &[Input], path: &str)`

Writes `processing.csv` with 10 columns:

| Col | Name | Unit | Source |
|-----|------|------|--------|
| 1 | time | hours (≥4 decimals) | filtered data time / 3600 |
| 2 | distance | km | filtered distance / 1000 |
| 3 | elevation | m | filtered elevation |
| 4 | speed | km/h | filtered speed × 3.6 |
| 5 | smooth_speed | km/h | smooth_speed × 3.6 |
| 6 | slope | % | slope × 100 |
| 7 | raw_distance | km | raw (NaN where no raw point) |
| 8 | raw_elevation | m | raw (NaN where no raw point) |
| 9 | oversample_distance | km | oversampled distance / 1000 |
| 10 | oversample_elevation | m | oversampled elevation |

For cols 7–10, align to 1Hz timeline by matching time (within 0.5s tolerance), else write NaN.

### `write_gnuplot(dir: &str)`

Writes 5 gnuplot scripts to `gnuplot/`:

| Script | Layout | Col mapping | Line styles |
|--------|--------|-------------|-------------|
| `oversample.gnuplot` | 1×2: dist, elev | 7,8 (input), 9,10 (output) | thin line → red dots |
| `filter_outliers.gnuplot` | 1×2: dist, elev | 9,10 (input), 2,3 (output) | blue thick → red thin |
| `smooth_speed.gnuplot` | 1×1: speed | 4 (input), 5 (output) | blue thick → red thin |
| `slope.gnuplot` | 1×3: dist, elev, slope | 2,3 (input), 6 (output) | blue thick → red thin |
| `output.gnuplot` | 1×2: smooth_speed, slope | 5,6 | blue thin |

All scripts accept xrange via `gnuplot -e "xrange='[a:b]'"`.

## CLI (`main.rs`)

```
gps-preprocess --input <path> [options]

Options:
  --input <file>                 Input CSV (required)
  --max-speed <kmh>              Default: 60
  --max-elevation-speed <mps>    Default: 3
  --speed-smooth-window <sec>    Default: 10
  --slope-reg-window <sec>       Default: 50
  --output <path>                Default: processing.csv
  --gnuplot-dir <dir>            Default: gnuplot
```

## Edge cases

- **Interpolation**: leading/trailing NaN runs stay NaN; all-NaN window stays NaN
- **Window boundaries**: clamp to valid index range (saturating_sub, min)
- **Slope regression**: if all x (distance) values equal → slope = 0
- **Zero time delta**: rows with zero time delta from previous are discarded in read step
- **Oversample alignment**: raw data aligned to 1Hz grid by nearest second match
