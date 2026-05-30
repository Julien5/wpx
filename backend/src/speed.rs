use std::cmp::Ordering;

use chrono::TimeDelta;

use crate::mercator::DateTime;

// from mps to kmh
pub fn _kmh(_mps: f64) -> f64 {
    // m/s => kmh
    _mps * 3.6f64
}

// from kmh to mps
pub fn mps(_kmh: f64) -> f64 {
    _kmh / 3.6f64
}

const ACP_MAX_DISTANCE: f64 = 1250_000f64;
const LRM_MIN_DISTANCE: f64 = 1050_000f64;

#[allow(non_snake_case)]
mod LRM {
    use super::time_delta;
    use crate::{
        mercator::DateTime,
        speed::{ControlSpeedData, LRMSpec, LRM_MIN_DISTANCE},
    };

    fn speed_kmh(end_distance: f64) -> f64 {
        debug_assert!(end_distance >= LRM_MIN_DISTANCE);
        let distance_km = end_distance / 1000.0;
        let kmh = if distance_km >= 2500f64 {
            200.0 / 24.0
        } else if distance_km >= 1900f64 {
            10.0
        } else {
            12.0
        };
        kmh
    }

    pub fn guess_spec(end_distance: f64) -> Option<LRMSpec> {
        if end_distance < LRM_MIN_DISTANCE {
            return None;
        }
        Some(LRMSpec {
            kmh: speed_kmh(end_distance),
        })
    }

    pub fn interpolation_points(
        end_distance: f64,
        start_time: &DateTime,
        spec: &LRMSpec,
    ) -> Vec<ControlSpeedData> {
        let mps = spec.kmh * 1000.0 / 3600.0;
        let mut all = Vec::new();
        all.push(ControlSpeedData {
            distance: 0f64,
            time: Some(start_time.clone()),
            is_end: false,
        });
        let end_duration = time_delta(end_distance / mps);
        all.push(ControlSpeedData {
            distance: end_distance,
            time: Some(*start_time + end_duration),
            is_end: true,
        });
        all.sort();
        debug_assert!(all.len() >= 2);
        all.iter().for_each(|c| debug_assert!(c.time.is_some()));
        all
    }
}

/*

[0] https://www.audax-club-parisien.com/wp-content/uploads/2024/01/Rules-for-rider-2024.pdf
Covers 200-1000 km
- 13:30 for 200 KM,
- 20:00 for 300 KM,
- 27:00 for 400 KM,
- 40:00 for 600 KM, and
- 75:00 for 1000 KM.
Intermediate control times are an advisory to help keep the rider inside the final time limit.
Closing:
- 1 hour + 20 km / h (km 1 to 60);
- 15 km / h (km 61 to 600);
- 11.428 km / h (km 601 to 1000);

[1] https://www.randonneursmondiaux.org/12-Rules.html?langue=EN
The allocated time for RM sanctioned events of 1200 km is ninety (90) hours.
For events of 1400 km and greater the total time will be based on
an average global speed of twelve (12) km per hour.

[2] https://www.randonneursmondiaux.org/files/Rules_2019.pdf
- from 1200 to 1299 km: 13.33 kph
- from 1300 to 1899 km: 12 kph
- from 1900 to 2499 km: 10 kph
- 2500 km and above: 200 km per day

[3] https://en.wikipedia.org/wiki/Randonneuring#Time_limits => 10kmh at 2200km
There is some regional variation in these, but the following list is typical:
1,400 kilometres (870 mi) – 116:40 hours (12 km/h)
2,200 kilometres (1,400 mi) – 220 hours (10 km/h)

https://docs.google.com/spreadsheets/u/0/d/e/2PACX-1vRU8adejamxip0ue6pMMGgRjPDNrboJp6SWYlf_k7HmhLyXSjEIMqOetBS5MSiRHZ96r9K7nzgtU9uc/pubhtml?gid=1480200001&single=true&pli=1
 */

#[allow(non_snake_case)]
mod ACP {
    use super::time_delta;
    use super::ControlSpeedData;
    use super::ACP_MAX_DISTANCE;
    use crate::mercator::DateTime;
    use crate::speed::ACPSpec;
    use chrono::TimeDelta;

