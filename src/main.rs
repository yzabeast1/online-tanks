mod structs;
mod cards;
use structs::*;

use std::{convert::Infallible, fs::File, io::BufReader, net::SocketAddr, sync::Arc};

use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, StatusCode};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

const JS_TANKS_SERVER_DIR: &str = "/home/yehoshua/git-repos/js-tanks/server";
const JS_TANKS_WEBSITE_DIR: &str = "/home/yehoshua/git-repos/js-tanks/website";

async fn serve_file(path: &str, content_type: &str) -> Result<Response<Body>, Infallible> {
    match tokio::fs::read(path).await {
        Ok(bytes) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", content_type)
            .body(Body::from(bytes))
            .unwrap()),
        Err(_) => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap()),
    }
}

fn header_value(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.to_string())
}

fn text_response(status: StatusCode, body: impl Into<Body>) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("content-type", "text/plain; charset=utf-8")
        .header("access-control-allow-origin", "*")
        .header("access-control-expose-headers", "joincode")
        .body(body.into())
        .unwrap()
}

fn json_response(status: StatusCode, body: String) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .header("access-control-allow-origin", "*")
        .body(Body::from(body))
        .unwrap()
}

fn generate_joincode() -> String {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let mut code = String::with_capacity(6);
    for _ in 0..6 {
        let letter = (b'A' + rng.gen_range(0..26)) as char;
        code.push(letter);
    }
    code
}

fn unique_joincode(existing: &std::collections::HashMap<String, Vec<String>>) -> String {
    loop {
        let code = generate_joincode();
        if !existing.contains_key(&code) {
            return code;
        }
    }
}

fn load_certs(path: &str) -> Vec<Certificate> {
    let certfile = File::open(path).expect("cannot open certificate");
    let mut reader = BufReader::new(certfile);
    certs(&mut reader)
        .expect("failed to read certificates")
        .into_iter()
        .map(Certificate)
        .collect()
}

