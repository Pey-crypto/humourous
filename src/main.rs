use actix::{Actor, StreamHandler, Addr, Context, Message, Handler, Recipient,AsyncContext};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use std::collections::HashMap;
use uuid::Uuid;

use actix_rt::time::interval;

struct WebSocket {
    id: Uuid,
    server: Addr<Server>,
}

impl Actor for WebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.server.do_send(Connect {
            id: self.id,
            addr: addr.recipient(),
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        self.server.do_send(Disconnect { id: self.id });
        actix::Running::Stop
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                self.server.do_send(ClientMessage {
                    id: self.id,
                    msg: text,
                })
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

struct Server {
    sessions: HashMap<Uuid, Recipient<MyWsMessage>>,
}

impl Actor for Server {
    type Context = Context<Self>;
}

struct Connect {
    id: Uuid,
    addr: Recipient<MyWsMessage>,
}

impl Message for Connect {
    type Result = ();
}

impl Handler<Connect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.insert(msg.id, msg.addr);
    }
}

struct Disconnect {
    id: Uuid,
}

impl Message for Disconnect {
    type Result = ();
}

impl Handler<Disconnect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        println!("Client {} disconnected", msg.id);
        self.sessions.remove(&msg.id);
    }
}

struct ClientMessage {
    id: Uuid,
    msg: String,
}

impl Message for ClientMessage {
    type Result = ();
}

impl Handler<ClientMessage> for Server {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        for (id, addr) in &self.sessions {
            if *id != msg.id {
                let message_with_id = format!("{}: {}", msg.id, msg.msg);
                let _ = addr.do_send(MyWsMessage(ws::Message::Text(message_with_id)));
            }
        }
    }
}
impl Handler<MyWsMessage> for WebSocket {
    type Result = ();

    fn handle(&mut self, msg: MyWsMessage, ctx: &mut Self::Context) {
        match msg.0 {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Binary(bin) => ctx.binary(bin),
            _ => (),
        }
    }
}


pub struct MyWsMessage(pub ws::Message);

impl Message for MyWsMessage {
    type Result = ();
}

struct ListClients;

impl Message for ListClients {
    type Result = ();
}

impl Handler<ListClients> for Server {
    type Result = ();

    fn handle(&mut self, _: ListClients, _: &mut Context<Self>) {
        println!("Connected clients: {:?}", self.sessions.keys().collect::<Vec<_>>());
    }
}


async fn chat_route(req: HttpRequest, stream: web::Payload, srv: web::Data<Addr<Server>>) -> Result<HttpResponse, Error> {
    let resp = ws::start(WebSocket { id: Uuid::new_v4(), server: srv.get_ref().clone() }, &req, stream);
    resp
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let server = Server { sessions: HashMap::new() }.start();

    let server_addr = server.clone();
    actix::spawn(async move {
        let mut interval = interval(std::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            server_addr.do_send(ListClients);
        }
    });

    HttpServer::new(move || App::new().data(server.clone()).route("/ws/", web::get().to(chat_route)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

