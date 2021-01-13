use crate::opts::PubSubOpt;
use bryggio_lib::config::Config;
use bryggio_lib::pub_sub::nats_client::NatsClient;
use bryggio_lib::pub_sub::{PubSubMsg, Subject};

async fn get_client(opt: &PubSubOpt) -> NatsClient {
    let config = Config::try_new(&opt.config).unwrap_or_else(|err| {
        panic!(
            "Error parsing config '{}': {}",
            opt.config.to_string_lossy(),
            err
        )
    });
    NatsClient::try_new(&config.nats).await.unwrap_or_else(|err| {
        panic!(
            "Error connecting to NATS server:\n{:?}\n{}",
            &config.nats, err
        );
    })
}

pub async fn request(opt: &PubSubOpt) {
    let response = get_client(opt).await
        .request(&Subject(opt.topic.clone()), &PubSubMsg(opt.msg.clone())).await
        .unwrap_or_else(|err| panic!("Error publishing: '{}'", err));
    println!("Response: {:?}", response);
}

pub async fn publish_command(opt: &PubSubOpt) {
    get_client(opt).await
        .publish(&Subject(opt.topic.clone()), &PubSubMsg(opt.msg.clone())).await
        .unwrap_or_else(|err| panic!("Error publishing: '{}'", err));
}
