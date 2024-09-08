use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
type Responder<T> = oneshot::Sender<T>;

// example rewritte to channel based synchronization

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { run_server().await })
}

async fn run_server() -> () {
    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let (tx, mut rx) = mpsc::channel::<CommandAndResponder>(32);

    let manager = tokio::spawn(async move {
        // A hashmap is used to store data
        let mut db: HashMap<String, Bytes> = HashMap::new();

        while let Some(message) = rx.recv().await {
            handle_command(message, &mut db);
        }
    });

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(socket, tx.clone()));
    }
}

fn handle_command(message: CommandAndResponder, db: &mut HashMap<String, Bytes>) {
    let r = match message.cmd {
        mini_redis::Command::Set(cmd) => {
            db.insert(cmd.key().to_string(), cmd.value().clone());
            Frame::Simple("OK".to_string())
        }
        mini_redis::Command::Get(cmd) => {
            if let Some(value) = db.get(cmd.key()) {
                Frame::Bulk(value.clone())
            } else {
                Frame::Null
            }
        }
        cmd => panic!("unimplemented {:?}", cmd),
    };
    message.resp.send(r);
}

async fn handle_connection(socket: TcpStream, sender: mpsc::Sender<CommandAndResponder>) {
    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame).unwrap();
        let (resp, resp_rx) = oneshot::channel();
        sender.send(CommandAndResponder{cmd, resp}).await;

        let response = resp_rx.await;

        // Write the response to the client
        connection.write_frame(&response.unwrap()).await.unwrap();
    }
}

struct CommandAndResponder {
    cmd: mini_redis::Command,
    resp: Responder<Frame>,
}
