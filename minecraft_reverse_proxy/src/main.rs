/*
The minecraft_reverse_proxy has to be run on a machine whose firewall is
configured to allow incoming connections to ports 25565 and 25566.

* Port 25565 is where the minecraft clients will connect to
  (the actual reverse proxy part).
* Port 25566 is where the minecraft_server_exposer will connect to.
*/

const LISTEN_PORT_MINECRAFT: u16 = 25565;
const LISTEN_PORT_PROXYING: u16 = 25566;

use std::thread;
use std::net::{TcpListener, TcpStream};
use std::sync::RwLock;
use std::collections::VecDeque;
use std::io;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref PROXY_CONNECTIONS: RwLock<VecDeque<TcpStream>> = RwLock::new(VecDeque::new());
    static ref MC_CLIENT_CONNECTIONS: RwLock<VecDeque<TcpStream>> = RwLock::new(VecDeque::new());
}

fn main() {
    println!("Starting minecraft_reverse_proxy...");

    // Thread A: accepts incoming connections from the minecraft_server_exposer
    //           and adds them to PROXY_CONNECTIONS:
    thread::spawn(|| {
        let proxy_listener =
        TcpListener::bind(format!("0.0.0.0:{}", LISTEN_PORT_PROXYING))
            .expect("Creating TCP listener for proxying failed!");

        for proxy_stream in proxy_listener.incoming() {
            match proxy_stream {
                Ok(proxy_stream) => {
                    println!("Accepted proxy connection from {:?} on {:?}",
                        proxy_stream.peer_addr(),
                        proxy_stream.local_addr());
                    PROXY_CONNECTIONS.write().unwrap().push_back(proxy_stream);
                },
                Err(err) => {
                    eprintln!("Error accepting proxy connection: {}", err);
                }
            }
        }
    });

    // Thread B: accepts incoming connections from the minecraft clients
    //           and adds them to MC_CLIENT_CONNECTIONS:
    thread::spawn(|| {
        let mc_client_listener =
        TcpListener::bind(format!("0.0.0.0:{}", LISTEN_PORT_MINECRAFT))
            .expect("Creating TCP listener for minecraft clients failed!");

        for minecraft_stream in mc_client_listener.incoming() {
            match minecraft_stream {
                Ok(minecraft_stream) => {
                    println!("Accepted minecraft client from {:?} on {:?}",
                        minecraft_stream.peer_addr(),
                        minecraft_stream.local_addr());
                    MC_CLIENT_CONNECTIONS.write().unwrap().push_back(minecraft_stream);
                },
                Err(err) => {
                    eprintln!("Error accepting minecraft client: {}", err);
                }
            }  
        }
    });

    // Main thread: brings together pairs of proxy and minecraft client
    //              TcpStreams and creates 2 new threads for each pair
    //              (1 for each direction of traffic):
    loop {
        while PROXY_CONNECTIONS.read().unwrap().is_empty() || MC_CLIENT_CONNECTIONS.read().unwrap().is_empty() {
            // wait until there is both a proxy and a client connection that we can join
        }

        let mut proxy_stream = PROXY_CONNECTIONS.write().unwrap().pop_front().unwrap();
        let mut mc_client_stream = MC_CLIENT_CONNECTIONS.write().unwrap().pop_front().unwrap();

        let mut proxy_stream_clone = proxy_stream.try_clone().unwrap();
        let mut mc_client_stream_clone = mc_client_stream.try_clone().unwrap();

        thread::spawn(move || {
            io::copy(&mut proxy_stream, &mut mc_client_stream).unwrap();
        });

        thread::spawn(move || {
            io::copy(&mut mc_client_stream_clone, &mut proxy_stream_clone).unwrap();
        });
    }
}
