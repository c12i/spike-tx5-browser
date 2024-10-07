use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn init() {
    // Initialize panic hook for better error messages
    console_error_panic_hook::set_once();
}

struct Stat {
    ep: Arc<tx5::Endpoint>,
    event_list: Vec<String>,
}

fn evt_to_json(evt: &tx5::EndpointEvent) -> Result<String, JsValue> {
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
                "message": String::from_utf8_lossy(message),
            })
        }
    })
    .map_err(|e| JsValue::from_str(&e.to_string()))
}

static STAT: Mutex<Option<Stat>> = Mutex::new(None);

#[wasm_bindgen]
pub async fn connect(sig_url: String) -> Result<String, JsValue> {
    let sig_url = tx5::SigUrl::parse(&sig_url).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let config = tx5::Config {
        signal_allow_plain_text: true,
        ..Default::default()
    };
    let (ep, mut ep_rcv) = tx5::Endpoint::new(Arc::new(config));

    wasm_bindgen_futures::spawn_local(async move {
        while let Some(evt) = ep_rcv.recv().await {
            if let Ok(evt_json) = evt_to_json(&evt) {
                if let Some(stat) = &mut *STAT.lock().unwrap() {
                    stat.event_list.push(evt_json);
                }
            }
        }
    });

    let sig_url = ep
        .listen(sig_url)
        .await
        .ok_or_else(|| JsValue::from_str("Connection Failed"))?;

    *STAT.lock().unwrap() = Some(Stat {
        ep: Arc::new(ep),
        event_list: Vec::new(),
    });

    Ok(sig_url.to_string())
}

#[wasm_bindgen]
pub fn get_events() -> Result<Vec<String>, JsValue> {
    let mut out = Vec::new();
    if let Some(stat) = &mut *STAT.lock().unwrap() {
        out.append(&mut stat.event_list);
    }
    Ok(out)
}

#[wasm_bindgen]
pub async fn send(peer_url: String, data: String) -> Result<(), JsValue> {
    let peer_url = tx5::PeerUrl::parse(&peer_url).map_err(|e| JsValue::from_str(&e.to_string()))?;
    let ep = match &*STAT.lock().unwrap() {
        None => return Err(JsValue::from_str("No Connection")),
        Some(stat) => stat.ep.clone(),
    };
    ep.send(peer_url, data.into_bytes())
        .await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(())
}

#[wasm_bindgen]
pub fn close() {
    let _ = STAT.lock().unwrap().take();
}

