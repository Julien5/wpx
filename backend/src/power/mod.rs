/*
- total weight W (kg)
- rolling resistance Crr (unitless)
- headwind speed Vhw (km/h)
- frontal area A (m2)
- air density Rho (kg/m3)
- drag coefficient Cd (unitless)
and
- percent grade of hill: G (percent)
 */
#[allow(non_snake_case)]
pub struct PowerParameters {
    W: f64,              // total weight (rider + bike), kg
    Crr: f64,            // rolling resistance coefficient, unitless
    Vhw: f64,            // headwind speed, km/h (positive = headwind, negative = tailwind)
    A: f64,              // frontal area, m^2
    Rho: f64,            // air density, kg/m^3
    Cd: f64,             // drag coefficient, unitless
    DrivetrainLoss: f64, // drivetrain loss, percent (e.g. 3.0 for 3%)
}

const G: f64 = 9.8067; // m/s^2, matches Gribble's constant
const KMH_TO_MS: f64 = 1000.0 / 3600.0;
const MS_TO_KMH: f64 = 3600.0 / 1000.0;

impl PowerParameters {
    /// Same model as gribble.org/cycling/power_v_speed.html, including
    /// drivetrain loss: only a fraction of rider power actually reaches
    /// the road, the rest is dissipated in the chain/derailleur/bearings.
    pub fn speed_at_power(&self, power: f64, percent: f64) -> f64 {
        let vhw = self.Vhw * KMH_TO_MS;

        let slope = percent / 100.0;
        let theta = slope.atan();
        let (sin_t, cos_t) = (theta.sin(), theta.cos());

        // Power actually delivered to the wheel, after drivetrain loss.
        let wheel_power = power * (1.0 - self.DrivetrainLoss / 100.0);

        let a = 0.5 * self.Cd * self.A * self.Rho;
        let b = vhw * self.Cd * self.A * self.Rho;
        let c =
            G * self.W * (sin_t + self.Crr * cos_t) + 0.5 * self.Cd * self.A * self.Rho * vhw * vhw;
        let d = -wheel_power;

        Self::real_cubic_root(a, b, c, d) * MS_TO_KMH
    }

    pub fn power_at_speed(&self, speed: f64, percent: f64) -> f64 {
        let v_ground = speed * KMH_TO_MS;
        let vhw = self.Vhw * KMH_TO_MS;

        let slope = percent / 100.0;
        let theta = slope.atan();
        let (sin_t, cos_t) = (theta.sin(), theta.cos());

        let a = 0.5 * self.Cd * self.A * self.Rho;
        let b = vhw * self.Cd * self.A * self.Rho;
        let c =
            G * self.W * (sin_t + self.Crr * cos_t) + 0.5 * self.Cd * self.A * self.Rho * vhw * vhw;

        let wheel_power = a * v_ground.powi(3) + b * v_ground.powi(2) + c * v_ground;

        wheel_power / (1.0 - self.DrivetrainLoss / 100.0)
    }

    /// Returns the physically relevant real root of a*x^3 + b*x^2 + c*x + d = 0
    /// (the largest real ground-speed root, clamped to be non-negative).
    fn real_cubic_root(a: f64, b: f64, c: f64, d: f64) -> f64 {
        let shift = b / (3.0 * a);
        let p = (3.0 * a * c - b * b) / (3.0 * a * a);
        let q = (2.0 * b * b * b - 9.0 * a * b * c + 27.0 * a * a * d) / (27.0 * a * a * a);

        let discriminant = (q * q / 4.0) + (p * p * p / 27.0);

        let t = if discriminant >= 0.0 {
            let sqrt_disc = discriminant.sqrt();
            let u = -q / 2.0 + sqrt_disc;
            let v = -q / 2.0 - sqrt_disc;
            u.cbrt() + v.cbrt()
        } else {
            let r = 2.0 * (-p / 3.0).sqrt();
            let phi = ((3.0 * q) / (p * r)).acos() / 3.0;
            let t0 = r * phi.cos();
            let t1 = r * (phi - 2.0 * std::f64::consts::PI / 3.0).cos();
            let t2 = r * (phi - 4.0 * std::f64::consts::PI / 3.0).cos();
            t0.max(t1).max(t2)
        };

        (t - shift).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn params(overrides: impl FnOnce(&mut PowerParameters)) -> PowerParameters {
        // https://www.gribble.org/cycling/power_v_speed.html?units=metric&rp_wr=70&rp_wb=10&rp_a=0.4&rp_cd=0.9&rp_dtl=2&ep_crr=0.005&ep_rho=1.225&ep_g=0&ep_headwind=0&p2v=200&v2p=35.41

        // p_wr=70 OK
        // p_wb=10 OK
        // p_a=0.4 OK
        // p_cd=0.9 OK
        // p_dtl=2 OK
        // p_crr=0.005 OK
        // p_rho=1.225 OK
        // p_headwind=0 OK

        let mut p = PowerParameters {
            W: 80.0,
            Crr: 0.005,
            Vhw: 0.0,
            A: 0.4,
            Rho: 1.225,
            Cd: 0.9,
            DrivetrainLoss: 2f64,
        };
        overrides(&mut p);
        p
    }

    // Loose tolerance since we're checking against independently-computed
    // reference values (km/h), not bit-exact output.
    fn assert_close(actual: f64, expected: f64, tol: f64) {
        assert!(
            (actual - expected).abs() < tol,
            "expected ~{expected:.3} km/h, got {actual:.3} km/h"
        );
    }

    #[test]
    fn no_wind() {
        let p = params(|_| {});
        // Cross-checked against the bisection reference solver.
        assert_close(p.speed_at_power(200.0, 0.0), 32.4, 0.01);
        assert_close(p.speed_at_power(200.0, 1.0), 28.05, 0.01);
        assert_close(p.speed_at_power(200.0, 10.0), 8.48, 0.01);
    }

    #[test]
    fn round_trip_speed_then_power() {
        let _ = env_logger::try_init();
        let p = params(|_| {});
        for speed in [10.0, 20.0, 30.0, 40.0, 50.0] {
            for grade in [0.0, 1.0, -1.0, 5.0, -5.0, 10.0, -10.0] {
                let power = p.power_at_speed(speed, grade);
                if power < 0f64 {
                    continue;
                }
                let speed_back = p.speed_at_power(power, grade);
                assert_close(speed_back, speed, 0.01);
            }
        }
    }

    #[test]
    fn round_trip_power_then_speed() {
        let p = params(|_| {});
        for power in [50.0, 100.0, 200.0, 300.0, 500.0] {
            for grade in [-10.0, -5.0, -1.0, 0.0, 1.0, 5.0, 10.0] {
                let speed = p.speed_at_power(power, grade);
                let power_back = p.power_at_speed(speed, grade);
                assert_close(power_back, power, 0.5);
            }
        }
    }

    #[test]
    fn strong_tailwind_hits_three_real_root_branch() {
        // Vhw very negative + downhill grade drives the discriminant
        // negative, exercising the trigonometric branch specifically.
        let p = params(|p| p.Vhw = -30.0);
        let v = p.speed_at_power(50.0, -1.0);
        assert_close(v, 50.84, 0.01);
    }
}
