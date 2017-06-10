extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate chrono;
extern crate notify;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_core;

pub mod container;
pub use container::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_one() {
        let sample = r#"{
            "log":"time=\"2017-05-02T15:56:35Z\" level=info msg=\"PurgeUploads starting: olderThan=2017-04-25 15:56:35.846514213 +0000 UTC, actuallyDelete=true\" \n",
            "stream":"stdout",
            "time":"2017-05-02T15:56:35.846897572Z"
        }"#;

        let _: DockerLogLine = serde_json::from_str(sample).unwrap();
    }

    #[test]
    fn parse_many() {
        let sample = r#"{"log":"time=\"2017-05-02T15:56:35Z\" level=info msg=\"PurgeUploads starting: olderThan=2017-04-25 15:56:35.846514213 +0000 UTC, actuallyDelete=true\" \n","stream":"stdout","time":"2017-05-02T15:56:35.846897572Z"}
    {"log":"time=\"2017-05-02T15:56:35Z\" level=info msg=\"Purge uploads finished.  Num deleted=0, num errors=0\" \n","stream":"stdout","time":"2017-05-02T15:56:35.849976968Z"}
    {"log":"time=\"2017-05-02T15:56:35Z\" level=info msg=\"Starting upload purge in 24h0m0s\" go.version=go1.7.3 instance.id=e930db88-e8b8-47b0-ade1-20183a3167b4 version=v2.6.0 \n","stream":"stdout","time":"2017-05-02T15:56:35.850005422Z"}"#;

        use std::io::{BufRead, BufReader, Cursor};
        let sample_buf = BufReader::new(Cursor::new(sample));

        for sample in sample_buf.lines() {
            let _: DockerLogLine = serde_json::from_str(&sample.unwrap()).unwrap();
        }
    }

//    #[test]
//    fn parse_file() {
//        use std::fs::{ File };
//
//        let f = File::open("/Users/xlange/30ece7c43b7c302f43e54ac9f299ca36efb8184375eac0563c9340a15100ee3c-json.log").unwrap();
//
//        use std::io::{BufRead, BufReader};
//        let buf_f = BufReader::new(&f);
//        for sample in buf_f.lines() {
//            let line = sample.unwrap();
//            let _: DockerLogLine = serde_json::from_str(&line[..]).unwrap();
//        }
//    }
}