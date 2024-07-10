pub enum Command {
    GeoAdd { key: String, lat: f64, lon: f64 },
    GeoSearch { lat: f64, lon: f64, radius: f64 },
    Heartbeat,
}

pub fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    match parts.as_slice() {
        ["GEOADD", key, lat, lon] => Some(Command::GeoAdd {
            key: key.to_string(),
            lat: lat.parse().ok()?,
            lon: lon.parse().ok()?,
        }),
        ["GEOSEARCH", lat, lon, radius] => Some(Command::GeoSearch {
            lat: lat.parse().ok()?,
            lon: lon.parse().ok()?,
            radius: radius.parse().ok()?,
        }),
        ["HEARTBEAT"] => Some(Command::Heartbeat),
        _ => None,
    }
}