    fn all_brevets_points() -> Vec<(f64, f64)> {
        vec![
            (200.0, 13.5),  // 14.81 kmh
            (300.0, 20.0),  // 15.00 kmh
            (400.0, 27.0),  // 14.81 kmh
            (600.0, 40.0),  // 15.00 kmh
            (1000.0, 75.0), // 13.333 kmh
            (1200.0, 90.0), // 13.333 kmh
        ]
    }

    fn fixed_interpolation_points() -> Vec<(f64, f64)> {
        vec![
            (0.0, 1.0),
            (60.0, 4.0),
            (600.0, 40.0),  // 15.00 kmh
            (1000.0, 75.0), // 13.333 kmh
            (1200.0, 90.0), // 13.333 kmh
        ]
    }

    pub fn guess_spec(end_distance: f64) -> Option<ACPSpec> {
        let distance_km = end_distance / 1000.0;
        let acp_points = all_brevets_points();
        let _max_acp = acp_points.last().unwrap();
        let closest = acp_points.iter().copied().min_by(|a, b| {
            (a.0 - distance_km)
                .abs()
                .partial_cmp(&(b.0 - distance_km).abs())
                .unwrap()
        });

        match closest {
            Some((km, hours)) => {
                let error_km = (km - distance_km).abs();
                if error_km > 50f64 {
                    return None;
                }
                return Some(ACPSpec { km, hours });
            }
            _ => {
                panic!("could not find ACP distance for {}", end_distance);
            }
        }
    }

    // ACP (Audax Club Parisien) control closing time rules:
    // Staggered minimum speeds based on distance segments:
    //   -    0- 600 km: 15.0 km/h
    //   -  600-1000 km: 11.428 km/h (8/7 km/h)
    //   - 1000-1300 km: 13.333 km/h (40/3 km/h)
    // Special case for short distances (0-60 km): T = 1 + (D / 20)
    // Note: function used only for the pre-last control.
    fn duration(distance: f64) -> TimeDelta {
        debug_assert!(distance <= ACP_MAX_DISTANCE);
        let distance_km = distance / 1000.0;
        let time_hours = {
            let mut remain_km = distance_km;
            let mut prev = (0.0, 0.0);
            let mut time = 0f64;
            for (km, hours) in fixed_interpolation_points() {
                let (delta_km, delta_hours) = (km - prev.0, hours - prev.1);
                if remain_km >= delta_km {
                    time += delta_hours;
                    remain_km -= delta_km;
                } else {
                    let speed = delta_km / delta_hours;
                    time += remain_km / speed;
                    remain_km = 0f64;
                    break;
                }
                prev = (km, hours);
            }
            // may happen if pre-last control is > 1200km
            // example: PBP with 1230 km and one control at 1215km.
            if remain_km > 0.0 {
                // 200 km per day.
                let days = distance_km / 200.0;
                let (delta_km, delta_hours) = (distance_km - prev.0, 24.0 * days - prev.1);
                let speed = delta_km / delta_hours;
                time += remain_km / speed;
                remain_km = 0f64;
            }
            debug_assert_eq!(remain_km, 0f64);
            time
        };
        time_delta(time_hours * 3600.0)
    }

    fn fixed_interpolation_controls(start: &DateTime) -> Vec<ControlSpeedData> {
        let mut ret = Vec::new();
        let acp_points = fixed_interpolation_points();
        for (km, hours) in acp_points.iter() {
            ret.push(ControlSpeedData {
                distance: km * 1000f64,
                time: Some(*start + TimeDelta::seconds((hours * 3600f64).round() as i64)),
                is_end: false,
            });
        }
        ret
    }

