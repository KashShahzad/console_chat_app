use std::io::{ErrorKind, Read, Write};
//allow us to create server and listen to a port
use std::net::TcpListener;
//allow us to create channels
use std::sync::mpsc;
use std::thread;

//holds the address of local host
const LOCAL: &str = "127.0.0.1:6000";
//holds the size of message buffer
const MSG_SIZE: usize = 64;

fn sleep() {
    thread::sleep(::std::time::Duration::from_millis(100));
}

fn main() {
    //expect is used to set the failure case
    let server = TcpListener::bind(LOCAL).expect("listener failed to bind");
    //non-blocking mode actually let server to check constantly for messages
    server
        .set_nonblocking(true)
        .expect("failed to initialize non-blocking");

    //saving multiple clients
    let mut clients = vec![];
    //create a channel and bind it to a string type, ensuring that strings will be passed through the channel
    let (tx, rx) = mpsc::channel::<String>();
    loop {
        if let Ok((mut socket, addr)) = server.accept() {
            println!("Client {} connection", addr);
            let tx = tx.clone();
            clients.push(socket.try_clone().expect("failed to clone client"));

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SIZE];

                match socket.read_exact(&mut buff) {
                    Ok(_) => {
                        let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        println!("{} : {:?}", addr, msg);
                        //sending message from transmitter to receiver
                        tx.send(msg).expect("failed to send message to rx");
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with: {}", addr);
                        break;
                    }
                }
                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);

                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }
        sleep();
    }
}
