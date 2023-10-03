use std::io::Read;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::vec::Vec;
use std::{fs, io};

use bytebuffer::{ByteBuffer, Endian};
use color_eyre::{eyre::eyre, Result};
use http::uri::{Authority, Scheme};
use http::{HeaderValue, Method};
use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn, Service};
use hyper::{Body, Client, Request, Response, Server, StatusCode, Uri};
use hyper_rustls::{acceptor::TlsStream, ConfigBuilderExt, TlsAcceptor};
use tokio::sync::Mutex;
use tracing::{info, warn};

mod bancho;

use crate::preferences::{BeatmapMirror, Preferences};
use bancho::{BanchoPacket, BanchoPacketHeader};

const SUBDOMAINS: &[&str] = &["c", "ce", "c4", "osu", "b", "api"];

const SOURCE_DOMAIN: &str = "osus.zihad.dev";
const DEFAULT_TARGET_DOMAIN: &str = "osu.ppy.sh";

pub async fn start(preferences: Arc<Mutex<Preferences>>) -> Result<()> {
    let addr = ([127, 0, 0, 1], 8000).into();

    let certs = load_certs("./server.crt")?;
    let key = load_private_key("./server.key")?;

    let incoming = AddrIncoming::bind(&addr)?;
    let acceptor = TlsAcceptor::builder()
        .with_single_cert(certs, key)
        .map_err(|e| eyre!("{}", e))?
        .with_http11_alpn()
        // .with_all_versions_alpn()
        .with_incoming(incoming);

    let make_svc = make_service_fn(|conn: &TlsStream| {
        let remote_addr = conn.io().map(|x| x.remote_addr());
        let mut inner_svc = service_fn(handle_requests);

        let preferences_clone = preferences.clone();
        let outer_svc = service_fn(move |mut req: Request<Body>| {
            req.extensions_mut().insert(preferences_clone.clone());

            if let Some(remote_addr) = remote_addr {
                req.extensions_mut().insert(remote_addr);
            }

            inner_svc.call(req)
        });

        async move { Ok::<_, String>(outer_svc) }
    });

    let server = Server::builder(acceptor).serve(make_svc);

    println!("Starting to serve on https://{}.", addr);

    server.await?;

    Ok(())
}

async fn handle_requests(mut req: Request<Body>) -> Result<Response<Body>> {
    let Some(host) = req.headers().get("Host").and_then(|x| x.to_str().ok()).map(|x| x.to_owned()) else {
        let mut response = Response::new(Body::from("host header not found"));
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Ok(response);
    };
    let Some((subdomain, _)) = SUBDOMAINS
        .iter()
        .map(|&subdomain| subdomain.to_owned())
        .map(|subdomain| {
            (
                subdomain.clone(),
                subdomain + &format!(".{}", SOURCE_DOMAIN),
            )
        })
        .find(|(_subdomain, full_source_host)| full_source_host == &host)
        else {
            let mut response = Response::new(Body::from(format!(
                "target domain for host {} not found",
                host
            )));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            return Ok(response);
        };
    let target_host = {
        let target_domain = if let Some(preferences) = req.extensions()
            .get::<Arc<Mutex<Preferences>>>() {
            let preferences = preferences.lock().await;
            preferences.server_address.clone()
        } else {
            DEFAULT_TARGET_DOMAIN.to_owned()
        };
        subdomain + &format!(".{}", &target_domain)
    };

    let mut uri_parts = req.uri().clone().into_parts();
    uri_parts.scheme.get_or_insert(Scheme::HTTPS);
    uri_parts.authority = Some(Authority::from_str(&target_host).unwrap());
    let mut new_uri = Uri::from_parts(uri_parts).unwrap();
    std::mem::swap(req.uri_mut(), &mut new_uri);

    let client_ip_addr = req
        .extensions()
        .get::<SocketAddr>()
        .map(|x| x.ip().to_string())
        .unwrap_or_else(String::new);

    let headers = req.headers_mut();
    headers.insert(
        "X-Forwarded-For",
        HeaderValue::from_str(&client_ip_addr).unwrap(),
    );
    headers.insert("X-Real-IP", HeaderValue::from_str(&client_ip_addr).unwrap());
    headers.insert("Host", HeaderValue::from_str(&target_host).unwrap());
    dbg!(&headers);

    let tls = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_native_roots()
        .with_no_client_auth();
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls)
        .https_or_http()
        .enable_http1()
        .build();

    let client = Client::builder().build(https);

    let req_path = req.uri().path().to_owned();
    let req_method = req.method().clone();
    let preferences = req.extensions().get::<Arc<Mutex<Preferences>>>().map(|x| x.clone());

    match client.request(req).await {
        Ok(mut response) => {
            if let Some(preferences) = preferences {
                if req_path == "/" && req_method == Method::POST {
                    let (parts, body) = response.into_parts();
                    let body_bytes = hyper::body::to_bytes(body).await.unwrap();
                    let mut packets = decode_bancho_packets(body_bytes.as_ref()).await.unwrap();
                    let preferences = preferences.lock().await;
                    process_bancho_packets(&preferences, &mut packets).await;
                    let body_bytes = encode_bancho_packets(packets).await.unwrap();
                    response = Response::from_parts(parts, Body::from(body_bytes));
                } else if host == "osu.".to_owned() + &*SOURCE_DOMAIN && req_method == Method::GET {
                    if req_path.starts_with("/d/") {
                        if let Ok(id) = req_path.replace("/d/", "").replace('n', "").parse::<u32>() {
                            let preferences = preferences.lock().await;
                            match &preferences.beatmap_mirror {
                                BeatmapMirror::ServerDefault => {}
                                mirror => {
                                    let link = mirror.direct_download_link(id, false);
                                    info!("Redirecting download request for beatmap set {} to {}", id, mirror);
                                    response = Response::builder()
                                        .status(StatusCode::FOUND)
                                        .header("Location", link)
                                        .body(Body::empty())
                                        .unwrap()
                                }
                            }
                        }
                    }
                }
            }
            Ok(response)
        }
        Err(err) => {
            let mut response = Response::new(Body::from(format!("error fetching: {}", err)));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(response)
        }
    }
}

