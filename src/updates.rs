use actix_rt::Arbiter;
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Bytes, Data};
use actix_web::{web, HttpResponse, Responder};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tokio::prelude::*;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::timer::Interval;
use uuid::Uuid;

use crate::document::Document;
use crate::user::AuthorizedUser;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/updates")
            .service(web::resource("").route(web::get().to(register_client))),
    );
}

fn register_client(updates: Data<Mutex<ClientUpdates>>, login: AuthorizedUser) -> impl Responder {
    let rx = updates.lock().unwrap().new_client(login);

    HttpResponse::Ok()
        .header("content-type", "text/event-stream")
        .no_chunking()
        .streaming(rx.map_err(ErrorInternalServerError))
}

pub struct ClientUpdates {
    clients: Vec<Client>,
}

impl ClientUpdates {
    pub fn create() -> Data<Mutex<Self>> {
        // Data ≈ Arc
        let me = Data::new(Mutex::new(Self::new()));

        // ping clients every 10 seconds to see if they are alive
        Self::spawn(me.clone());

        me
    }

    fn new() -> Self {
        ClientUpdates {
            clients: Vec::new(),
        }
    }

    fn spawn(me: Data<Mutex<Self>>) {
        let ping = Interval::new(Instant::now(), Duration::from_secs(10))
            .for_each(move |_| {
                me.lock().unwrap().remove_stale_clients();
                Ok(())
            })
            .map_err(|e| panic!("ping interval errored; err={:?}", e));

        Arbiter::spawn(ping);
    }

    fn remove_stale_clients(&mut self) {
        let mut ok_clients = Vec::new();

        while let Some(client) = self.clients.pop() {
            let result = client.tx.clone().try_send(Bytes::from("data: ping\n\n"));

            if let Ok(()) = result {
                ok_clients.push(client);
            }
        }

        self.clients = ok_clients;
    }

    pub fn new_client(&mut self, user: AuthorizedUser) -> Receiver<Bytes> {
        let (client, rx) = Client::new(user);
        self.clients.push(client);
        rx
    }

    pub fn inserted(&self, document: &Document, login: &AuthorizedUser) {
        self.msg(b"insert", serde_json::to_vec(document).unwrap(), login)
    }

    pub fn updated(&self, document: &Document, login: &AuthorizedUser) {
        self.msg(b"update", serde_json::to_vec(document).unwrap(), login)
    }

    pub fn deleted(&self, id: &str, login: &AuthorizedUser) {
        self.msg(b"delete", id.into(), login)
    }

    fn msg(&self, event_type: &[u8], data: Vec<u8>, login: &AuthorizedUser) {
        let message = Bytes::from([b"event: ", event_type, b"\ndata: ", &data, b"\n\n"].concat());

        for client in self.clients.iter() {
            if is_same_user_on_different_session(client, login) {
                client.tx.clone().try_send(message.clone()).unwrap_or(());
            }
        }
    }
}

struct Client {
    username: String,
    uuid: Uuid,
    tx: Sender<Bytes>,
}

impl Client {
    fn new(user: AuthorizedUser) -> (Self, Receiver<Bytes>) {
        let (tx, rx) = channel(100);
        let username = user.username;
        let uuid = user.uuid;
        (Self { username, uuid, tx }, rx)
    }
}

fn is_same_user_on_different_session(c: &Client, b: &AuthorizedUser) -> bool {
    c.username == b.username && c.uuid != b.uuid
}
