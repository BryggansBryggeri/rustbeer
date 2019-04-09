#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use bryggio::api;
use bryggio::brewery;
use bryggio::config;
use std::thread;

mod routes;

fn main() {
    let config_file = "./Bryggio.toml";
    let config = config::Config::new(&config_file);
    let (web_endpoint, brew_endpoint) = api::create_api_endpoints();

    let mut brewery = brewery::Brewery::new(&config, brew_endpoint);
    thread::spawn(move || brewery.run());

    rocket::ignite()
        .mount(
            "/",
            routes![
                routes::serve_static::general_files,
                routes::serve_static::javascript,
                routes::index::index,
                routes::backend::start_controller,
                routes::backend::stop_controller,
                routes::backend::get_temp,
                routes::backend::set_target_temp,
                routes::backend::get_full_state
            ],
        )
        .manage(web_endpoint)
        .launch();
}