async fn decode_bancho_packets(bytes: &[u8]) -> io::Result<Vec<BanchoPacket>> {
    let mut packets = vec![];

    let mut bytebuf = ByteBuffer::from_bytes(bytes);
    bytebuf.set_endian(Endian::LittleEndian);

    loop {
        let remaining_bytes = bytebuf.len() - bytebuf.get_rpos();
        if remaining_bytes == 0 {
            break;
        } else if remaining_bytes < 7 {
            let leftover = bytebuf.read_bytes(remaining_bytes)?;
            warn!("Encountered {remaining_bytes} leftover bytes:");
            rhexdump::rhexdump!(&leftover);
            break;
        } else {
            let mut header_bytes = [0; 7];
            bytebuf.read_exact(&mut header_bytes)?;
            let header = BanchoPacketHeader::from_bytes(header_bytes)?;
            let packet = BanchoPacket::from_header_and_bytebuf(&header, &mut bytebuf)?;
            packets.push(packet);
        }
    }

    Ok(packets)
}

async fn process_bancho_packets(preferences: &Preferences, packets: &mut Vec<BanchoPacket>) {
    for packet in packets {
        match packet {
            BanchoPacket::Privilege {
                privileges_bitfield,
            } => {
                if preferences.fake_supporter {
                    // Add supporter if does not already exist
                    *privileges_bitfield = *privileges_bitfield | (1 << 2);

                    // Remove supporter if exists, to test with local bancho.py or cmyui.xyz since those give supporter by default
                    // *privileges_bitfield = *privileges_bitfield & !(1 << 2);
                }
            }
            _ => {}
        }
    }
}

async fn encode_bancho_packets(packets: Vec<BanchoPacket>) -> io::Result<Vec<u8>> {
    let mut bytes = vec![];
    for packet in packets {
        bytes.append(&mut packet.to_bytes());
    }

    Ok(bytes)
}

fn load_certs(filename: &str) -> Result<Vec<rustls::Certificate>> {
    let certfile =
        fs::File::open(filename).map_err(|e| eyre!("failed to open {}: {}", filename, e))?;
    let mut reader = io::BufReader::new(certfile);

    let certs =
        rustls_pemfile::certs(&mut reader).map_err(|_| eyre!("failed to load certificate"))?;
    Ok(certs.into_iter().map(rustls::Certificate).collect())
}

fn load_private_key(filename: &str) -> Result<rustls::PrivateKey> {
    let keyfile =
        fs::File::open(filename).map_err(|e| eyre!("failed to open {}: {}", filename, e))?;
    let mut reader = io::BufReader::new(keyfile);

    let keys = rustls_pemfile::rsa_private_keys(&mut reader)
        .map_err(|_| eyre!("failed to load private key"))?;
    if keys.len() != 1 {
        return Err(eyre!("expected a single private key"));
    }

    Ok(rustls::PrivateKey(keys[0].clone()))
}
