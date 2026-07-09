use chrono::TimeDelta;

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
#[derive(Clone)]
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

impl Default for PowerParameters {
    fn default() -> PowerParameters {
        PowerParameters {
            W: 80.0,
            Crr: 0.005,
            Vhw: 0.0,
            A: 0.4,
            Rho: 1.225,
            Cd: 0.9,
            DrivetrainLoss: 2f64,
        }
    }
}

const G: f64 = 9.8067; // m/s^2, matches Gribble's constant
const KMH_TO_MS: f64 = 1000.0 / 3600.0;
const MS_TO_KMH: f64 = 3600.0 / 1000.0;

impl PowerParameters {
    /// Iterates over each segment in `start..end` and calls `f(i, seg_time_seconds)`.
    /// The grade and speed calculations (including v_min clamping) live here,
    /// so callers like `solve_interval` don't duplicate the loop.
    pub fn for_each_segment<F>(
        &self,
        power: f64,
        distance: &impl Fn(usize) -> f64,
        elevation: &impl Fn(usize) -> f64,
        start: usize,
        end: usize,
        mut f: F,
    ) where
        F: FnMut(usize, TimeDelta),
    {
        let v_min = 0.0;
        for i in start..=end {
            if i == 0 {
                continue;
            }
            let ds = distance(i) - distance(i - 1);
            if ds <= 0.0 {
                continue;
            }
            let de = elevation(i) - elevation(i - 1);
            let grade = de / ds * 100.0;
            let v_kmh = self.speed_at_power(power, grade);
            let v_ms = v_kmh.max(v_min) * KMH_TO_MS;
            let seconds = ds / v_ms;
            let duration = TimeDelta::nanoseconds((seconds * 1_000_000_000.0).round() as i64);
            f(i, duration);
        }
    }

    // duration need to ride from distance(start) to distance(end),
    // the elevation at each index is given by elevation(index).
    // returns the duration in seconds.
    // if any segment is impassable (speed <= 0), returns INFINITY.
    pub fn duration_at_power(
        &self,
        power: f64,
        distance: &impl Fn(usize) -> f64,
        elevation: &impl Fn(usize) -> f64,
        start: usize,
        end: usize,
    ) -> TimeDelta {
        let mut total = TimeDelta::seconds(0);
        self.for_each_segment(power, distance, elevation, start, end, |_, duration| {
            total += duration;
        });
        total
    }

    pub fn power_at_duration(
        &self,
        duration: &TimeDelta,
        distance: impl Fn(usize) -> f64,
        elevation: impl Fn(usize) -> f64,
        start: usize,
        end: usize,
    ) -> f64 {
        if *duration <= TimeDelta::seconds(0) {
            return 0.0;
        }

        let mut power_high = 1.0;
        while self.duration_at_power(power_high, &distance, &elevation, start, end) > *duration {
            power_high *= 2.0;
            if power_high > 1_000_000.0 {
                return f64::INFINITY;
            }
        }

        let mut power_low = 0.0;
        for _ in 0..60 {
            let mid = (power_low + power_high) / 2.0;
            let dur = self.duration_at_power(mid, &distance, &elevation, start, end);
            if dur > *duration {
                power_low = mid;
            } else {
                power_high = mid;
            }
        }

        (power_low + power_high) / 2.0
    }

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

    fn assert_close_duration(actual: TimeDelta, expected: TimeDelta, tol: f64) {
        assert!(
            (actual.num_milliseconds() as f64 / 1000f64
                - expected.num_milliseconds() as f64 / 1000f64)
                .abs()
                < tol,
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
    fn duration_on_flat() {
        let p = params(|_| {});
        // distance: 3 points at 0, 500, 1000 metres
        let dist = |i: usize| -> f64 { [0.0, 500.0, 1000.0][i] };
        let elev = |_: usize| -> f64 { 100.0 };
        // 200 W on flat -> 32.4 km/h ~= 9.0 m/s -> 1000/9.0 ~= 111.11 s
        let dur = p.duration_at_power(200.0, &dist, &elev, 0, 2);
        assert_close_duration(dur, TimeDelta::milliseconds(111_110), 0.1);
    }

    #[test]
    fn duration_matches_speed_times_distance() {
        let _ = env_logger::try_init();
        let p = params(|_| {});
        // 10 equally spaced points over 10 km, flat
        let n = 10;
        let dist = |i: usize| -> f64 { i as f64 * 1000.0 };
        let elev = |_: usize| -> f64 { 100.0 };
        let dur = p.duration_at_power(200.0, &dist, &elev, 0, n);
        // expected = total distance / actual speed
        let v_ms = p.speed_at_power(200.0, 0.0) * KMH_TO_MS;
        let expected = 10000.0 / v_ms;
        assert_close_duration(
            dur,
            TimeDelta::milliseconds((expected * 1000f64).round() as i64),
            0.01,
        );
    }

    #[test]
    fn duration_uphill() {
        let _ = env_logger::try_init();
        let p = params(|_| {});
        // 1 km at 5% grade: elevation goes from 0 to 50
        let dist = |i: usize| -> f64 { [0.0, 1000.0][i] };
        let elev = |i: usize| -> f64 { [0.0, 50.0][i] };
        let dur = p.duration_at_power(200.0, &dist, &elev, 0, 1);
        // speed_at_power(200, 5) ≈ 13.47 km/h (from gribble model)
        let v_ms = p.speed_at_power(200.0, 5.0) * KMH_TO_MS;
        let expected = 1000.0 / v_ms;
        assert_close_duration(
            dur,
            TimeDelta::milliseconds((expected * 1000f64).round() as i64),
            0.1,
        );
    }

    #[test]
    fn power_at_duration_round_trip() {
        let p = params(|_| {});
        let dist = |i: usize| -> f64 { i as f64 * 1000.0 };
        let elev = |_: usize| -> f64 { 100.0 };
        let n = 10;
        for power in [50.0, 100.0, 200.0, 300.0] {
            let seconds = p.duration_at_power(power, &dist, &elev, 0, n);
            let power_back = p.power_at_duration(&seconds, dist, elev, 0, n);
            assert_close(power_back, power, 0.5);
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
