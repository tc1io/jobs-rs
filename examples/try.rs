use futures::future::{loop_fn, ok, Future, FutureResult, Loop};
use std::io::Error;

struct Client {
    ping_count: u8,
}

impl Client {
    fn new() -> Self {
        Client { ping_count: 0 }
    }

    fn send_ping(self) -> FutureResult<Self, Error> {
        ok(Client {
            ping_count: self.ping_count + 1,
        })
    }

    fn receive_pong(self) -> FutureResult<(Self, bool), Error> {
        let done = self.ping_count >= 5;
        ok((self, done))
    }
}

fn main() {
    let ping_til_done = loop_fn(Client::new(), |client| {
        client
            .send_ping()
            .and_then(|client| client.receive_pong())
            .and_then(|(client, done)| {
                if done {
                    Ok(Loop::Break(client))
                } else {
                    Ok(Loop::Continue(client))
                }
            })
    });
}
