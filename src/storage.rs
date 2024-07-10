use geo::algorithm::haversine_distance::HaversineDistance;
use geo_types::Point;
use rstar::RTree;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct GeoDatabase {
    tree: RTree<Point<f64>>,
    data: HashMap<String, Point<f64>>,
}

impl GeoDatabase {
    pub fn new() -> Self {
        GeoDatabase {
            tree: RTree::new(),
            data: HashMap::new(),
        }
    }

    pub fn geo_add(&mut self, key: String, lat: f64, lon: f64) {
        let point = Point::new(lon, lat);
        self.tree.insert(point);
        self.data.insert(key, point);
    }

    pub fn geo_search(&self, lat: f64, lon: f64, radius: f64) -> Vec<String> {
        let center = Point::new(lon, lat);
        self.tree
            .nearest_neighbor_iter(&center)
            .filter_map(|p: &Point<f64>| {
                let distance = p.haversine_distance(&center);
                if distance <= radius {
                    self.data
                        .iter()
                        .find(|&(_, &v)| v == *p)
                        .map(|(k, _)| k.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