    pub fn interpolation_controls(
        start_time: &DateTime,
        end_distance: f64,
        controls: &Vec<ControlSpeedData>,
        spec: &ACPSpec,
    ) -> Vec<ControlSpeedData> {
        let mut ret = Vec::new();
        debug_assert!(end_distance <= ACP_MAX_DISTANCE);
        // the START control
        ret.push(ControlSpeedData {
            distance: 0f64,
            time: Some(*start_time),
            is_end: false,
        });

        let spec_duration = time_delta(spec.hours * 3600.0);
        let spec_distance = spec.km * 1000.0;

        // END
        /* For brevets up to 1200 km, the end control has a
         * fixed ACP time (e.g. 90h, even if distance = 1230km).
         */
        let end = ControlSpeedData {
            distance: end_distance,
            time: Some(*start_time + spec_duration),
            is_end: true,
        };
        ret.push(end);

        // one before END (if there is one)
        let mut copy = controls.clone();
        copy.retain(|c| !c.is_end && c.distance > 0f64);
        copy.sort();

        if !copy.is_empty() {
            let prelastc = copy.last().unwrap();
            // in ACP mode, ignore the time set by user on that control.
            let prelast = ControlSpeedData {
                distance: prelastc.distance,
                time: Some(*start_time + duration(prelastc.distance)),
                is_end: false,
            };
            ret.push(prelast);
        }

        let mut fixed = fixed_interpolation_controls(start_time);
        // exclude the acp point at closest_to_end_acp.
        // The right point for that is END (included above).
        fixed.retain(|c| c.distance < end_distance && c.distance != spec_distance);
        ret.extend_from_slice(&fixed);

        ret.sort();
        ret
    }
}

#[allow(non_snake_case)]
mod MPS {
    use super::time_delta;
    use super::ControlSpeedData;
    use crate::mercator::DateTime;
    pub fn interpolation_points(
        controls: &Vec<ControlSpeedData>,
        end_distance: f64,
        start_time: &DateTime,
        mps: f64,
    ) -> Vec<ControlSpeedData> {
        let mut all = controls.clone();
        all.retain(|c| c.time.is_some());
        // add START if needed
        if all.iter().find(|c| c.distance == 0f64).is_none() {
            all.push(ControlSpeedData {
                distance: 0f64,
                time: Some(start_time.clone()),
                is_end: false,
            });
        }
        // add END if needed
        if all.iter().find(|c| c.is_end).is_none() {
            let end_duration = time_delta(end_distance / mps);
            all.push(ControlSpeedData {
                distance: end_distance,
                time: Some(*start_time + end_duration),
                is_end: true,
            });
        }
        all.sort();
        debug_assert!(all.len() >= 2);
        all.iter().for_each(|c| debug_assert!(c.time.is_some()));
        all
    }
}

fn time_delta(seconds: f64) -> TimeDelta {
    let nano = (seconds * 1_000_000_000f64).round() as i64;
    TimeDelta::nanoseconds(nano)
}

#[derive(Clone, Debug, Default)]
pub struct ControlSpeedData {
    pub distance: f64,
    pub time: Option<DateTime>,
    pub is_end: bool,
}

impl ControlSpeedData {
    pub fn unwrap_time(&self) -> DateTime {
        self.time.unwrap().clone()
    }
}

impl PartialOrd for ControlSpeedData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ControlSpeedData {
    fn cmp(&self, other: &Self) -> Ordering {
        // Compare by distance first (NaN values are sorted last)
        let dist_order = self
            .distance
            .partial_cmp(&other.distance)
            .unwrap_or_else(|| match (self.distance.is_nan(), other.distance.is_nan()) {
                (true, false) => Ordering::Greater,
                (false, true) => Ordering::Less,
                _ => Ordering::Equal,
            });

        if dist_order != Ordering::Equal {
            return dist_order;
        }

        // Distances are equal — break tie by time (None sorts last)
        match (&self.time, &other.time) {
            (Some(a), Some(b)) => a.cmp(b),
            (Some(_), None) => Ordering::Less,
            (None, Some(_)) => Ordering::Greater,
            (None, None) => Ordering::Equal,
        }
    }
}

