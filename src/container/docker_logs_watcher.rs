use super::DockerLogLine;
use std::io::{Error as IOError, ErrorKind, BufRead, BufReader};
use std::fs::File;
use std::collections::HashMap;
use std::path::PathBuf;
use std::thread;

use notify::{Watcher, RecursiveMode, raw_watcher, op};
use std::sync::mpsc as std_mpsc;

use futures::sync::mpsc as futures_mpsc;
use futures::{ Sink, Future };

use tokio_core::reactor::Remote;

use serde_json;

//use futures_cpupool::CpuPool;

pub enum Event {
    Create(PathBuf),
    Delete(PathBuf),
    Modify(PathBuf)
}

pub struct DockerLogsWatcher {
    path: String,
    files: HashMap<PathBuf,File>
}

impl DockerLogsWatcher {
    pub fn new(path: String, remote: Remote) -> (DockerLogsWatcher, futures_mpsc::Receiver<Result<PathBuf, String>>, thread::JoinHandle<Result<(),String>>) {

        let dlw = DockerLogsWatcher {
            path: path,
            files: HashMap::new()
        };

        let (file_changes,handle) = dlw.watch(remote);

        return (dlw,file_changes,handle)
    }

    pub fn read(&mut self, path: PathBuf, buf: &mut String) -> Result<usize,IOError> {
        let exists = self.files.contains_key(&path);
        if !exists {
            let file = try!(File::open(&path));
            self.files.insert(path.clone(), file);
        }

        match self.files.get_mut(&path) {
            Some(file) => {
                let file : &mut File = file;
                println!("got file: {:?}", file);
                let mut reader = BufReader::new(file);

                let bytes_read : usize = 0;
                while let Ok(bytes_read) = reader.read_line(buf) {
                    if bytes_read == 0 {
                        break
                    }
                    println!("==== {}", buf)
                }
//                let bytes_read = try!(reader.read_line(buf));
                Ok(bytes_read)
            },
            None => {
                Err(IOError::new(ErrorKind::Other, "oh no!"))
            }
        }
    }

    pub fn docker_logs(&mut self, path: PathBuf, remote: Remote) -> futures_mpsc::Receiver<Result<DockerLogLine,String>> {
        let (lines_tx, lines_rx) = futures_mpsc::channel(1);

        let exists = self.files.contains_key(&path);
        if !exists {
            let file = File::open(&path).unwrap();
            self.files.insert(path.clone(), file);
        }

        match self.files.get_mut(&path) {
            Some(file) => {
                let file : &mut File = file;
                //                println!("got file: {:?}", file);
                let mut reader = BufReader::new(file);

                let mut buf = String::new();
                loop {
                    let bytes_read : usize = 0;

                    // TODO: read_line should be in a CPUPool
                    if let Ok(bytes_read) = reader.read_line(&mut buf) {
                        let lines_tx = lines_tx.clone();

                        match serde_json::from_str(&buf) {
                            Ok(deserialized) => {
                                if bytes_read != 0 {
                                    remote.spawn(|_|{
                                        lines_tx.send(Ok(deserialized)).then(|_sender| {
                                            Ok(())
                                        })
                                    });
                                }
                            },
                            Err(_) => {
                                remote.spawn(|_|{
                                   lines_tx.send(Err("Couldn't deserialize.".to_string())).then(|_sender|{
                                       Ok(())
                                   })
                                });
                            }
                        }

                    }
                    if bytes_read == 0 {
                        break
                    }
                    buf.clear();
                }

                //                let bytes_read = try!(reader.read_line(buf));
                //                Ok(bytes_read)
                ()
            },
            None => {
                () // Err(IOError::new(ErrorKind::Other, "oh no!"))
            }
        }



        lines_rx
    }

