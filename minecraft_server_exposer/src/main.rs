/*
The minecraft_server_exposer has to be run on the machine which hosts the
minecraft server.
No ports have to be opened in the firewall, as all connections are outbound!

(1) The minecraft_server_exposer will open up a connection on port 25566 to the
    proxy address specified via stdin (where the minecraft_reverse_proxy is
    running).
(2) As soon as any byte of data arrives on this streams, a new connection
    to 127.0.0.1:25565 is established and all data forwarded in both directions
    (in 2 new threads that is).
(3) A new connection will be opened to the proxy, i.e., step (1) repeated.
*/

const CONNECT_PORT_MINECRAFT: u16 = 25565;
const CONNECT_PORT_PROXYING: u16 = 25566;

use std::thread;
use std::net::TcpStream;
use std::io;
use std::io::Write;

fn main() {
    println!("Starting minecraft_server_exposer...");

    // Ask user for the proxy address:
    print!("Please enter the IP address of the reverse proxy: ");
    io::stdout().flush().unwrap();
    let mut proxy_addr = String::new();
    io::stdin().read_line(&mut proxy_addr).unwrap();
    proxy_addr = proxy_addr.trim_end().to_string(); 

    loop {
        // Connect to the proxy:
        println!("Connecting to {}:{} ...", proxy_addr, CONNECT_PORT_PROXYING);
        let mut proxy_stream =
        TcpStream::connect(format!("{}:{}", proxy_addr, CONNECT_PORT_PROXYING))
            .expect("Creating TCP stream to proxy failed!");

        // Wait until the first data arrives:
        let mut tiny_buffer: [u8; 1] = [0];
        while proxy_stream.peek(&mut tiny_buffer).unwrap_or(0) == 0 {
            // wait
        }

        // Connect to localhost on port CONNECT_PORT_MINECRAFT:
        println!("Connecting to 127.0.0.1:{} ...", CONNECT_PORT_MINECRAFT);
        let mut minecraft_stream =
        TcpStream::connect(format!("127.0.0.1:{}", CONNECT_PORT_MINECRAFT))
            .expect("Creating TCP stream to MC server on localhost failed! Is your MC server running?!");

        // Forward data in both directions using 2 new threads:
        let mut proxy_stream_clone = proxy_stream.try_clone().unwrap();
        let mut minecraft_stream_clone = minecraft_stream.try_clone().unwrap();

        thread::spawn(move || {
            io::copy(&mut proxy_stream, &mut minecraft_stream).unwrap();
        });

        thread::spawn(move || {
            io::copy(&mut minecraft_stream_clone, &mut proxy_stream_clone).unwrap();
        });
    }
}
