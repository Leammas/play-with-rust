use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
type Responder<T> = oneshot::Sender<T>;
use tokio_postgres::NoTls;

// example rewritte to channel based synchronization

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async { run_server().await })
}

async fn run_server() -> () {
    // Connect to the database.
    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres password=mysecretpassword port=5432", NoTls)
        .await
        .unwrap();

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .batch_execute(
            "
CREATE TABLE kv (
    key TEXT PRIMARY KEY,
    value TEXT
);
",
        )
        .await
        .ok();

    // Bind the listener to the address
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let (tx, mut rx) = mpsc::channel::<CommandAndResponder>(32);

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            handle_command(message, &client).await;
        }
    });

    loop {
        // The second item contains the IP and port of the new connection.
        let (socket, _) = listener.accept().await.unwrap();
        tokio::spawn(handle_connection(socket, tx.clone()));
    }
}

async fn handle_command(message: CommandAndResponder, client: &tokio_postgres::Client) {
    let r = match message.cmd {
        mini_redis::Command::Set(cmd) => {
            let value = String::from_utf8(cmd.value().clone().to_vec()).unwrap();

            client
                .query(
                    "INSERT INTO kv (key, value) VALUES ($1, $2) ON CONFLICT (key) DO UPDATE SET value = $2;",
                    &[&cmd.key().to_string(), &value],
                )
                .await
                .unwrap();

            Frame::Simple("OK".to_string())
        }
        mini_redis::Command::Get(cmd) => {
            let key = cmd.key();

            let rows = client
                .query("SELECT value FROM kv WHERE key = $1", &[&key.to_string()])
                .await
                .unwrap();

                println!("r {:?}", &rows);

            if let Some(row) = rows.get(0) {
                let value: String = row.get(0);

                println!("v {}", &value);

                Frame::Bulk(Bytes::from(value))
            } else {
                Frame::Null
            }
        }
        cmd => panic!("unimplemented {:?}", cmd),
    };
    let _ = message.resp.send(r);
}

async fn handle_connection(socket: TcpStream, sender: mpsc::Sender<CommandAndResponder>) {
    // Connection, provided by `mini-redis`, handles parsing frames from
    // the socket
    let mut connection = Connection::new(socket);

    while let Some(frame) = connection.read_frame().await.unwrap() {
        let cmd = Command::from_frame(frame).unwrap();
        let (resp, resp_rx) = oneshot::channel();
        let _ = sender.send(CommandAndResponder { cmd, resp }).await;

        let response = resp_rx.await;

        println!("r {:?}", &response);

        // Write the response to the client
        connection.write_frame(&response.unwrap()).await.unwrap();
    }
}

struct CommandAndResponder {
    cmd: mini_redis::Command,
    resp: Responder<Frame>,
}