impl PartialEq for ControlSpeedData {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for ControlSpeedData {}

fn interpolate_time(interpolation_points: &Vec<ControlSpeedData>, distance: f64) -> DateTime {
    // controls must contains START and END.
    debug_assert!(interpolation_points.len() >= 2);
    // controls has to be sorted by distance and time
    debug_assert!(interpolation_points.is_sorted());

    /* There is a somewhat degenerate case in the case of ACP at
     * start. The interpolations points have two elements at d==0:
     * 0h and 1h.
     */
    let mut equals = interpolation_points.clone();
    equals.retain(|c| c.distance == distance);

    let (index_next, next) = {
        if equals.len() > 1 {
            equals.iter().enumerate().last().unwrap()
        } else {
            let larger = interpolation_points
                .iter()
                .enumerate()
                .find(|(_, c)| c.distance >= distance);
            match larger {
                Some((index, point)) => (index, point),
                None => interpolation_points.iter().enumerate().last().unwrap(),
            }
        }
    };
    ////////////////////////////////////////////////////////////////////
    // log::trace!("next: {:?}", next);								  //
    // for c in interpolation_points {								  //
    //     log::trace!("[TP1] inter:{:?},{:.1}", c.time, c.distance); //
    // }															  //
    ////////////////////////////////////////////////////////////////////
    if index_next == 0 {
        return interpolation_points.first().unwrap().unwrap_time();
    }
    let previous_candidate = interpolation_points
        .iter()
        .enumerate()
        .filter(|(index, control)| *index < index_next && control.distance <= next.distance)
        .last();
    let (_, previous) = match previous_candidate {
        Some((index, point)) => (index, point),
        None => (0, interpolation_points.first().unwrap()),
    };

    let (t1, d1) = (previous.unwrap_time(), previous.distance);
    let (t2, d2) = (next.unwrap_time(), next.distance);
    ///////////////////////////////////////////////////
    // log::trace!("[TP1] previous:{:?}", previous); //
    // log::trace!("[TP1] 1:{:?},{:.1}", t1, d1);	 //
    // log::trace!("[TP1] next:{:?}", next);		 //
    // log::trace!("[TP1] 2:{:?},{:.1}", t2, d2);	 //
    ///////////////////////////////////////////////////
    // This is the "somwhat degenerate" case mentioned above.
    if d1 == d2 {
        debug_assert!(d1 == distance);
        // the *cutoff* corresponds to the maximum time for a given distance.
        // => t2.
        return t2;
    }
    debug_assert!(d1 < d2);
    debug_assert!(d1 <= distance && distance <= d2 || d2 < distance);
    let fraction = (distance - d1) / (d2 - d1);
    let span_ns = (t2 - t1)
        .num_nanoseconds()
        .expect("time span overflows i64 nanoseconds");
    let offset_ns = (fraction * span_ns as f64).round() as i64;
    let ret = t1 + TimeDelta::nanoseconds(offset_ns);
    //////////////////////////////////////////////////////////////////
    // log::info!("[TP1] distance:{:.1} time:{:?}", distance, ret); //
    //////////////////////////////////////////////////////////////////
    ret
}

pub fn interpolate_distance(
    interpolation_points: &Vec<ControlSpeedData>,
    start_time: &DateTime,
    duration: &TimeDelta,
) -> f64 {
    let current_time = *start_time + *duration;

    let next_candidate = interpolation_points
        .iter()
        .enumerate()
        .find(|(_, c)| c.unwrap_time() >= current_time);

    let (index_next, next) = match next_candidate {
        Some((index, point)) => (index, point),
        None => interpolation_points.iter().enumerate().last().unwrap(),
    };

    if index_next == 0 {
        //log::trace!("next control is start");
        return 0f64;
    }

    let previous_candidate = interpolation_points
        .iter()
        .enumerate()
        .filter(|(index, control)| {
            *index < index_next && control.unwrap_time() < next.time.unwrap()
        })
        .last();
    let (_, previous) = match previous_candidate {
        Some((index, point)) => (index, point),
        None => (0, interpolation_points.first().unwrap()),
    };

    let (t1, d1) = (previous.unwrap_time(), previous.distance);
    let (t2, d2) = (next.unwrap_time(), next.distance);
    let span_ns = (t2 - t1)
        .num_nanoseconds()
        .expect("time span overflows i64 nanoseconds");
    debug_assert!(t1 < t2);
    debug_assert!(t1 <= current_time && current_time <= t2 || current_time > t2);

    let offset_ns = (current_time - t1)
        .num_nanoseconds()
        .expect("time offset overflows i64 nanoseconds");

    let fraction = offset_ns as f64 / span_ns as f64;
    debug_assert!(fraction >= 0f64);
    let ret = d1 + fraction * (d2 - d1);
    /*log::trace!(
        "[TP2] distance:{:.1} time:{:?}",
        ret / 1000f64,
        current_time
    );*/
    ret
}

#[derive(Clone, Debug)]
pub struct ACPSpec {
    pub km: f64,    // km
    pub hours: f64, // hours
}

#[derive(Clone, Debug)]
pub struct LRMSpec {
    pub kmh: f64, // kmh
}

#[derive(Clone, Debug)]
pub struct KMHSpec {
    pub kmh: f64, // kmh
}

#[derive(Clone, Debug)]
pub enum Speed {
    KMH(KMHSpec),
    ACP(ACPSpec),
    LRM(LRMSpec),
}

impl Default for Speed {
    fn default() -> Self {
        Speed::KMH(KMHSpec { kmh: 15.0 })
    }
}

pub fn parse_speed(data: &str) -> Speed {
    if data.contains("ACP") {
        let parts: Vec<&str> = data.split('-').collect();
        if parts.len() != 3 {
            log::trace!("parts:{:?}", parts);
        }
        debug_assert!(parts.len() == 3);
        let distance: f64 = parts[1].parse().expect("Failed to parse distance");
        let time: f64 = parts[2].parse().expect("Failed to parse hours");
        return Speed::ACP(ACPSpec {
            km: distance,
            hours: time,
        });
    }
    if data.contains("LRM") {
        let parts: Vec<&str> = data.split('-').collect();
        if parts.len() != 2 {
            log::trace!("parts:{:?}", parts);
        }
        debug_assert!(parts.len() == 2);
        let kmh: f64 = parts[1]
            .parse()
            .expect(&format!("Failed to parse LRM kmh: {}", data));
        return Speed::LRM(LRMSpec { kmh });
    }
    if data.contains("KMH") {
        let parts: Vec<&str> = data.split('-').collect();
        if parts.len() != 2 {
            log::trace!("parts:{:?}", parts);
        }
        debug_assert!(parts.len() == 2);
        let kmh: f64 = parts[1]
            .parse()
            .expect(&format!("Failed to parse KMH kmh: {}", data));
        return Speed::KMH(KMHSpec { kmh });
    }
    panic!("invalid speed string {}", data)
}

pub fn format_kmh(kmh: f64) -> String {
    format!("KMH-{:.3}", kmh)
}

pub fn allowed_speeds(end_distance: f64) -> Vec<String> {
    let mut ret = Vec::new();
    ret.push(format!("KMH-*"));
    if let Some(spec) = ACP::guess_spec(end_distance) {
        ret.push(format!("ACP-{:.0}-{:.1}", spec.km, spec.hours));
    }
    if let Some(spec) = LRM::guess_spec(end_distance) {
        ret.push(format!("LRM-{:.2}", spec.kmh));
    }
    ret
}

#[derive(Clone, Default)]
pub struct TimeParameters {
    pub controls: Vec<ControlSpeedData>,
    pub start: DateTime,
    pub speed: Speed,
    pub track_distance: f64,
}

impl TimeParameters {
    fn interpolation_points(&self) -> Vec<ControlSpeedData> {
        let ret = match &self.speed {
            Speed::ACP(spec) => {
                ACP::interpolation_controls(&self.start, self.track_distance, &self.controls, &spec)
            }
            Speed::LRM(spec) => LRM::interpolation_points(self.track_distance, &self.start, &spec),
            Speed::KMH(kmh) => {
                let mps = kmh.kmh * 1000.0 / 3600.0;
                MPS::interpolation_points(&self.controls, self.track_distance, &self.start, mps)
            }
        };
        debug_assert!(ret.is_sorted());
        debug_assert!(ret.len() >= 2);
        ret.iter().for_each(|c| debug_assert!(c.time.is_some()));
        ret
    }

