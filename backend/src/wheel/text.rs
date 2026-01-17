use crate::math::Point2D;

pub enum Region {
    Inner,
    Outer,
}

fn zone(angle: f64) -> usize {
    let mut ret = 1;
    if angle < 22.5 {
        return ret; // 1
    }
    ret += 1;
    if angle < 67.5 {
        return ret; // 2
    }
    ret += 1;
    if angle < 112.5 {
        return ret;
    }
    ret += 1;
    if angle < 157.5 {
        return ret;
    }
    ret += 1;
    if angle < 202.5 {
        return ret;
    }
    ret += 1;
    if angle < 247.5 {
        return ret;
    }
    ret += 1;
    if angle < 292.5 {
        return ret;
    }
    ret += 1;
    if angle < 337.5 {
        return ret; // 8
    }
    return 1;
}

pub fn anchor(angle: f64, region: Region) -> String {
    match region {
        Region::Inner => match zone(angle) {
            1 | 5 => "middle".into(),
            2 | 3 | 4 => "end".into(),
            _ => "start".into(),
        },
        Region::Outer => match zone(angle) {
            1 | 5 => "middle".into(),
            2 | 3 | 4 => "start".into(),
            _ => "end".into(),
        },
    }
}

pub fn position(angle: f64, radius: f64, text_height: f64, region: Region) -> Point2D {
    let mut ret = Point2D::new(angle.to_radians().sin(), -angle.to_radians().cos()) * radius;
    match region {
        Region::Inner => match zone(angle) {
            1 | 2 | 8 => {
                ret.y += text_height;
            }
            3 | 7 => {
                ret.y += text_height / 2f64;
            }
            4 | 5 | 6 => {}
            _ => {
                assert!(false);
            }
        },
        Region::Outer => match zone(angle) {
            1 | 2 | 8 => {}
            3 | 7 => {
                ret.y += text_height / 2f64;
            }
            4 | 5 | 6 => {
                ret.y += text_height;
            }
            _ => {
                assert!(false);
            }
        },
    }
    ret
}

pub fn height() -> f64 {
    10.0
}
