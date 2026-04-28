#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Location {
    pub lat: i32,
    pub long: i32,
}

pub const COORD_SCALE: i32 = 1_000_000;

pub fn scale_coord(value: f64) -> i32 {
    (value * COORD_SCALE as f64) as i32
}

pub fn unscale_coord(value: i32) -> f64 {
    value as f64 / COORD_SCALE as f64
}

pub fn validate_location(lat: i32, long: i32) -> Result<(), &'static str> {
    let lat_f = unscale_coord(lat);
    let long_f = unscale_coord(long);

    if lat_f < -90.0 || lat_f > 90.0 {
        return Err("Latitude out of range");
    }

    if long_f < -180.0 || long_f > 180.0 {
        return Err("Longitude out of range");
    }

    Ok(())
}
