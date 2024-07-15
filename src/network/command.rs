pub enum Command {
    GeoAdd { key: String, coords: Vec<(f64, f64)> },
    GeoSearch { lat: f64, lon: f64, radius: f64 },
    GeoGet { key: String },
    Heartbeat,
}

pub fn parse_command(input: &str) -> Option<Command> {
    let parts: Vec<&str> = input.trim().split_whitespace().collect();
    match parts.as_slice() {
        ["GEOADD", key, rest @ ..] => {
            let coords: Vec<(f64, f64)> = rest
                .chunks(2)
                .filter_map(|chunk| {
                    if chunk.len() == 2 {
                        Some((chunk[0].parse().ok()?, chunk[1].parse().ok()?))
                    } else {
                        None
                    }
                })
                .collect();
            Some(Command::GeoAdd { key: key.to_string(), coords })
        }
        ["GEOSEARCH", lat, lon, radius] => Some(Command::GeoSearch {
            lat: lat.parse().ok()?,
            lon: lon.parse().ok()?,
            radius: radius.parse().ok()?,
        }),
        ["GEOGET", key] => Some(Command::GeoGet {
            key: key.to_string(),
        }),
        ["HEARTBEAT"] => Some(Command::Heartbeat),
        _ => None,
    }
}
