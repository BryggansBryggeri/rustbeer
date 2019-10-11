use crate::api;
use crate::brewery;
use crate::sensor;
use rocket::State;
use rocket_contrib::json;

#[get("/start_controller?<controller_id>&<sensor_id>&<actor_id>")]
pub fn start_controller(
    controller_id: String,
    sensor_id: String,
    actor_id: String,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let request = brewery::Command::StartController {
        controller_id,
        sensor_id,
        actor_id,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/stop_controller?<controller_id>")]
pub fn stop_controller(
    controller_id: String,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let request = brewery::Command::StopController { controller_id };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/set_target_signal?<controller_id>&<new_target_signal>")]
pub fn set_target_signal(
    controller_id: String,
    new_target_signal: f32,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let request = brewery::Command::SetTarget {
        controller_id,
        new_target_signal,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/get_measurement?<sensor_id>")]
pub fn get_measurement(
    sensor_id: String,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let request = brewery::Command::GetMeasurement { sensor_id };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/add_sensor?<sensor_id>&<sensor_type>")]
pub fn add_sensor(
    sensor_id: String,
    sensor_type: String,
    api_endpoint: State<api::WebEndpoint>,
) -> json::Json<api::Response> {
    let sensor_type = sensor::SensorType::from_str(sensor_type);
    let request = brewery::Command::AddSensor {
        sensor_id,
        sensor_type,
    };
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}

#[get("/get_full_state")]
pub fn get_full_state(api_endpoint: State<api::WebEndpoint>) -> json::Json<api::Response> {
    let request = brewery::Command::GetFullState;
    let api_response = api_endpoint.send_and_wait_for_response(request);
    api::generate_web_response(api_response)
}