    pub fn time(&self, distance: f64) -> DateTime {
        interpolate_time(&self.interpolation_points(), distance)
    }
    pub fn distance(&self, duration: &TimeDelta) -> f64 {
        interpolate_distance(&self.interpolation_points(), &self.start, duration)
    }
    pub fn duration(&self, distance_a: f64, distance_b: f64) -> TimeDelta {
        debug_assert!(distance_a <= distance_b);
        let ta = interpolate_time(&self.interpolation_points(), distance_a);
        let tb = interpolate_time(&self.interpolation_points(), distance_b);
        debug_assert!(ta <= tb);
        tb - ta
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        format::round_time,
        parameters::{self, Parameters},
    };

    use super::*;

    const TRACK_DISTANCE_1200: f64 = 1_200_000f64;
    const TRACK_DISTANCE_3000: f64 = 3_000_000f64;

    fn best_guess_acp(end_distance: f64) -> Speed {
        Speed::ACP(ACP::guess_spec(end_distance).unwrap())
    }

    fn best_guess_lrm(end_distance: f64) -> Speed {
        Speed::LRM(LRM::guess_spec(end_distance).unwrap())
    }

    #[test]
    fn test_constant_speed_mode() {
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("{}", 15.0);
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);
        let dist_300km = 300_000.0;
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_acp(TRACK_DISTANCE_1200),
            track_distance: TRACK_DISTANCE_1200,
        };

