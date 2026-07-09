use chrono::TimeDelta;

fn time_delta(seconds: f64) -> TimeDelta {
    let nano = (seconds * 1_000_000_000f64).round() as i64;
    TimeDelta::nanoseconds(nano)
}

#[allow(non_snake_case)]
pub mod LRM {
    const LRM_MIN_DISTANCE: f64 = 1050_000f64;
    use super::time_delta;
    use crate::speed::{InterpolationPoint, LRMSpec};

    fn speed_kmh(end_distance: f64) -> f64 {
        // https://www.randonneursmondiaux.org/files/Rules_2019.pdf
        debug_assert!(end_distance >= LRM_MIN_DISTANCE);
        let distance_km = end_distance / 1000.0;
        let kmh = if distance_km >= 2500f64 {
            200.0 / 24.0
        } else if distance_km >= 1900f64 {
            10.0
        } else if distance_km >= 1300f64 {
            12.0
        } else {
            13.33
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

    pub fn interpolation_points(end_distance: f64, spec: &LRMSpec) -> Vec<InterpolationPoint> {
        let mps = spec.kmh * 1000.0 / 3600.0;
        let mut all = Vec::new();
        all.push(InterpolationPoint {
            distance: 0f64,
            duration: Some(time_delta(0f64)),
            is_end: false,
        });
        let end_duration = time_delta(end_distance / mps);
        all.push(InterpolationPoint {
            distance: end_distance,
            duration: Some(end_duration),
            is_end: true,
        });
        all.sort();
        debug_assert!(all.len() >= 2);
        all.iter().for_each(|c| debug_assert!(c.duration.is_some()));
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
pub mod ACP {
    const ACP_MAX_DISTANCE: f64 = 1250_000f64;
    use super::super::InterpolationPoint;
    use super::time_delta;
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

    fn fixed_interpolation_controls() -> Vec<InterpolationPoint> {
        let mut ret = Vec::new();
        let acp_points = fixed_interpolation_points();
        for (km, hours) in acp_points.iter() {
            ret.push(InterpolationPoint {
                distance: km * 1000f64,
                duration: Some(TimeDelta::seconds((hours * 3600f64).round() as i64)),
                is_end: false,
            });
        }
        ret
    }

    pub fn interpolation_controls(
        end_distance: f64,
        controls: &Vec<InterpolationPoint>,
        spec: &ACPSpec,
    ) -> Vec<InterpolationPoint> {
        let mut ret = Vec::new();
        debug_assert!(end_distance <= ACP_MAX_DISTANCE);
        // the START control
        ret.push(InterpolationPoint {
            distance: 0f64,
            duration: Some(time_delta(0f64)),
            is_end: false,
        });

        let spec_duration = time_delta(spec.hours * 3600.0);
        let spec_distance = spec.km * 1000.0;

        // END
        /* For brevets up to 1200 km, the end control has a
         * fixed ACP time (e.g. 90h, even if distance = 1230km).
         */
        let end = InterpolationPoint {
            distance: end_distance,
            duration: Some(spec_duration),
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
            let prelast = InterpolationPoint {
                distance: prelastc.distance,
                duration: Some(duration(prelastc.distance)),
                is_end: false,
            };
            ret.push(prelast);
        }

        let mut fixed = fixed_interpolation_controls();
        // exclude the acp point at closest_to_end_acp.
        // The right point for that is END (included above).
        fixed.retain(|c| c.distance < end_distance && c.distance != spec_distance);
        ret.extend_from_slice(&fixed);

        ret.sort();
        ret
    }
}

#[allow(non_snake_case)]
pub mod MPS {
    use super::super::InterpolationPoint;
    use super::time_delta;
    pub fn interpolation_points(
        controls: &Vec<InterpolationPoint>,
        end_distance: f64,
        mps: f64,
    ) -> Vec<InterpolationPoint> {
        let mut all = controls.clone();
        all.retain(|c| c.duration.is_some());
        // add START if needed
        if all.iter().find(|c| c.distance == 0f64).is_none() {
            all.push(InterpolationPoint {
                distance: 0f64,
                duration: Some(time_delta(0f64)),
                is_end: false,
            });
        }
        // add END if needed
        if all.iter().find(|c| c.is_end).is_none() {
            let end_duration = time_delta(end_distance / mps);
            all.push(InterpolationPoint {
                distance: end_distance,
                duration: Some(end_duration),
                is_end: true,
            });
        }
        all.sort();
        debug_assert!(all.len() >= 2);
        all.iter().for_each(|c| debug_assert!(c.duration.is_some()));
        all
    }
}
