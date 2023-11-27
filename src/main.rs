mod server;

use mio::Poll;
use server::Server;

fn main() {
    // Create a poll instance
    let poll = Poll::new().unwrap();

    // Setup the server
    let mut server = Server::new("127.0.0.1:13265", poll).unwrap();

    // Run the server
    server.run().unwrap();
}
