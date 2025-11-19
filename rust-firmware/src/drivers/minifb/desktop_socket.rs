extern crate alloc;
use alloc::vec::Vec;
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{Read, Write, ErrorKind};
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::fs;
use std::string::String;

use crate::log_info;

const SOCKET_PATH: &str = "/tmp/hito_Linux.sock";
const BUFFER_SIZE: usize = 4096;

pub struct SocketProtocol {
    listener: Arc<Mutex<Option<UnixListener>>>,
    client: Arc<Mutex<Option<UnixStream>>>,
    rx_buffer: Arc<Mutex<Vec<u8>>>,
    tx_buffer: Arc<Mutex<Vec<u8>>>,
    connected: Arc<Mutex<bool>>,
}

impl SocketProtocol {
    pub fn new() -> Self {
        // Remove existing socket file if it exists
        if Path::new(SOCKET_PATH).exists() {
            let _ = fs::remove_file(SOCKET_PATH);
        }

        Self {
            listener: Arc::new(Mutex::new(None)),
            client: Arc::new(Mutex::new(None)),
            rx_buffer: Arc::new(Mutex::new(Vec::new())),
            tx_buffer: Arc::new(Mutex::new(Vec::new())),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    pub fn init(&mut self) -> Result<(), std::io::Error> {
        let listener = UnixListener::bind(SOCKET_PATH)?;
        listener.set_nonblocking(true)?;

        *self.listener.lock().unwrap() = Some(listener);

        log_info!("Linux Socket initialized at {}", SOCKET_PATH);
        Ok(())
    }

    pub fn accept_connection(&mut self) -> Result<bool, std::io::Error> {
        let mut listener_lock = self.listener.lock().unwrap();

        if let Some(ref listener) = *listener_lock {
            match listener.accept() {
                Ok((stream, _addr)) => {
                    stream.set_nonblocking(true)?;
                    *self.client.lock().unwrap() = Some(stream);
                    *self.connected.lock().unwrap() = true;
                    log_info!("Linux Socket: Client connected");
                    Ok(true)
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    Ok(false)
                }
                Err(e) => Err(e),
            }
        } else {
            Ok(false)
        }
    }

    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    pub fn receive(&mut self) -> Result<Option<Vec<u8>>, std::io::Error> {
        let mut client_lock = self.client.lock().unwrap();

        if let Some(ref mut stream) = *client_lock {
            let mut buffer = [0u8; BUFFER_SIZE];

            match stream.read(&mut buffer) {
                Ok(0) => {
                    // Connection closed
                    *self.connected.lock().unwrap() = false;
                    *client_lock = None;
                    log_info!("Linux Socket: Client disconnected");
                    Ok(None)
                }
                Ok(n) => {
                    let data = buffer[..n].to_vec();
                    log_info!("Linux Socket: Received {} bytes", n);
                    log_info!("Linux Socket: Data: {:?}", String::from_utf8_lossy(&data));
                    Ok(Some(data))
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    Ok(None)
                }
                Err(e) => {
                    *self.connected.lock().unwrap() = false;
                    *client_lock = None;
                    Err(e)
                }
            }
        } else {
            Ok(None)
        }
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
        let mut client_lock = self.client.lock().unwrap();

        if let Some(ref mut stream) = *client_lock {
            match stream.write(data) {
                Ok(n) => {
                    stream.flush()?;
                    log_info!("Linux Socket: Sent {} bytes", n);
                    Ok(n)
                }
                Err(e) => {
                    *self.connected.lock().unwrap() = false;
                    *client_lock = None;
                    Err(e)
                }
            }
        } else {
            Err(std::io::Error::new(
                ErrorKind::NotConnected,
                "No client connected"
            ))
        }
    }

    pub fn close(&mut self) {
        *self.client.lock().unwrap() = None;
        *self.listener.lock().unwrap() = None;
        *self.connected.lock().unwrap() = false;

        if Path::new(SOCKET_PATH).exists() {
            let _ = fs::remove_file(SOCKET_PATH);
        }

        log_info!("Linux Socket: Closed");
    }
}

impl Drop for SocketProtocol {
    fn drop(&mut self) {
        self.close();
    }
}