    pub fn path_to_lines_rx(&mut self, path: PathBuf, remote: Remote) -> futures_mpsc::Receiver<Result<String,String>> {
        let (lines_tx, lines_rx) = futures_mpsc::channel(1);

        let exists = self.files.contains_key(&path);
        if !exists {
            let file = File::open(&path).unwrap();
            self.files.insert(path.clone(), file);
        }

        match self.files.get_mut(&path) {
            Some(file) => {
                let file : &mut File = file;
                //                println!("got file: {:?}", file);
                let mut reader = BufReader::new(file);

                loop {
                    let bytes_read : usize = 0;
                    let mut buf = String::new();

                    if let Ok(bytes_read) = reader.read_line(&mut buf) {
                        let lines_tx = lines_tx.clone();
                        if bytes_read != 0 {
                            remote.spawn(|_|{
                                lines_tx.send(Ok(buf)).then(|_sender| {
                                    Ok(())
                                })
                            });
                        }
                    }
                    if bytes_read == 0 {
                        break
                    }
                }

                //                let bytes_read = try!(reader.read_line(buf));
                //                Ok(bytes_read)
                ()
            },
            None => {
                () // Err(IOError::new(ErrorKind::Other, "oh no!"))
            }
        }



        lines_rx
    }

    pub fn lines(&mut self, path: PathBuf, remote: Remote) -> Result<futures_mpsc::Receiver<Result<String, String>>,IOError> {
        let (lines_tx, lines_rx) = futures_mpsc::channel(1);

        let exists = self.files.contains_key(&path);
        if !exists {
            let file = try!(File::open(&path));
            self.files.insert(path.clone(), file);
        }

        match self.files.get_mut(&path) {
            Some(file) => {
                let file : &mut File = file;
//                println!("got file: {:?}", file);
                let mut reader = BufReader::new(file);

                loop {
                    let bytes_read : usize = 0;
                    let mut buf = String::new();

                    if let Ok(bytes_read) = reader.read_line(&mut buf) {
                        let lines_tx = lines_tx.clone();
                        if bytes_read != 0 {
                            remote.spawn(|_|{
                                let deleteme = buf.clone();
                                lines_tx.send(Ok(buf)).then(move |_sender| {
                                    println!("we flushed out the line {}", deleteme);
                                    Ok(())
                                })
                            });
                        }
                    }
                    if bytes_read == 0 {
                        break
                    }
                }

                //                let bytes_read = try!(reader.read_line(buf));
//                Ok(bytes_read)
                ()
            },
            None => {
                () // Err(IOError::new(ErrorKind::Other, "oh no!"))
            }
        }



        Ok(lines_rx)
    }

    pub fn watch(&self, remote: Remote) ->  (futures_mpsc::Receiver<Result<PathBuf, String>>, thread::JoinHandle<Result<(),String>>) {
        let (futures_tx, futures_rx) = futures_mpsc::channel(1);

        let path = self.path.clone();

        let guard = thread::spawn(move || {

            let (tx, rx) = std_mpsc::channel();
            let mut watcher = match raw_watcher(tx) {
                Ok(thing) => thing,
                Err(_) => return Err("failed to watch".to_string())
            };
            watcher.watch(&path[..], RecursiveMode::NonRecursive).unwrap();

            loop {
                match rx.recv() {
                    Ok(event) => {
                        let futures_tx = futures_tx.clone();
                        if event.path.is_some() && event.op.is_ok() && (event.op.as_ref().unwrap().contains(op::WRITE) ||  event.op.as_ref().unwrap().contains(op::REMOVE) || event.op.as_ref().unwrap().contains(op::CREATE)) {
                            let path = event.path.to_owned().unwrap();
                            remote.spawn(|_|{
                                futures_tx.send(Ok(path)).then(|_sender|{
                                    Ok(())
                                })
                            })
                        }
                    },
                    err => {
                        println!("watch error: {:?}", err);
                    },
                }
            }
        });

        (futures_rx, guard)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
//    use super::super::DockerLogLine;

    #[test]
    pub fn doit(){
        let dl = DockerLogsWatcher::new("/tmp/deleteme".to_string());
        dl.doit();
    }
}