use async_nats::{Message, Subscription};
use async_trait::async_trait;
use crate::actor::Actor;
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsConfig, ClientId,
    ClientState, PubSubClient, PubSubError, PubSubMsg, Subject,
};
use crate::time::TimeStamp;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
pub struct ActorClient {
    id: ClientId,
    actor: Box<dyn Actor + Sync>,
    /// TODO: Make generic over PubSubClient
    client: NatsClient,
}

impl ActorClient {
    pub async fn new(id: ClientId, actor: Box<dyn Actor + Sync>, config: &NatsConfig) -> Self {
        let client = NatsClient::try_new(config).await.unwrap();
        ActorClient { id, actor, client }
    }

    fn gen_signal_subject(&self) -> Subject {
        Subject(format!("actor.{}.current_signal", self.id))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignalMsg {
    pub(crate) id: ClientId,
    pub(crate) timestamp: TimeStamp,
    pub(crate) signal: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActorSubMsg {
    #[serde(rename = "set_signal")]
    SetSignal(SignalMsg),
    #[serde(rename = "stop")]
    Stop,
}

impl TryFrom<Message> for ActorSubMsg {
    type Error = PubSubError;
    fn try_from(msg: Message) -> Result<Self, Self::Error> {
        decode_nats_data(&msg.data)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ActorPubMsg {
    CurrentSignal(SignalMsg),
}

impl Into<PubSubMsg> for ActorPubMsg {
    fn into(self) -> PubSubMsg {
        match &self {
            ActorPubMsg::CurrentSignal(signal_msg) => {
                PubSubMsg(serde_json::to_string(&signal_msg).expect("Pub sub serialization error"))
            }
        }
    }
}

#[async_trait]
impl PubSubClient for ActorClient {
    async fn client_loop(mut self) -> Result<(), PubSubError> {
        let sub = self.subscribe(&Subject(format!("actor.{}.set_signal", self.id))).await?;
        let mut state = ClientState::Active;
        while state == ClientState::Active {
            if let Some(contr_message) = sub.next().await {
                if let Ok(msg) = ActorSubMsg::try_from(contr_message) {
                    match msg {
                        ActorSubMsg::SetSignal(msg) => {
                            if let Ok(()) = self.actor.set_signal(msg.signal) {
                                self.publish(
                                    &self.gen_signal_subject(),
                                    &ActorPubMsg::CurrentSignal(msg).into(),
                                ).await?;
                            }
                        }
                        ActorSubMsg::Stop => state = ClientState::Inactive,
                    }
                }
            }
        }
        // TODO: Exit gracefully
        Ok(())
    }

    async fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject).await
    }

    async fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg).await
    }
}
