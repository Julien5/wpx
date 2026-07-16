mod data;
mod filter;
mod output;
mod oversample;
mod read;
mod smooth;

use clap::Parser;

#[derive(Parser)]
#[command(name = "gps-preprocess")]
struct Args {
    #[arg(long)]
    input: String,

    #[arg(long, default_value = "60")]
    max_speed: f64,

    #[arg(long, default_value = "3")]
    max_elevation_speed: f64,

    #[arg(long, default_value = "10")]
    speed_smooth_window: f64,

    #[arg(long, default_value = "50")]
    slope_reg_window: f64,

    #[arg(long, default_value = "processing.csv")]
    output: String,

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let raw = read::read_csv(&args.input)?;
    eprintln!("Read {} points from {}", raw.len(), args.input);

    let oversampled = oversample::oversample(&raw);
    eprintln!("Oversampled to {} points (1 Hz)", oversampled.len());

    let filtered = filter::filter_outliers(&oversampled, args.max_speed, args.max_elevation_speed);
    eprintln!(
        "Filtered outliers (max_speed={} km/h, max_vert_speed={} m/s)",
        args.max_speed, args.max_elevation_speed
    );

    let smooth_speeds = smooth::smooth_speed(&filtered, args.speed_smooth_window);
    let slopes = smooth::slope(&filtered, args.slope_reg_window);

    let smooth_data: Vec<data::Smooth> = filtered
        .iter()
        .zip(smooth_speeds.iter())
        .zip(slopes.iter())
        .map(|((input, ss), sl)| data::Smooth {
            time: input.time,
            distance: input.distance,
            elevation: input.elevation,
            speed: input.speed,
            smooth_speed: *ss,
            vertical_speed: input.vertical_speed,
            slope: *sl,
            measured_power: input.measured_power,
        })
        .collect();

    output::write_csv(&smooth_data, &raw, &oversampled, &args.output)?;
    eprintln!("Wrote {}", args.output);

    Ok(())
}
