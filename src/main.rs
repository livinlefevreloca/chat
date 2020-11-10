use std::net::{TcpStream, TcpListener};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::thread;
use std::borrow::Cow;
use std::io::prelude::*;
use std::result::Result;
use std::error::Error;

mod errors;

fn main() -> std::io::Result<()> {
    
    let listener = TcpListener::bind("127.0.0.1:8000")?;
    let room = Room::new("Cool Chat Room");
    let locked_room = Arc::new(Mutex::new(room));
    


    for (i, incoming) in listener.incoming().enumerate() {
        let stream = incoming.unwrap();
        let cloned_stream = stream.try_clone().unwrap();
        
        let client = Client::new(i, stream); 
        
        {
            let mut room = locked_room.lock().unwrap();
            room.add_client(i, cloned_stream)
        }
        let room = Arc::clone(&locked_room); 
        thread::spawn(move || client.handle(room));
    }

    Ok(())
}


struct Client {
    buf: [u8; 512],
    id: usize,
    sock: TcpStream,
}

impl Client {

    pub fn new(id: usize, sock: TcpStream) -> Self {
        let buf = [0 as u8; 512];

        Client { buf, id, sock}
    }

    pub fn handle(mut self, locked_room: Arc<Mutex<Room>>) -> Result<(), Box<dyn Error + Send + Sync>> { 
        {
            let room = locked_room.lock().unwrap();
            self.sock.write(format!("Hello welcome to the Room: {}", &room.name).as_bytes())?;
        }
        loop {
            match self.sock.read(&mut self.buf) {
                Ok(bytes) => {
                    println!("user {} sent {} bytes", self.id, bytes);
                    let mut room = locked_room.lock().unwrap();
                    room.broadcast(&self.id, String::from_utf8_lossy(&self.buf).into());
                },
                Err(err) => {
                    match err.kind() {
                         std::io::ErrorKind::BrokenPipe => {
                            println!("Client {} closed the connection...cleaning up", self.id);
                            let mut room = locked_room.lock().unwrap();
                            room.cleanup_client(self.id)?; 
                            break;
                         },
                         _ => {
                            eprintln!("Some error occuered: {}", err);
                            break;
                         }

                    } 
                }
            }
            self.buf.iter_mut().map(|i| *i = 0 as u8).for_each(drop);

        }
        Ok(())
    }
}


struct Room<'a> {
    clients: HashMap<usize, TcpStream>,
    pub name: Cow<'a, str>,
}


impl<'a> Room<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(name: S) -> Self {
        let clients = HashMap::new();
        Self { name: name.into(), clients }
    }

    pub fn add_client(&mut self, id: usize, sock: TcpStream) {
        self.clients.insert(id, sock);
    }

    pub fn cleanup_client(&mut self, id: usize) -> Result<(), errors::ChatError> {
        match self.clients.remove(&id) {
            Some(_) => {
                Ok(())
            },
            None => {
                Err(errors::ChatError::new(
                    errors::ChatErrorKind::RemoveClientError("client {} was not found and could not be removed)")
                ))
            }
        }
    }
    
    pub fn broadcast(&mut self, sender_id: &usize, msg: String) {
        self.clients.iter_mut().map(|(id, client)|{
            if id != sender_id {
                println!("Sending message to {}", id);
                client.write(msg.as_bytes()).unwrap();
            }
        }).for_each(drop);
    }
}


