//{"log":"time=\"2017-05-02T15:56:35Z\" level=info msg=\"PurgeUploads starting: olderThan=2017-04-25 15:56:35.846514213 +0000 UTC, actuallyDelete=true\" \n","stream":"stdout","time":"2017-05-02T15:56:35.846897572Z"}
//{"log":"time=\"2017-05-02T15:56:35Z\" level=info msg=\"Purge uploads finished.  Num deleted=0, num errors=0\" \n","stream":"stdout","time":"2017-05-02T15:56:35.849976968Z"}
//{"log":"time=\"2017-05-02T15:56:35Z\" level=info msg=\"Starting upload purge in 24h0m0s\" go.version=go1.7.3 instance.id=e930db88-e8b8-47b0-ade1-20183a3167b4 version=v2.6.0 \n","stream":"stdout","time":"2017-05-02T15:56:35.850005422Z"}

extern crate whaletail;
use whaletail::DockerLogsWatcher;

extern crate tokio_core;
use tokio_core::reactor::Core;

extern crate futures;
use futures::{Stream};

// http://hermanradtke.com/2017/03/03/future-mpsc-queue-with-tokio.html
fn main() {
    let mut core = Core::new().expect("Failed to create core");
    let (mut dl, file_changes, handle) = DockerLogsWatcher::new("/tmp/deleteme".to_owned(), core.remote());

    let lines_core = core.remote();
    let job = file_changes.map(|path| dl.docker_logs(path.unwrap(), lines_core.clone())).flatten().for_each(|dl| {
        println!("docker log: {:?}", dl);
        Ok(())
    });

    core.run(job).expect("failed to run");
    let _ = handle.join().unwrap();
}
