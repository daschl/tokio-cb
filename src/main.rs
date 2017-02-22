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
        self.db
            .get("airline_10226") // Fetch the document
            .map_err(|e| e.into()) // Turn a CouchbaseError into an io::Error
            .map(|doc| {
                let mut response = Response::new();
                response.header("Content-Type", "application/json");
                response.body(doc.content_as_str().unwrap()); // serve the json content directly
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
