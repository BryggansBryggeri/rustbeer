use crate::actor;
use crate::api;
use crate::config;
use crate::control;
use crate::control::Control;
use crate::sensor;
use std::error as std_error;
use std::sync;
use std::thread;

pub enum Command {
    GetMeasurement,
    SetTarget,
    StartController,
    StopController,
    GetFullState,
    Error,
}

pub struct Brewery {
    api_endpoint: api::BreweryEndpoint,
    controller: sync::Arc<sync::Mutex<Box<dyn control::Control>>>,
    sensor_handle: sensor::SensorHandle,
    actor: actor::ActorHandle,
}

impl Brewery {
    pub fn new(brew_config: &config::Config, api_endpoint: api::BreweryEndpoint) -> Brewery {
        let control_box: Box<dyn Control> =
            Box::new(control::hysteresis::Controller::new(1.0, 0.0).expect("Invalid parameters."));
        let controller = sync::Arc::new(sync::Mutex::new(control_box));

        let actor: actor::ActorHandle = sync::Arc::new(sync::Mutex::new(Box::new(
            actor::dummy::Actor::new("dummy"),
        )));

        // TODO: Fix ugly hack. Remove to handle if no sensor data is provided.
        let sensor_config = brew_config.sensors.clone().unwrap();
        let sensor: Box<dyn sensor::Sensor> =
            Box::new(sensor::dummy::Sensor::new(String::from(&sensor_config.id)));
        let sensor_handle = sync::Arc::new(sync::Mutex::new(sensor));
        Brewery {
            api_endpoint,
            controller,
            sensor_handle,
            actor,
        }
    }

    pub fn run(&mut self) {
        loop {
            let request = match self.api_endpoint.receiver.recv() {
                Ok(request) => request,
                Err(_) => api::Request {
                    command: Command::Error,
                    id: None,
                    parameter: None,
                },
            };
            let response = self.process_request(&request);
            self.api_endpoint.sender.send(response).unwrap();
        }
    }

    fn process_request(&mut self, request: &api::Request) -> api::Response {
        match request.command {
            Command::StartController => match self.start_controller() {
                Ok(_) => api::Response {
                    result: None,
                    message: None,
                    success: true,
                },
                Err(err) => api::Response {
                    result: None,
                    message: Some(err.to_string()),
                    success: false,
                },
            },

            Command::StopController => match self.change_controller_state(control::State::Inactive)
            {
                Ok(_) => api::Response {
                    result: None,
                    message: None,
                    success: true,
                },
                Err(err) => api::Response {
                    result: None,
                    message: Some(err.to_string()),
                    success: false,
                },
            },

            Command::GetMeasurement => match sensor::get_measurement(&self.sensor_handle) {
                Ok(measurement) => api::Response {
                    result: Some(measurement),
                    message: None,
                    success: true,
                },
                Err(err) => api::Response {
                    result: None,
                    message: Some(err.to_string()),
                    success: false,
                },
            },

            Command::SetTarget => match self.change_controller_target(request.parameter) {
                Ok(()) => api::Response {
                    result: None,
                    message: None,
                    success: true,
                },
                Err(err) => api::Response {
                    result: None,
                    message: Some(err.to_string()),
                    success: false,
                },
            },

            _ => api::Response {
                result: None,
                message: Some(String::from("Not implemented yet")),
                success: false,
            },
        }
    }

    fn start_controller(&mut self) -> Result<(), Box<dyn std_error::Error>> {
        let mut controller = match self.controller.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock. Error: {}", err),
        };

        match controller.get_state() {
            control::State::Inactive => {
                let controller_send = self.controller.clone();
                let actor = self.actor.clone();
                let sensor = self.sensor_handle.clone();
                thread::spawn(move || control::run_controller(controller_send, actor, sensor));
                controller.set_state(control::State::Automatic);
            }
            control::State::Automatic => println!("Already running"),
            control::State::Manual => {}
        };
        Ok(())
    }

    fn change_controller_state(
        &mut self,
        new_state: control::State,
    ) -> Result<(), Box<dyn std_error::Error>> {
        let mut controller = match self.controller.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock. Error {}.", err),
        };
        controller.set_state(new_state);
        Ok(())
    }

    fn change_controller_target(
        &mut self,
        new_target: Option<f32>,
    ) -> Result<(), Box<dyn std_error::Error>> {
        let mut controller = match self.controller.lock() {
            Ok(controller) => controller,
            Err(err) => panic!("Could not acquire controller lock. Error {}.", err),
        };
        if let Some(new_target) = new_target {
            controller.set_target(new_target);
        };
        Ok(())
    }
}
