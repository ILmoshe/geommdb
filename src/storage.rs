use std::collections::HashMap;
use geo::{HaversineDistance, Point, Polygon};
use rstar::{RTree, RTreeObject, AABB, PointDistance};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GeoDatabase {
    points: HashMap<String, Point<f64>>,
    polygons: HashMap<String, Polygon<f64>>,
    point_tree: RTree<Point<f64>>,
    polygon_tree: RTree<Polygon<f64>>,
}

impl GeoDatabase {
    pub fn new() -> Self {
        GeoDatabase {
            points: HashMap::new(),
            polygons: HashMap::new(),
            point_tree: RTree::new(),
            polygon_tree: RTree::new(),
        }
    }

    pub fn geo_add(&mut self, key: String, coords: Vec<(f64, f64)>) {
        if coords.len() == 1 {
            let point = Point::new(coords[0].1, coords[0].0); // (lon, lat)
            self.point_tree.insert(point);
            self.points.insert(key, point);
        } else {
            let polygon = Polygon::new(coords.into(), vec![]);
            self.polygon_tree.insert(polygon.clone());
            self.polygons.insert(key, polygon);
        }
    }

    pub fn geo_search(&self, lat: f64, lon: f64, radius: f64) -> Vec<String> {
        let center = Point::new(lon, lat);
        let mut results = Vec::new();

        // Search for points within the radius
        for point in self.point_tree.nearest_neighbor_iter(&center).filter(|p| p.haversine_distance(&center) <= radius) {
            if let Some((key, _)) = self.points.iter().find(|(_, &v)| v == *point) {
                results.push(key.clone());
            }
        }

        // Create an AABB for the search radius
        let search_aabb = AABB::from_corners(
            Point::new(lon - radius, lat - radius),
            Point::new(lon + radius, lat + radius)
        );

        // Search for polygons that intersect with the radius circle
        for polygon in self.polygon_tree.locate_in_envelope_intersecting(&search_aabb) {
            if let Some((key, _)) = self.polygons.iter().find(|(_, v)| v.clone() == polygon) {
                results.push(key.clone());
            }
        }

        results
    }

    pub fn geo_get(&self, key: &str) -> Option<String> {
        if let Some(point) = self.points.get(key) {
            Some(format!("POINT({} {})", point.y(), point.x()))
        } else if let Some(polygon) = self.polygons.get(key) {
            Some(format!("POLYGON(({}))", polygon.exterior().points().map(|p| format!("{} {}", p.y(), p.x())).collect::<Vec<_>>().join(", ")))
        } else {
            None
        }
    }
}
