use crate::storage::GeoDatabase;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use warp::Filter;

#[derive(Debug, Deserialize, Serialize)]
struct GeoAddRequest {
    key: String,
    coords: Vec<(f64, f64)>,
}

#[derive(Debug, Deserialize, Serialize)]
struct GeoSearchRequest {
    lat: f64,
    lon: f64,
    radius: f64,
}


pub fn create_api(
    db: Arc<Mutex<GeoDatabase>>,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let geoadd = warp::post()
        .and(warp::path("geoadd"))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(handle_geoadd);

    let geosearch = warp::post()
        .and(warp::path("geosearch"))
        .and(warp::body::json())
        .and(with_db(db.clone()))
        .and_then(handle_geosearch);


    let health = warp::get()
        .and(warp::path("health"))
        .and_then(handle_health);

    geoadd.or(geosearch).or(health)
}

async fn handle_health() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::json(&"HEALTH"))
}

fn with_db(
    db: Arc<Mutex<GeoDatabase>>,
) -> impl Filter<Extract = (Arc<Mutex<GeoDatabase>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn handle_geoadd(
    body: GeoAddRequest,
    db: Arc<Mutex<GeoDatabase>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut db = db.lock().unwrap();
    db.geo_add(body.key, body.coords);
    Ok(warp::reply::json(&"OK"))
}

async fn handle_geosearch(
    body: GeoSearchRequest,
    db: Arc<Mutex<GeoDatabase>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let db = db.lock().unwrap();
    let results = db.geo_search(body.lat, body.lon, body.radius);
    Ok(warp::reply::json(&results))
}

