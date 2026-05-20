mod cards;
mod structs;
use structs::*;

use std::{convert::Infallible, fs::File, io::BufReader, net::SocketAddr, sync::Arc};

use hyper::service::service_fn;
use hyper::{Body, Method, Request, Response, StatusCode};
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

use crate::cards::all_cards;

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

fn current_timestamp() -> usize {
    use std::time::{SystemTime, UNIX_EPOCH};

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_millis() as usize,
        Err(_) => 0,
    }
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

fn unique_joincode(existing: &std::collections::HashMap<String, LobbyState>) -> String {
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

async fn handle_request(
    req: Request<Body>,
    state: Arc<ServerState>,
) -> Result<Response<Body>, Infallible> {
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
        (&Method::GET, "/gameScreen.js")
        | (&Method::GET, "/lobby.js")
        | (&Method::GET, "/networkFunctions.js")
        | (&Method::GET, "/playCard.js")
        | (&Method::GET, "/chat.js") => {
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
                    lobbies.insert(joincode.clone(), LobbyState::new(username));
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
                        Some(lobby) => {
                            if lobby.players.iter().any(|player| player == &username) {
                                Ok(text_response(
                                    StatusCode::OK,
                                    "no duplicate usernames allowed",
                                ))
                            } else {
                                lobby.players.push(username);
                                Ok(text_response(StatusCode::OK, "joined game lobby"))
                            }
                        }
                        None => Ok(text_response(StatusCode::OK, "game does not exist")),
                    }
                }
                _ => Ok(text_response(
                    StatusCode::BAD_REQUEST,
                    "missing username or joincode",
                )),
            }
        }
        (&Method::POST, "/leaveLobby") => {
            let username = header_value(&req, "username");
            let joincode = header_value(&req, "joincode");
            match (username, joincode) {
                (Some(username), Some(joincode)) => {
                    let mut lobbies = state.lobbies.lock().unwrap();
                    match lobbies.get_mut(&joincode) {
                        Some(lobby) => {
                            if let Some(index) =
                                lobby.players.iter().position(|player| player == &username)
                            {
                                lobby.players.remove(index);
                                if lobby.players.is_empty() {
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
                _ => Ok(text_response(
                    StatusCode::BAD_REQUEST,
                    "missing username or joincode",
                )),
            }
        }
        (&Method::GET, "/lobbyState") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let lobbies = state.lobbies.lock().unwrap();
                    match lobbies.get(&joincode) {
                        Some(lobby) => Ok(json_response(
                            StatusCode::OK,
                            serde_json::to_string(&lobby.players).unwrap(),
                        )),
                        None => Ok(text_response(
                            StatusCode::NOT_FOUND,
                            "lobby not found for the given joincode",
                        )),
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
                    match lobbies.remove(&joincode) {
                        Some(lobby) => {
                            let mut game_state =
                                GameState::new(joincode.clone(), lobby.players.len());
                            game_state.chat_messages = lobby.chat_messages;
                            game_state.start_game();
                            state
                                .games
                                .lock()
                                .unwrap()
                                .insert(joincode.clone(), game_state);
                            state.started_games.lock().unwrap().insert(joincode.clone());
                            Ok(text_response(StatusCode::OK, "Game Started"))
                        }
                        None => Ok(text_response(StatusCode::NOT_FOUND, "game does not exist")),
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
                    Ok(text_response(
                        StatusCode::OK,
                        if started { "yes" } else { "no" },
                    ))
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::POST, "/endTurn") => {
            let joincode: Option<String> = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let mut games = state.games.lock().unwrap();
                    match games.get_mut(&joincode) {
                        Some(game) => {
                            game.next_turn();
                            let body = serde_json::to_string(&serde_json::json!({"current_turn_player": game.current_turn_player})).unwrap();
                            Ok(json_response(StatusCode::OK, body))
                        }
                        None => Ok(text_response(
                            StatusCode::NOT_FOUND,
                            "game not found for joincode",
                        )),
                    }
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::GET, "/gameState") => {
            let joincode: Option<String> = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    let mut games = state.games.lock().unwrap();
                    match games.get_mut(&joincode) {
                        Some(game) => {
                            let body = serde_json::to_string(game).unwrap();
                            Ok(json_response(StatusCode::OK, body))
                        }
                        None => Ok(text_response(
                            StatusCode::NOT_FOUND,
                            "game not found for joincode",
                        )),
                    }
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::POST, "/sendChat") => {
            let joincode = header_value(&req, "joincode");
            let sender = header_value(&req, "username").unwrap_or_else(|| "Anonymous".to_string());
            let timestamp = current_timestamp();
            let message = header_value(&req, "text");

            match (joincode, message) {
                (Some(joincode), Some(message)) => {
                    let chat_message = ChatMessage {
                        sender,
                        message,
                        timestamp,
                    };

                    {
                        let mut games = state.games.lock().unwrap();
                        if let Some(game) = games.get_mut(&joincode) {
                            game.chat_messages.push(chat_message);
                            return Ok(text_response(StatusCode::OK, "sent"));
                        }
                    }

                    {
                        let mut lobbies = state.lobbies.lock().unwrap();
                        if let Some(lobby) = lobbies.get_mut(&joincode) {
                            lobby.chat_messages.push(chat_message);
                            return Ok(text_response(StatusCode::OK, "sent"));
                        }
                    }

                    Ok(text_response(
                        StatusCode::NOT_FOUND,
                        "lobby/game not found for joincode",
                    ))
                }
                (None, _) => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
                (_, None) => Ok(text_response(StatusCode::BAD_REQUEST, "missing text")),
            }
        }
        (&Method::GET, "/getChat") => {
            let joincode = header_value(&req, "joincode");
            match joincode {
                Some(joincode) => {
                    {
                        let games = state.games.lock().unwrap();
                        if let Some(game) = games.get(&joincode) {
                            let body = serde_json::to_string(&game.chat_messages).unwrap();
                            return Ok(json_response(StatusCode::OK, body));
                        }
                    }

                    {
                        let lobbies = state.lobbies.lock().unwrap();
                        if let Some(lobby) = lobbies.get(&joincode) {
                            let body = serde_json::to_string(&lobby.chat_messages).unwrap();
                            return Ok(json_response(StatusCode::OK, body));
                        }
                    }

                    Ok(text_response(
                        StatusCode::NOT_FOUND,
                        "lobby/game not found for joincode",
                    ))
                }
                None => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
            }
        }
        (&Method::GET, "/getDeck") => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&all_cards()).unwrap()))
            .unwrap()),
        (&Method::POST, "/quitGame") => {
            let joincode = header_value(&req, "joincode");
            let username = header_value(&req, "username");
            match (joincode, username) {
                (Some(joincode), Some(username)) => {
                    let mut games = state.games.lock().unwrap();
                    match games.get_mut(&joincode) {
                        Some(game) => {
                            // Find the quitting player
                            if let Some(player_index) = game.players.iter().position(|p| p.name == username) {
                                let quitting_player_id = game.players[player_index].id;
                                let was_their_turn = game.current_turn_player == player_index;
                                
                                // 1. Remove their active firing filters
                                game.active_cards.active_firing_filters.retain(|f| f.owner != quitting_player_id);
                                
                                // 2. Remove their calculated shootings
                                game.active_cards.active_calculated_shootings.retain(|s| s.owner != quitting_player_id);
                                
                                // 3. If they played no shooting, transfer to next player
                                if game.no_shooting_played_by == player_index as isize {
                                    let next_player_index = (player_index + 1) % game.players.len();
                                    game.no_shooting_played_by = next_player_index as isize;
                                }
                                
                                // 4. Move their hand to the discard pile
                                let hand = std::mem::take(&mut game.players[player_index].hand);
                                game.discard_pile.extend(hand);
                                
                                // 5. Remove the player from the game
                                game.players.remove(player_index);
                                
                                // 6. Adjust indices after removal
                                if game.no_shooting_played_by as usize > player_index {
                                    game.no_shooting_played_by -= 1;
                                }
                                if game.current_turn_player > player_index {
                                    game.current_turn_player -= 1;
                                } else if game.current_turn_player == player_index && !game.players.is_empty() {
                                    game.current_turn_player = game.current_turn_player % game.players.len();
                                }
                                
                                // 7. If it was their turn, advance to the next player's turn
                                if was_their_turn && !game.players.is_empty() {
                                    game.next_turn();
                                }
                                
                                // 8. If no players left, delete the game
                                if game.players.is_empty() {
                                    games.remove(&joincode);
                                }
                                
                                return Ok(text_response(StatusCode::OK, "quit"));
                            } else {
                                return Ok(text_response(StatusCode::NOT_FOUND, "player not found in game"));
                            }
                        }
                        None => Ok(text_response(
                            StatusCode::NOT_FOUND,
                            "game not found for joincode",
                        )),
                    }
                }
                (None, _) => Ok(text_response(StatusCode::BAD_REQUEST, "missing joincode")),
                (_, None) => Ok(text_response(StatusCode::BAD_REQUEST, "missing username")),
            }
        }
        (&Method::POST, "/playCard")
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
    // Load certificates from the directory containing the running binary
    let exe_path = std::env::current_exe().expect("cannot determine current exe path");
    let exe_dir = exe_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));

    // Determine base directory for certificates. If the binary appears to be
    // inside a `target` directory (typical when running via `cargo run`),
    // look two folders up (repo root). Otherwise use the binary directory.
    let mut cert_base = exe_dir.clone();
    if exe_dir.iter().any(|comp| comp == "target") {
        if let Some(two_up) = exe_dir.parent().and_then(|p| p.parent()) {
            cert_base = two_up.to_path_buf();
            eprintln!("Running from cargo; looking for certs at {}", cert_base.display());
        }
    }

    let cert_path = cert_base.join("certificate.crt");
    let key_path = cert_base.join("private.key");
    let ca_path = cert_base.join("ca_bundle.crt");

    // Require cert and key to exist at the chosen location; fail otherwise.
    if !cert_path.exists() || !key_path.exists() {
        eprintln!("Certificate or key not found in {}.\nPlace certificate.crt and private.key in that directory.", cert_base.display());
        return Err("certificate files not found".into());
    }

    // Load leaf cert(s)
    let mut certs = load_certs(
        cert_path
            .to_str()
            .expect("certificate path is not valid UTF-8"),
    );
    // If a CA bundle exists, append its certs to form a chain
    if ca_path.exists() {
        let mut ca_certs = load_certs(
            ca_path
                .to_str()
                .expect("ca bundle path is not valid UTF-8"),
        );
        certs.append(&mut ca_certs);
    }

    let key = load_private_key(
        key_path
            .to_str()
            .expect("private key path is not valid UTF-8"),
    );

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
