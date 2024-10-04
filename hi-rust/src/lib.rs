use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;

#[napi::module_init]
fn init() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    create_custom_tokio_runtime(rt);
}

struct Stat {
    ep: Arc<tx5::Endpoint>,
    rcv_task: tokio::task::JoinHandle<()>,
    event_list: Vec<String>,
}

fn evt_to_json(evt: &tx5::EndpointEvent) -> Result<String> {
    serde_json::to_string_pretty(&match evt {
        tx5::EndpointEvent::ListeningAddressOpen { local_url } => {
            serde_json::json!({
                "type": "ListeningAddressOpen",
                "local_url": local_url,
            })
        }
        tx5::EndpointEvent::ListeningAddressClosed { local_url } => {
            serde_json::json!({
                "type": "ListeningAddressOpen",
                "local_url": local_url,
            })
        }
        tx5::EndpointEvent::Connected { peer_url } => {
            serde_json::json!({
                "type": "Connected",
                "peer_url": peer_url,
            })
        }
        tx5::EndpointEvent::Disconnected { peer_url } => {
            serde_json::json!({
                "type": "Disconnected",
                "peer_url": peer_url,
            })
        }
        tx5::EndpointEvent::Message { peer_url, message } => {
            serde_json::json!({
                "type": "Message",
                "peer_url": peer_url,
                "message": String::from_utf8_lossy(&message),
            })
        }
    })
    .map_err(|e| std::io::Error::other(e).into())
}

static STAT: Mutex<Option<Stat>> = Mutex::new(None);

#[napi_derive::napi]
pub async fn connect(sig_url: String) -> Result<String> {
    let sig_url = tx5::SigUrl::parse(sig_url)?;

    let config = tx5::Config {
        signal_allow_plain_text: true,
        ..Default::default()
    };

    let (ep, mut ep_rcv) = tx5::Endpoint::new(Arc::new(config));

    let rcv_task = tokio::task::spawn(async move {
        while let Some(evt) = ep_rcv.recv().await {
            // TODO - remove this unwrap
            let evt = evt_to_json(&evt).unwrap();
            if let Some(stat) = &mut *STAT.lock().unwrap() {
                stat.event_list.push(evt);
            }
        }
    });

    let sig_url = match ep.listen(sig_url).await {
        None => return Err(std::io::Error::other("Connection Failed").into()),
        Some(sig_url) => sig_url,
    };

    *STAT.lock().unwrap() = Some(Stat {
        ep: Arc::new(ep),
        rcv_task,
        event_list: Vec::new(),
    });

    Ok(sig_url.to_string())
}

#[napi_derive::napi]
pub async fn get_events() -> Result<Vec<String>> {
    let mut out = Vec::new();
    if let Some(stat) = &mut *STAT.lock().unwrap() {
        out.append(&mut stat.event_list);
    }
    Ok(out)
}

#[napi_derive::napi]
pub async fn send(peer_url: String, data: String) -> Result<()> {
    let peer_url = tx5::PeerUrl::parse(peer_url)?;
    let ep = match &*STAT.lock().unwrap() {
        None => return Err(std::io::Error::other("No Connection").into()),
        Some(stat) => stat.ep.clone(),
    };

    ep.send(peer_url, data.into_bytes()).await?;

    Ok(())
}

#[napi_derive::napi]
pub async fn close() {
    if let Some(stat) = STAT.lock().unwrap().take() {
        stat.rcv_task.abort();
    }
}
