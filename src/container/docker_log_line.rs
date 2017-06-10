use chrono::{DateTime, UTC};

#[derive(Deserialize,Serialize,Debug)]
pub struct DockerLogLine {
    pub log: String,
    pub stream: String,
    pub time: DateTime<UTC>
}