        // 300 km at 15 km/h should take 20 hours
        let time_300 = time_parameters.time(dist_300km);
        let duration_sec = (time_300 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;

        assert!(
            duration_hours == 20.0,
            "Expected ~20 hours, got {}",
            duration_hours
        );
    }

    #[test]
    fn test_acp_short_distance() {
        let _ = env_logger::try_init();
        let start_time = "2026-04-29T10:00:00+02:00".to_string();
        let start = parameters::parse_time(&start_time);
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_acp(TRACK_DISTANCE_1200),
            track_distance: TRACK_DISTANCE_1200,
        };

        let dist_40km = 40_000.0;
        let time_40 = time_parameters.time(dist_40km);
        let duration_sec = (time_40 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 1.0 + (40.0 / 20.0); // 3 hours

        assert!(
            (duration_hours - expected).abs() < 0.01,
            "Expected ~{} hours for 40km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_below_600km() {
        let _ = env_logger::try_init();
        let start_time = "2026-04-29T10:00:00+02:00".to_string();
        let start = parameters::parse_time(&start_time);
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_acp(TRACK_DISTANCE_1200),
            track_distance: TRACK_DISTANCE_1200,
        };
        let dist_300km = 300_000.0;
        let time_300 = time_parameters.time(dist_300km);
        let duration_sec = (time_300 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;

        assert!(
            duration_hours == 20.0,
            "Expected 20 hours for 300km, got {}",
            duration_hours
        );
    }

    #[test]
    fn test_acp_at_600km() {
        let _ = env_logger::try_init();
        let start = parameters::parse_time(&"2026-04-29T10:00:00+02:00");

        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_acp(TRACK_DISTANCE_1200),
            track_distance: TRACK_DISTANCE_1200,
        };

        let dist_600km = 600_000.0;
        let time_600 = time_parameters.time(dist_600km);
        let duration_sec = (time_600 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;

        assert!(
            (duration_hours - 40.0).abs() < 0.01,
            "Expected 40 hours for 600km (hard cap), got {}",
            duration_hours
        );
    }

    #[test]
    fn test_acp_between_600_and_1000km() {
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);
        let dist_800km = 800_000.0;
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_acp(TRACK_DISTANCE_1200),
            track_distance: TRACK_DISTANCE_1200,
        };

