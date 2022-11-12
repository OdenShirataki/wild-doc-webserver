use std::net::SocketAddr;
use std::sync::Arc;
use futures_util::future::join;
use hyper::server::conn::{AddrIncoming, Http};
use hyper::service::{make_service_fn, service_fn};
use hyper::Server;
use std::{ io, sync};
use tokio::net::TcpListener;
use rustls::server::ResolvesServerCertUsingSni;

mod tls;
mod request;
pub(crate) mod response;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr_http: SocketAddr = ([127, 0, 0, 1], 1337).into();
    let http_server = async move {
        let listener = TcpListener::bind(addr_http).await.unwrap();
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            tokio::task::spawn(async move {
                if let Err(err)=Http::new().serve_connection(stream, service_fn(request::request)).await{
                    println!("Error serving connection: {:?}", err);
                }
            });
        }
    };

    // Serve an echo service over HTTPS, with proper error handling.
    let addr_https: SocketAddr = ([127, 0, 0, 1], 443).into();
    let https_server = async move {
        let cfg = rustls::ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
        ;
        let cfg=if true{
            cfg.with_single_cert(
                tls::load_certs("certificates/localhost/fullchain.pem").unwrap()
                ,tls::load_private_key("certificates/localhost/privkey.pem").unwrap()
            ).map_err(|e| tls::error(format!("{}", e))).unwrap()
        }else{
            let mut resolver = ResolvesServerCertUsingSni::new();
            tls::add_certificate_to_resolver(
                "localhost", "localhost", &mut resolver
            );
            cfg.with_cert_resolver(Arc::new(resolver))
        };
        // Configure ALPN to accept HTTP/2, HTTP/1.1 in that order.
        //cfg.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        // Create a TCP listener via tokio.
        let incoming = AddrIncoming::bind(&addr_https)?;
        let service = make_service_fn(|_| async { Ok::<_, io::Error>(service_fn(request::request)) });
        Server::builder(tls::TlsAcceptor::new(sync::Arc::new(cfg), incoming)).serve(service).await
    };
    let _ret = join(https_server, http_server).await;
    Ok(())
}
