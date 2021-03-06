#![forbid(unsafe_code)]
use bryggio_cli::opts::{InstallTarget, Opt};
use bryggio_cli::{brewery, install, rbpi};
use bryggio_lib::{
    control::ControllerConfig,
    pub_sub::nats_client::NatsClient,
    supervisor::config::SupervisorConfig,
    supervisor::pub_sub::{NewContrData, SupervisorSubMsg},
};
use log::info;
use structopt::StructOpt;

fn run_subcommand(opt: Opt) {
    match opt {
        Opt::Publish(opts) => {
            brewery::publish_command(&opts);
        }
        Opt::Request(opts) => {
            brewery::request(&opts);
        }
        Opt::Install(target) => match target {
            InstallTarget::Supervisor(opt) => install::supervisor::install_supervisor(&opt),
            InstallTarget::Cli(_opt) => info!("Installing `bryggio-cli`"),
        },
        Opt::RbPiSetup(opt) => {
            rbpi::setup(&opt);
        }
        Opt::Test(opt) => {
            let config = SupervisorConfig::try_new(&opt.config).unwrap_or_else(|err| {
                panic!(
                    "Error parsing config '{}': {}",
                    opt.config.to_string_lossy(),
                    err
                )
            });
            let client = NatsClient::try_new(&config.nats).unwrap_or_else(|err| {
                panic!(
                    "Error connecting to NATS server:\n{:?}\n{}",
                    &config.nats, err
                );
            });

            let start_controller_msg = SupervisorSubMsg::StartController {
                contr_data: NewContrData::new(ControllerConfig::dummy(), 50.0),
            };
            client
                .publish(
                    &start_controller_msg.subject(),
                    &start_controller_msg.into(),
                )
                .unwrap_or_else(|err| panic!("Error publishing: '{}'", err));
            println!("Sleep to make sure the controller gets started");
            let switch_controller_msg = SupervisorSubMsg::SwitchController {
                contr_data: NewContrData::new(ControllerConfig::dummy(), 50.0),
            };
            std::thread::sleep(std::time::Duration::from_millis(5000));
            println!("Switching controller");
            client
                .request(
                    &switch_controller_msg.subject(),
                    &switch_controller_msg.into(),
                )
                .expect("Request error");
        }
    }
}

fn main() {
    let opt = Opt::from_args();
    bryggio_cli::init_logging(&opt);
    run_subcommand(opt)
}
