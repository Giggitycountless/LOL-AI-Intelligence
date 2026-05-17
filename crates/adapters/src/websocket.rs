use std::{fmt, pin::Pin, task::{Context, Poll}};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use futures_util::{SinkExt, Stream, StreamExt};
use serde_json::Value;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    Connector, MaybeTlsStream, WebSocketStream,
    tungstenite::{self, Message, client::IntoClientRequest, http::HeaderValue},
};

use crate::constants::LOCAL_LCU_HOST;
use crate::lockfile::LockfileCredentials;
use crate::log_lcu_adapter_event;

#[derive(Debug, Clone, PartialEq)]
pub struct LcuWebSocketEvent {
    pub uri: String,
    pub event_type: String,
    pub data: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcuSubscription {
    JsonApiEvent(&'static str),
}

impl fmt::Display for LcuSubscription {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::JsonApiEvent(path) => write!(
                formatter,
                "OnJsonApiEvent_{}",
                path.trim_start_matches('/').replace('/', "_")
            ),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LcuWebSocketError {
    Unavailable,
    Authentication,
    Disconnected,
    Send,
    Unexpected,
}

impl fmt::Debug for LcuWebSocketError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Unavailable => "Unavailable",
            Self::Authentication => "Authentication",
            Self::Disconnected => "Disconnected",
            Self::Send => "Send",
            Self::Unexpected => "Unexpected",
        };
        formatter
            .debug_tuple("LcuWebSocketError")
            .field(&label)
            .finish()
    }
}

pub struct LcuWebSocketClient {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl LcuWebSocketClient {
    pub(crate) async fn connect(credentials: LockfileCredentials) -> Result<Self, LcuWebSocketError> {
        let auth = BASE64_STANDARD.encode(format!("riot:{}", credentials.password));
        let mut request = format!("wss://{LOCAL_LCU_HOST}:{}", credentials.port)
            .into_client_request()
            .map_err(|_| LcuWebSocketError::Authentication)?;
        request.headers_mut().insert(
            "Authorization",
            HeaderValue::from_str(format!("Basic {auth}").as_str())
                .map_err(|_| LcuWebSocketError::Authentication)?,
        );

        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .map_err(|_| LcuWebSocketError::Unavailable)?;
        let connector = Connector::NativeTls(tls);
        let (stream, _) =
            tokio_tungstenite::connect_async_tls_with_config(request, None, false, Some(connector))
                .await
                .map_err(|error| match error {
                    tungstenite::Error::Http(response)
                        if response.status() == tungstenite::http::StatusCode::UNAUTHORIZED
                            || response.status() == tungstenite::http::StatusCode::FORBIDDEN =>
                    {
                        LcuWebSocketError::Authentication
                    }
                    _ => LcuWebSocketError::Disconnected,
                })?;

        Ok(Self { stream })
    }

    pub async fn subscribe(
        &mut self,
        subscription: LcuSubscription,
    ) -> Result<(), LcuWebSocketError> {
        self.stream
            .send(Message::Text(format!("[5,\"{subscription}\"]").into()))
            .await
            .map_err(|error| match error {
                tungstenite::Error::ConnectionClosed | tungstenite::Error::AlreadyClosed => {
                    LcuWebSocketError::Disconnected
                }
                _ => LcuWebSocketError::Send,
            })
    }

    pub async fn next_event(&mut self) -> Result<Option<LcuWebSocketEvent>, LcuWebSocketError> {
        while let Some(message) = self.stream.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Some(event) = parse_lcu_websocket_event_text(text.as_str()) {
                        return Ok(Some(event));
                    }
                }
                Ok(Message::Close(_)) => {
                    log_lcu_adapter_event("websocket closed by client");
                    return Ok(None);
                }
                Ok(_) => {}
                Err(error) => {
                    return Err(match error {
                        tungstenite::Error::ConnectionClosed
                        | tungstenite::Error::AlreadyClosed => {
                            log_lcu_adapter_event("websocket disconnected");
                            LcuWebSocketError::Disconnected
                        }
                        _ => {
                            log_lcu_adapter_event("websocket unexpected error");
                            LcuWebSocketError::Unexpected
                        }
                    });
                }
            }
        }

        log_lcu_adapter_event("websocket stream ended");
        Ok(None)
    }
}

impl Stream for LcuWebSocketClient {
    type Item = LcuWebSocketEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.stream.poll_next_unpin(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(Ok(Message::Text(text)))) => {
                    if let Some(event) = parse_lcu_websocket_event_text(text.as_str()) {
                        return Poll::Ready(Some(event));
                    }
                }
                Poll::Ready(Some(Ok(Message::Close(_))) | Some(Err(_)) | None) => {
                    return Poll::Ready(None);
                }
                Poll::Ready(Some(Ok(_))) => {}
            }
        }
    }
}

pub fn parse_lcu_websocket_event_text(text: &str) -> Option<LcuWebSocketEvent> {
    let value: Value = serde_json::from_str(text).ok()?;
    let items = value.as_array()?;
    if items.first()?.as_i64()? != 8 {
        return None;
    }

    let subscription = items.get(1).and_then(Value::as_str);
    let payload = items.get(2)?.as_object()?;
    let uri = payload
        .get("uri")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| subscription.and_then(uri_from_lcu_subscription))?;
    let event_type = payload
        .get("eventType")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let data = payload.get("data").cloned().unwrap_or(Value::Null);

    Some(LcuWebSocketEvent {
        uri,
        event_type,
        data,
    })
}

fn uri_from_lcu_subscription(subscription: &str) -> Option<String> {
    subscription
        .strip_prefix("OnJsonApiEvent_")
        .map(|path| format!("/{}", path.replace('_', "/")))
}