        // 800 km: 600/15 + (800-600)/11.428
        //       = 40 + 200/11.428 = 40 + 17.5 = 57.5 hours
        let time_800 = time_parameters.time(dist_800km);
        let duration_sec = (time_800 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 40.0 + (200.0 / 11.428); // ~57.5 hours

        assert!(
            (duration_hours - expected).abs() < 0.1,
            "Expected ~{} hours for 800km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_at_1000km() {
        let _ = env_logger::try_init();
        let start = parameters::parse_time(&"2026-04-29T10:00:00+02:00".to_string());
        let dist_1000km = 1_000_000.0;
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_acp(TRACK_DISTANCE_1200),
            track_distance: TRACK_DISTANCE_1200,
        };

        // 1000 km: hard cap should be 75 hours
        // Calculated: 600/15 + 400/11.428 = 40 + 35 = 75 hours
        let time_1000 = time_parameters.time(dist_1000km);
        let duration_sec = (time_1000 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;

        assert!(
            (duration_hours - 75.0).abs() < 0.1,
            "Expected 75 hours for 1000km (hard cap), got {}",
            duration_hours
        );
    }

    #[test]
    fn test_acp_above_1000km() {
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_acp(TRACK_DISTANCE_1200),
            track_distance: TRACK_DISTANCE_1200,
        };

        // 1200 km: 600/15 + 400/11.428 + 200/13.333
        //        = 40 + 35 + 15 = 90 hours
        let dist_1200km = 1_200_000.0;

        let time_1200 = time_parameters.time(dist_1200km);
        let duration_sec = (time_1200 - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 40.0 + (400.0 / 11.428) + (200.0 / 13.333);

        assert!(
            (duration_hours - expected).abs() < 0.1,
            "Expected ~{} hours for 1200km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_above_2000km() {
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-10T00:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_lrm(2f64 * TRACK_DISTANCE_1200),
            track_distance: 2f64 * TRACK_DISTANCE_1200,
        };

        // 2400 km at 10kmh => 240 hours => 10 days
        let dist = 2_400_000.0;
        let time = time_parameters.time(dist);
        let duration_sec = (time - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 240.0;

        assert!(
            (duration_hours - expected).abs() < 0.00001,
            "Expected ~{} hours for 2400km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_above_2500km() {
        let _ = env_logger::try_init();
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-10T00:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_lrm(TRACK_DISTANCE_3000),
            track_distance: TRACK_DISTANCE_3000,
        };

        // 3000 km at 200km/day => 15 days => 360 hours
        let dist = TRACK_DISTANCE_3000;
        let time = time_parameters.time(dist);
        let duration_sec = (time - start).num_seconds();
        let duration_hours = duration_sec as f64 / 3600.0;
        let expected = 360.0;

        assert!(
            (duration_hours - expected).abs() < 0.00001,
            "Expected ~{} hours for 3000km, got {}",
            expected,
            duration_hours
        );
    }

    #[test]
    fn test_acp_standard_brevets() {
        let _ = env_logger::try_init();
        // Test the standard brevet distances with their hard caps
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        // Test each standard distance
        let test_cases = vec![
            (200_000.0, 13.5),   // 200 km: 13h 30m
            (300_000.0, 20.0),   // 300 km: 20h 00m
            (400_000.0, 27.0),   // 400 km: 27h 00m
            (600_000.0, 40.0),   // 600 km: 40h 00m
            (1_000_000.0, 75.0), // 1000 km: 75h 00m
        ];

        let start = parameters::parse_time(&params.start_time);
        for (distance, expected_hours) in test_cases {
            let time_parameters = TimeParameters {
                controls: Vec::new(),
                start: start.clone(),
                speed: best_guess_acp(distance),
                track_distance: distance,
            };
            let time = time_parameters.time(distance);
            let duration_hours = (time - start).num_seconds() as f64 / 3600.0;
            println!("{} km: {:.2} hours", distance / 1000.0, duration_hours);
            assert!(
                duration_hours <= expected_hours + 0.1,
                "{} km should not exceed {} hours, got {}",
                distance / 1000.0,
                expected_hours,
                duration_hours
            );
        }
    }

    #[test]
    fn test_acp_standard_brevets_dev1() {
        let _ = env_logger::try_init();
        // Test the standard brevet distances with their hard caps
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-10T00:00:00+02:00".to_string();

        // Test each standard distance
        let test_cases = vec![
            (0_000.0, 1.0),
            (300_000.0, 20.0),
            (600_000.0, 40.0),
            (622_800.0, 41.995),
            (1_000_000.0, 75.0),
            (1_100_000.0, 82.5),
            (1_200_000.0, 87.5),
            (1_250_000.0, 90.0),
        ];

        let start = parameters::parse_time(&params.start_time);
        for (distance, expected_hours) in test_cases {
            let time_parameters = TimeParameters {
                controls: vec![ControlSpeedData {
                    distance: 1_100_000f64,
                    time: None,
                    is_end: false,
                }],
                start: start.clone(),
                speed: best_guess_acp(1_250_000f64),
                track_distance: 1_250_000f64,
            };
            let time = time_parameters.time(distance);
            let duration_hours = (time - start).num_seconds() as f64 / 3600.0;
            println!("{} km: {:.2} hours", distance / 1000.0, duration_hours);
            assert_eq!(
                duration_hours,
                expected_hours,
                "{} km should be {} hours, got {}",
                distance / 1000.0,
                expected_hours,
                duration_hours
            );
        }
    }

    #[test]
    fn test_acp_standard_brevets_exact() {
        let _ = env_logger::try_init();
        // Test the standard brevet distances with their hard caps
        let mut params = Parameters::default();
        params.speed = format!("ACP");
        params.start_time = "2026-04-29T10:00:00+02:00".to_string();

        let start = parameters::parse_time(&params.start_time);
        let time_parameters = TimeParameters {
            controls: Vec::new(),
            start: start.clone(),
            speed: best_guess_acp(TRACK_DISTANCE_1200),
            track_distance: TRACK_DISTANCE_1200,
        };
        let distance = 1_200_000f64;
        let time_end = start + TimeDelta::hours(90);
        let time = time_parameters.time(distance);
        assert!(time == time_end);
    }

    #[test]
    fn test_acp_revert() {
        let _ = env_logger::try_init();
        let start = ControlSpeedData {
            distance: 0f64,
            time: None,
            is_end: false,
        };
        let end = ControlSpeedData {
            distance: 400_000f64,
            time: None,
            is_end: true,
        };
        let controls = vec![start, end.clone()];
        let distance = 20_000f64;
        let start_time = parameters::parse_time(&"2026-04-29T00:00:00");

        let time_parameters = TimeParameters {
            controls,
            start: start_time.clone(),
            speed: best_guess_acp(end.distance),
            track_distance: end.distance,
        };

        let expected_time = parameters::parse_time(&"2026-04-29T02:00:00");
        let time = time_parameters.time(distance);
        assert_eq!(time, expected_time);
        let duration = time - start_time;
        let d = time_parameters.distance(&duration);
        assert_eq!(distance, d);
    }

    #[test]
    fn test_acp_time_parameters() {
        let _ = env_logger::try_init();
        let start = ControlSpeedData {
            distance: 0f64,
            time: None,
            is_end: false,
        };
        let end = ControlSpeedData {
            distance: 400_000f64,
            time: None,
            is_end: true,
        };
        let controls = vec![start.clone(), end.clone()];
        let start_time = parameters::parse_time(&"2026-04-29T00:00:00");
        let expected = parameters::parse_time(&"2026-04-29T01:00:00");
        let time_parameters = TimeParameters {
            controls,
            start: start_time,
            speed: best_guess_acp(end.distance),
            track_distance: end.distance,
        };
        let cut_off = time_parameters.time(0f64);
        assert_eq!(cut_off, expected);

        let cut_off = round_time(&time_parameters.time(72f64 * 1000f64));
        let expected = parameters::parse_time(&"2026-04-29T04:49:00");
        assert_eq!(cut_off, expected);
    }

    #[test]
    fn test_acp_lrm() {
        let _ = env_logger::try_init();
        log::trace!("X{:?}", super::allowed_speeds(1200_000f64));
    }
}
