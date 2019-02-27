#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::http::RawStr;

use rustbeer::config::Config;
use rustbeer::control;
use rustbeer::control::Control;

#[get("/")]
fn index() -> &'static str {
    "BRYGGANS BRYGGERI BÄRS BB"
}

#[get("/set_target_temp?<temp>")]
fn set_target_temp(temp: Option<&RawStr>) -> String {
    temp.map(|temp| format!("Target: {} C", temp))
        .unwrap_or_else(|| "Invalid target".into())
}

#[get("/get_temp")]
fn get_temp() -> String {
    let temp = Some("63");
    temp.map(|temp| format!("Current temp: {} C", temp))
        .unwrap_or_else(|| "Could not read temp".into())
}

fn main() {
    //rocket::ignite().mount("/", routes![index, get_temp, set_target_temp]).launch();
    let offset_on = 5.0;
    let offset_off = 3.0;
    let control = control::HysteresisControl::new(offset_on, offset_off).unwrap();
    control.run();
    println!("Power: {}", control.current_power);
}
