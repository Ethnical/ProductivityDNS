use anyhow::Result;
use rocket::log::private::info;
use rocket::*;
use simple_dns_server::{Config, SimpleDns};
use std;
use std::net::IpAddr;
use std::net::Ipv4Addr;

use tokio::sync::mpsc;

#[macro_use]
extern crate rocket;

use async_mutex::Mutex;
use rocket::State;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;

#[post("/", data = "<status>")]
async fn status(status: &str, sender: &State<SenderPlease>) -> String {
    if status.contains("on") {
        info!("Status is {}", status);
        let mut sender = sender.inner().0.lock().await;
        if sender.is_none() {
            let (tx, rx) = mpsc::channel(1);
            run_thread_dns(rx);
            sender.insert(ShutdownSender(tx));
        }
    } else if status.contains("off") {
        let mut sender = sender.inner().0.lock().await;
        if let Some(tx) = sender.take() {
            tx.0.send(()).await;
        }
    } else {
        info!("Not suppose to be here is {}", status);
    }
    status.to_string()
}

fn run_thread_dns(mut shutdown_receiver: Receiver<()>) {
    let config: Config = serde_yaml::from_str(include_str!("../config.yaml")).unwrap();

    tokio::spawn(async move {
        let server = SimpleDns::try_load(config).await.unwrap();

        tokio::select! {
        Some(_) = shutdown_receiver.recv() => {
                println!("Server asked to shut down...");
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Process asked to shut down...");
            }
            res = server.run() =>  {
                eprintln!("Server exited by itself: {res:?}");
            }
        }
    });
}

struct ShutdownSender(Sender<()>);
#[derive(Debug)]

struct ShutdownReceiver(Receiver<()>);

struct SenderPlease(Mutex<Option<ShutdownSender>>);

#[rocket::main]
async fn main() -> Result<()> {
    let mut config = rocket::Config::default();
    let any_network = Ipv4Addr::new(0, 0, 0, 0);
    config.address = IpAddr::V4(any_network);

    //let (sender, _receiver) = mpsc::channel(1);
    // rocket::ignite().mount("/hello", routes![hello, world]);
    rocket::build()
        .mount("/status", routes![status])
        .manage(SenderPlease(Mutex::new(None)))
        .launch()
        .await;

    Ok(())
}
