use async_trait::async_trait;
use crate::pub_sub::{
    nats_client::decode_nats_data, nats_client::NatsClient, nats_client::NatsConfig, PubSubClient,
    PubSubError, PubSubMsg, Subject,
};
use async_nats::Subscription;
use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

pub fn debug<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str) {
    log(client, msg, sub_subject, LogLevel::Debug);
}

pub fn info<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str) {
    log(client, msg, sub_subject, LogLevel::Info);
}

pub fn _warning<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str) {
    log(client, msg, sub_subject, LogLevel::Warning);
}

pub fn error<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str) {
    log(client, msg, sub_subject, LogLevel::Error);
}

async fn log<T: Into<LogMsg>, C: PubSubClient>(client: &C, msg: T, sub_subject: &str, level: LogLevel) {
    let msg: LogMsg = msg.into();
    let subj = Subject(format!("{}.{}", level.main_subject(), sub_subject));

    let msg = match serde_json::to_string(&msg) {
        Ok(msg) => PubSubMsg(msg),
        Err(err) => {
            println!("Log error: {}", err.to_string());
            return;
        }
    };
    //.map_err(|err| PubSubError::MessageParse(err.to_string()))?,
    match client.publish(&subj, &msg).await {
        Ok(_) => {}
        Err(err) => println!("Log error: {}", err.to_string()),
    };
}

pub struct Log {
    level: LogLevel,
    client: NatsClient,
}

impl Log {
    pub async fn new(config: &NatsConfig, level: LogLevel) -> Self {
        let client = NatsClient::try_new(config).await.unwrap();
        Log { level, client }
    }

    pub fn log(&self, msg: &str, level: LogLevel) {
        match level {
            LogLevel::Debug => self.debug(msg),
            LogLevel::Info => self.info(msg),
            LogLevel::Warning => self.warning(msg),
            LogLevel::Error => self.error(msg),
        }
    }

    pub fn debug(&self, msg: &str) {
        if self.level <= LogLevel::Debug {
            self.write(msg, LogLevel::Debug);
        }
    }

    pub fn info(&self, msg: &str) {
        if self.level <= LogLevel::Info {
            self.write(msg, LogLevel::Info);
        }
    }

    pub fn warning(&self, msg: &str) {
        if self.level <= LogLevel::Warning {
            self.write(msg, LogLevel::Warning);
        }
    }

    pub fn error(&self, msg: &str) {
        self.write(msg, LogLevel::Error);
    }

    fn write(&self, msg: &str, level: LogLevel) {
        println!("{}: {}", level, msg);
    }
}

#[derive(Deserialize, Debug)]
struct ExtComm {
    id: String,
    cmd: String,
}

#[async_trait]
impl PubSubClient for Log {
    async fn client_loop(self) -> Result<(), PubSubError> {
        let log_sub = self.subscribe(&Subject(String::from("log.>"))).await?;
        loop {
            if let Some(msg) = log_sub.next().await {
                match LogLevel::from_msg_subject(&msg.subject) {
                    Ok(log_level) => match decode_nats_data::<LogMsg>(&msg.data) {
                        Ok(msg) => self.log(&msg.0, log_level),
                        Err(err) => self.error(&err.to_string()),
                    },
                    Err(err) => self.error(&err.to_string()),
                };
            }
        }
    }

    async fn subscribe(&self, subject: &Subject) -> Result<Subscription, PubSubError> {
        self.client.subscribe(subject).await
    }

    async fn publish(&self, subject: &Subject, msg: &PubSubMsg) -> Result<(), PubSubError> {
        self.client.publish(subject, msg).await
    }
}

#[derive(Deserialize, Serialize, Display, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl TryFrom<&str> for LogLevel {
    type Error = PubSubError;
    fn try_from(value: &str) -> Result<Self, PubSubError> {
        match value {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warning" => Ok(LogLevel::Warning),
            "error" => Ok(LogLevel::Error),
            _ => Err(PubSubError::MessageParse(format!(
                "Not a log level: '{:?}'",
                value
            ))),
        }
    }
}

impl LogLevel {
    fn from_msg_subject(subject: &str) -> Result<Self, PubSubError> {
        let mut tmp = subject.split('.');
        tmp.next();
        let log_level = tmp.next().ok_or_else(|| {
            PubSubError::MessageParse(String::from(
                "No second-level sub subject (should not happen)",
            ))
        })?;
        LogLevel::try_from(log_level)
    }

    fn main_subject(&self) -> Subject {
        let str_ = match self {
            LogLevel::Debug => "log.debug",
            LogLevel::Info => "log.info",
            LogLevel::Warning => "log.warning",
            LogLevel::Error => "log.error",
        };
        Subject(String::from(str_))
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, From)]
pub struct LogMsg(pub(crate) String);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ord() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(!(LogLevel::Debug > LogLevel::Info));
        assert!((LogLevel::Error > LogLevel::Info));
    }
}
