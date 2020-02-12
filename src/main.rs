#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
use rocket::response::Responder;
#[macro_use] extern crate influx_db_client;
use influx_db_client::{Client, Point, Points, Value, Precision};
#[macro_use] extern crate failure;
use std::{env};

fn main() {
    rocket::ignite().mount("/", routes![import_eco_device_report]).launch();
}

#[get("/?<ppap>&<hphp>&<hchc>&<water>&<heater>")]
fn import_eco_device_report(ppap: i64, hphp: i64, hchc: i64, water: i64, heater: i64) -> Result<(), ApiError> {
    let influxdb_url: Result<String, std::env::VarError> = env::var("INFLUXDB_URL");
    if influxdb_url.is_err() {
        return Err(ApiError::MissingEnvParameter("Specify INFLUXDB_URL env parameter, like http://<ip>:8086".to_string()));
    }
    let influxdb_url = influxdb_url.unwrap();
    let influxdb_db: Result<String, std::env::VarError> = env::var("INFLUXDB_DB");
    if influxdb_db.is_err() {
        return Err(ApiError::MissingEnvParameter("Specify INFLUXDB_DB env parameter, like 'eco-device'".to_string()));
    }
    let influxdb_db = influxdb_db.unwrap();

    println!("PPAP: {}, HP: {}, HC: {}, water: {} L, heater: {} Wh", ppap, hphp, hchc, water, heater);
    ingest_into_influxdb(influxdb_url, influxdb_db, ppap, hphp, hchc, water, heater)
        .map_err(|e: influx_db_client::Error| ApiError::InfluxDB(format!("{}", e)))
}

fn ingest_into_influxdb(influxdb_url: String, influxdb_db: String,
    ppap: i64, hphp: i64, hchc: i64, water: i64, heater: i64) -> Result<(), influx_db_client::Error> {
    let client = Client::new(influxdb_url, influxdb_db);

    let mut electricity_point = point!("electricity");
    electricity_point
        .add_field("ppap", Value::Integer(ppap))
        .add_field("hp", Value::Integer(hphp))
        .add_field("hc", Value::Integer(hchc))
        .add_field("heating_index", Value::Integer(heater));

    let mut water_point = point!("water");
    water_point.add_field("index", Value::Integer(water));

    let mut heating_point = point!("heating");
    heating_point.add_field("index", Value::Integer(heater));
    
    let points = points!(electricity_point, water_point, heating_point);
    client.write_points(points, Some(Precision::Seconds), None)
}

#[derive(Fail, Debug, Responder)]
pub enum ApiError {
    #[fail(display = "Input was invalid UTF-8 at index {}", _0)]
    #[response(status = 500)]
    MissingEnvParameter(String),
    #[fail(display = "{}", _0)]
    #[response(status = 500)]
    InfluxDB(String),
}
