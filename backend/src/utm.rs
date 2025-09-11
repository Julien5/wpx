pub type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Clone)]
pub struct UTMPoint(pub f64, pub f64);

impl UTMPoint {
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
    pub fn xy(&self) -> (f64, f64) {
        (self.0, self.1)
    }
}