fn load_private_key(path: &str) -> PrivateKey {
    let keyfile = File::open(path).expect("cannot open private key");
    let mut reader = BufReader::new(keyfile);

    // Try pkcs8 first
    let pkcs8 = pkcs8_private_keys(&mut reader).expect("cannot parse pkcs8 private key");
    if !pkcs8.is_empty() {
        return PrivateKey(pkcs8[0].clone());
    }

    // Re-open and try rsa keys
    let keyfile = File::open(path).expect("cannot open private key");
    let mut reader = BufReader::new(keyfile);
    let rsa = rsa_private_keys(&mut reader).expect("cannot parse rsa private key");
    if !rsa.is_empty() {
        return PrivateKey(rsa[0].clone());
    }

    panic!("no private keys found in {}", path);
}
async fn handle_request(req: Request<Body>, state: Arc<ServerState>) -> Result<Response<Body>, Infallible> {
    let path = req.uri().path().to_string();
    match (req.method(), path.as_str()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => {
            let p = format!("{}/index.html", JS_TANKS_WEBSITE_DIR);
            serve_file(&p, "text/html; charset=utf-8").await
        }
        (&Method::GET, "/css.css") => {
            let p = format!("{}/css.css", JS_TANKS_WEBSITE_DIR);
            serve_file(&p, "text/css; charset=utf-8").await
        }
        (&Method::GET, "/gameScreen.js") | (&Method::GET, "/lobby.js") | (&Method::GET, "/networkFunctions.js") | (&Method::GET, "/playCard.js") | (&Method::GET, "/chat.js") => {
            let p = format!("{}/{}", JS_TANKS_WEBSITE_DIR, &path[1..]);
            serve_file(&p, "application/javascript").await
        }
        (&Method::GET, "/checkOnline") => Ok(text_response(StatusCode::OK, "Online")),
        (&Method::POST, "/createGame") => {
            let username = header_value(&req, "username");
            match username {
                Some(username) => {
                    let mut lobbies = state.lobbies.lock().unwrap();
                    let joincode = unique_joincode(&lobbies);
                    lobbies.insert(joincode.clone(), vec![username]);
                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("content-type", "text/plain; charset=utf-8")
                        .header("access-control-allow-origin", "*")
                        .header("access-control-expose-headers", "joincode")
                        .header("joincode", joincode.clone())
                        .body(Body::from(joincode))
                        .unwrap())
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing username")),
            }
        }
        (&Method::POST, "/joinGame") => {
            let username = header_value(&req, "username");
            let joincode = header_value(&req, "joincode");
            match (username, joincode) {
                (Some(username), Some(joincode)) => {
                    let mut lobbies = state.lobbies.lock().unwrap();
                    match lobbies.get_mut(&joincode) {
                        Some(players) => {
                            if players.iter().any(|player| player == &username) {
                                Ok(text_response(StatusCode::OK, "no duplicate usernames allowed"))
                            } else {
                                players.push(username);
                                Ok(text_response(StatusCode::OK, "joined game lobby"))
                            }
                        }
                        None => Ok(text_response(StatusCode::OK, "game does not exist")),
                    }
                }
                _ => Ok(text_response(StatusCode::BAD_REQUEST, "missing username or joincode")),
            }
        }
        (&Method::POST, "/leaveLobby") => {
            let username = header_value(&req, "username");
            let joincode = header_value(&req, "joincode");
            match (username, joincode) {
                (Some(username), Some(joincode)) => {
                    let mut lobbies = state.lobbies.lock().unwrap();
                    match lobbies.get_mut(&joincode) {
                        Some(players) => {
                            if let Some(index) = players.iter().position(|player| player == &username) {
                                players.remove(index);
                                if players.is_empty() {
                                    lobbies.remove(&joincode);
                                }
                                Ok(text_response(StatusCode::OK, "left game lobby"))
                            } else {
                                Ok(text_response(StatusCode::OK, "not in lobby"))
                            }
                        }
                        None => Ok(text_response(StatusCode::OK, "game does not exist")),
                    }
                }
                _ => Ok(text_response(StatusCode::BAD_REQUEST, "missing username or joincode")),
            }
        }
        (&Method::GET, "/lobbyState") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let lobbies = state.lobbies.lock().unwrap();
                    match lobbies.get(&joincode) {
                        Some(players) => Ok(json_response(StatusCode::OK, serde_json::to_string(players).unwrap())),
                        None => Ok(text_response(StatusCode::NOT_FOUND, "lobby not found for the given joincode")),
                    }
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::POST, "/startGame") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let mut lobbies = state.lobbies.lock().unwrap();
                    if lobbies.remove(&joincode).is_some() {
                        state.started_games.lock().unwrap().insert(joincode);
                        Ok(text_response(StatusCode::OK, "Game Started"))
                    } else {
                        Ok(text_response(StatusCode::NOT_FOUND, "game does not exist"))
                    }
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::GET, "/checkGameStarted") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let started = state.started_games.lock().unwrap().contains(&joincode);
                    Ok(text_response(StatusCode::OK, if started { "yes" } else { "no" }))
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        // Keep the rest as placeholders for now.
        (&Method::GET, "/cardInfo")
        | (&Method::GET, "/gameState")
        | (&Method::GET, "/getChat")
        | (&Method::GET, "/getDeck") => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap()),
        (&Method::POST, "/endTurn")
        | (&Method::POST, "/playCard")
        | (&Method::POST, "/quitGame")
        | (&Method::POST, "/sendChat")
        | (&Method::POST, "/activateDelayedCard")
        | (&Method::POST, "/useRadar") => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap()),
        _ => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Not found"))
            .unwrap()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load certificates from the js-tanks server folder (same as the node server)
    let cert_path = format!("{}/certificate.crt", JS_TANKS_SERVER_DIR);
    let key_path = format!("{}/private.key", JS_TANKS_SERVER_DIR);
    let ca_path = format!("{}/ca_bundle.crt", JS_TANKS_SERVER_DIR);

    // Load leaf cert(s)
    let mut certs = load_certs(&cert_path);
    // If a CA bundle exists, append its certs to form a chain
    if std::path::Path::new(&ca_path).exists() {
        let mut ca_certs = load_certs(&ca_path);
        certs.append(&mut ca_certs);
    }

    let key = load_private_key(&key_path);

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("bad certificates/private key");

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let state = Arc::new(ServerState::new());

    let addr: SocketAddr = ([0, 0, 0, 0], 8443).into();
    let listener = TcpListener::bind(addr).await?;

    println!("HTTPS server listening on https://0.0.0.0:8443");

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let state = state.clone();

        tokio::spawn(async move {
            let peer = peer_addr;
            match acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    let service = service_fn(move |req| handle_request(req, state.clone()));
                    if let Err(err) = hyper::server::conn::Http::new()
                        .serve_connection(tls_stream, service)
                        .await
                    {
                        eprintln!("Error serving connection from {}: {}", peer, err);
                    }
                }
                Err(err) => {
                    eprintln!("TLS handshake failed for {}: {}", peer, err);
                }
            }
        });
    }
}
