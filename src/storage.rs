use std::collections::HashMap;

use geo::algorithm::haversine_distance::HaversineDistance;
use geo_types::Point;
use rstar::RTree;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Debug)]
pub struct GeoDatabase {
    tree: RTree<Point<f64>>,
    data: HashMap<String, Point<f64>>,
}

impl Serialize for GeoDatabase {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("GeoDatabase", 1)?;
        state.serialize_field("data", &self.data)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for GeoDatabase {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "lowercase")]
        enum Field {
            Data,
        }

        struct GeoDatabaseVisitor;

        impl<'de> Visitor<'de> for GeoDatabaseVisitor {
            type Value = GeoDatabase;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct GeoDatabase")
            }

            fn visit_map<V>(self, mut map: V) -> Result<GeoDatabase, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut data = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Data => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                    }
                }
                let data: HashMap<String, Point<f64>> =
                    data.ok_or_else(|| de::Error::missing_field("data"))?;

                // Reconstruct the RTree from the data
                let mut tree = RTree::new();
                for point in data.values() {
                    tree.insert(*point);
                }

                Ok(GeoDatabase { tree, data })
            }
        }

        const FIELDS: &'static [&'static str] = &["data"];
        deserializer.deserialize_struct("GeoDatabase", FIELDS, GeoDatabaseVisitor)
    }
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
