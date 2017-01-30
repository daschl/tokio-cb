// basic dependencies from echo server before
extern crate futures;
extern crate tokio_proto;
extern crate tokio_service;

// our toy HTTP implementation
extern crate tokio_minihttp;

// database support
extern crate couchbase;

// misc support for random numbers, and json
extern crate rand;
extern crate rustc_serialize;

use std::io;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use futures::{BoxFuture, Future};
use couchbase::{Cluster, Bucket};
use tokio_proto::TcpServer;
use tokio_minihttp::{Request, Response};
use tokio_service::Service;

struct Server {
    db: Arc<Bucket>,
}

impl Service for Server {
    type Request = Request;
    type Response = Response;
    type Error = io::Error;
    type Future = BoxFuture<Response, io::Error>;

    fn call(&self, _: Request) -> Self::Future {
        let doc = self.db
            .get("airline_10226")
            .map_err(|e| Error::new(ErrorKind::Other, format!("{:?}", e)));

        let content = doc.map(|d| {
            match d {
                    Some(ref d) => d.content_as_str().unwrap(),
                    None => "{\"not found\": true}",
                }
                .to_owned()
        });

        content.map(|c| {
                let mut response = Response::new();
                response.header("Content-Type", "application/json");
                response.body(&c);
                response
            })
            .boxed()
    }
}

fn main() {
    let addr = "127.0.0.1:8080".parse().unwrap();

    let cluster = Cluster::new("127.0.0.1").unwrap();
    let db = Arc::new(cluster.open_bucket("travel-sample", "").expect("Could not open bucket!"));

    TcpServer::new(tokio_minihttp::Http, addr).serve(move || Ok(Server { db: db.clone() }))
}
