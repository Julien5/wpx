#[cfg(test)]
mod tests {
    use crate::{
        format,
        parameters::{self, *},
        speed::{self, spec::*, *},
    };

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
            power: None,
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
            power: None,
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
            power: None,
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
            power: None,
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
            power: None,
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
            power: None,
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
            power: None,
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
            power: None,
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
            power: None,
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
                power: None,
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
                controls: vec![InterpolationPoint {
                    distance: 1_100_000f64,
                    duration: None,
                    is_end: false,
                }],
                start: start.clone(),
                speed: best_guess_acp(1_250_000f64),
                track_distance: 1_250_000f64,
                power: None,
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
            power: None,
        };
        let distance = 1_200_000f64;
        let time_end = start + TimeDelta::hours(90);
        let time = time_parameters.time(distance);
        assert!(time == time_end);
    }

    #[test]
    fn test_acp_revert() {
        let _ = env_logger::try_init();
        let start = InterpolationPoint {
            distance: 0f64,
            duration: None,
            is_end: false,
        };
        let end = InterpolationPoint {
            distance: 400_000f64,
            duration: None,
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
            power: None,
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
        let start = InterpolationPoint {
            distance: 0f64,
            duration: None,
            is_end: false,
        };
        let end = InterpolationPoint {
            distance: 400_000f64,
            duration: None,
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
            power: None,
        };
        let cut_off = time_parameters.time(0f64);
        assert_eq!(cut_off, expected);

        let cut_off = format::round_time(&time_parameters.time(72f64 * 1000f64));
        let expected = parameters::parse_time(&"2026-04-29T04:49:00");
        assert_eq!(cut_off, expected);
    }

    #[test]
    fn test_acp_lrm() {
        let _ = env_logger::try_init();
        log::trace!("X{:?}", speed::allowed_speeds(1200_000f64));
    }
}
