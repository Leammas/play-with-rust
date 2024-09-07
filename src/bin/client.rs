use bytes::Bytes;
use mini_redis::client::{self};
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use Command::{Get, Set};

// example code from https://tokio.rs/tokio/tutorial/shared-state

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Command>(32);
    let tx2 = tx.clone();

    // Establish a connection to the server
    let mut client = client::connect("127.0.0.1:6379").await.unwrap();

    // Spawn two tasks, one gets a key, the other sets a key
    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx.send(Command::Get {
            key: String::from("foo"),
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        tx2.send(Command::Set {
            key: String::from("foo"),
            val: "bar".into(),
            resp: resp_tx,
        })
        .await
        .unwrap();

        let res = resp_rx.await;
        println!("GOT = {:?}", res);
    });

    let manager = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            match message {
                Get { key, resp } => {
                    let r = client.get(&key).await;
                    resp.send(r);
                }
                Set { key, val, resp } => {
                    let r = client.set(&key, val).await;
                    resp.send(r);
                }
            }
        }
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}

#[derive(Debug)]
enum Command {
    Get {
        key: String,
        resp: Responder<Option<Bytes>>,
    },
    Set {
        key: String,
        val: Bytes,
        resp: Responder<()>,
    },
}

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;
