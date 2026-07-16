#![allow(non_snake_case)]

use std::io::Write;

use clap::Parser;
use tracks::geometry::power::PowerModel;
use tracks::parameters::PowerParameters;

#[derive(Parser)]
struct Cli {
    #[arg(value_name = "CSV")]
    filename: std::path::PathBuf,

    #[arg(short, long, value_name = "output", default_value = "estimated_power.csv")]
    output: std::path::PathBuf,

    #[arg(long, value_name = "weight", default_value_t = 80.0)]
    weight: f64,

    #[arg(long, value_name = "headwind", default_value_t = 0.0)]
    headwind: f64,

    #[arg(long, value_name = "cd", default_value_t = 0.9)]
    cd: f64,

    #[arg(long, value_name = "crr", default_value_t = 0.005)]
    crr: f64,

    #[arg(long, value_name = "area", default_value_t = 0.4)]
    area: f64,

    #[arg(long, value_name = "rho", default_value_t = 1.225)]
    rho: f64,

    #[arg(long, value_name = "drivetrain_loss", default_value_t = 2.0)]
    drivetrain_loss: f64,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let model = PowerModel {
        parameters: PowerParameters {
            W: args.weight,
            Crr: args.crr,
            Vhw: args.headwind,
            A: args.area,
            Rho: args.rho,
            Cd: args.cd,
            DrivetrainLoss: args.drivetrain_loss,
        },
    };

    let input = std::fs::read_to_string(&args.filename)?;
    let mut lines = input.lines();

    let header = lines.next().unwrap_or("");
    let mut out = std::fs::File::create(&args.output)?;
    writeln!(out, "{},estimated_power", header)?;

    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split(',').collect();
        let smooth_speed: f64 = fields.get(4).unwrap_or(&"0").parse().unwrap_or(0.0);
        let slope: f64 = fields.get(5).unwrap_or(&"0").parse().unwrap_or(0.0);
        let estimated_power = model.power_at_speed(smooth_speed, slope);
        writeln!(out, "{},{:.1}", line, estimated_power)?;
    }

    println!("wrote: {}", args.output.display());

    Ok(())
}
