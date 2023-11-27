//! Server

extern crate mio;
extern crate std;

use mio::{
    net::{TcpListener, TcpStream},
    Events, Interest, Poll, Token,
};
use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::SocketAddr,
};

// Setup custom tokens
const SERVER: Token = Token(0);

// Server
pub struct Server {
    pub poll: Poll,
    pub listener: TcpListener,
    pub clients: HashMap<Token, TcpStream>,
}

impl Server {
    pub fn new(addr: &str, poll: Poll) -> io::Result<Self> {
        let addr = addr.parse::<SocketAddr>().map_err(|error| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid Address: {error}"),
            )
        })?;

        let listener = match TcpListener::bind(addr) {
            Ok(listener) => listener,
            Err(error) => return Err(error),
        };

        Ok(Self {
            listener,
            clients: HashMap::with_capacity(1024),
            poll,
        })
    }

    fn serve(&mut self, unique_token: &mut Token) {
        loop {
            let (mut connection, _) = match self.listener.accept() {
                Ok((connection, address)) => (connection, address),
                Err(ref error) if would_block(error) => {
                    break;
                }
                Err(ref error) if connection_error(error) => continue,
                Err(_) => return,
            };

            let token = self.generate_token(unique_token);

            self.poll
                .registry()
                .register(
                    &mut connection,
                    token,
                    Interest::READABLE.add(Interest::WRITABLE),
                )
                .unwrap();

            self.clients.insert(token, connection);
        }
    }

    fn generate_token(&self, current: &mut Token) -> Token {
        let next = current.0;
        current.0 += 1;
        Token(next)
    }

    pub fn run(&mut self) -> io::Result<()> {
        // Register the server to poll instance
        self.poll
            .registry()
            .register(&mut self.listener, SERVER, Interest::READABLE)?;

        // Create storage for events
        let mut events = Events::with_capacity(128);

        // Unique Token
        let mut unique_token = Token(SERVER.0 + 1);

        println!("Server running on {}", self.listener.local_addr()?);

        // Start the event loop
        loop {
            if let Err(error) = self.poll.poll(&mut events, None) {
                if interrupted(&error) {
                    continue;
                }
            }

            for event in events.iter() {
                match event.token() {
                    SERVER => self.serve(&mut unique_token),

                    token => {
                        let done = if let Some(stream) = self.clients.get_mut(&token) {
                            // perform task / function in loop which takes care of reading data handling errors and running main function which handles connection / does something (business logic)

                            echo(stream).unwrap()

                            // temporarily return true
                        } else {
                            false
                        };

                        if done {
                            if let Some(mut connection) = self.clients.remove(&token) {
                                self.poll.registry().deregister(&mut connection)?;
                                println!("remaining clients: {:?}", self.clients);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn echo(stream: &mut TcpStream) -> io::Result<bool> {
    let mut received_data = vec![0; 4096];
    let mut bytes_read = 0;

    loop {
        match stream.read(&mut received_data[bytes_read..]) {
            Ok(0) => {
                return Ok(true);
            }

            Ok(n) => {
                bytes_read += n;
                if bytes_read == received_data.len() {
                    received_data.resize(received_data.len() + 1024, 0);
                }
            }

            Err(ref err) if would_block(err) => break,
            Err(ref err) if interrupted(err) => continue,
            Err(ref err) if reset(err) => {
                break;
            }
            Err(err) => {
                println!("error");
                return Err(err);
            }
        }

        if bytes_read != 0 {
            let received_data = &received_data[..bytes_read];
            if let Ok(str_buf) = std::str::from_utf8(received_data) {
                println!("Received data: {}", str_buf.trim_end());
                stream.write_all(str_buf.as_bytes())?;
            } else {
                println!("Received (none UTF-8) data: {:?}", received_data);
            }
        }
    }

    Ok(false)
}

fn connection_error(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::ConnectionRefused
        || error.kind() == io::ErrorKind::ConnectionAborted
        || error.kind() == io::ErrorKind::ConnectionReset
}

fn interrupted(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Interrupted
}

fn reset(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::ConnectionReset
}

fn would_block(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::WouldBlock
}
