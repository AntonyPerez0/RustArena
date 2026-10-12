//! The HTTP API, tested in-process: each test builds the router on a fresh in-memory
//! database and sends it requests with `tower::ServiceExt::oneshot`, without opening a
//! port. Provided: don't change this file (the grader checks that). Make it pass.

use std::time::Duration;

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::{HeaderMap, Method, Request, StatusCode, header};
use serde_json::{Value, json};
use shortener::{AppState, app, db};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tower::ServiceExt;

const PUBLIC_URL: &str = "https://example.com";
const RUST: &str = "https://www.rust-lang.org/learn";

async fn test_app() -> Router {
    let pool = db::connect("sqlite::memory:")
        .await
        .expect("connect and migrate");
    app(AppState::new(pool, PUBLIC_URL))
}

struct Reply {
    status: StatusCode,
    headers: HeaderMap,
    body: Bytes,
}

impl Reply {
    fn json(&self) -> Value {
        serde_json::from_slice(&self.body).unwrap_or_else(|_| {
            panic!(
                "expected a JSON body, got {:?}",
                String::from_utf8_lossy(&self.body)
            )
        })
    }
    fn header(&self, name: header::HeaderName) -> Option<&str> {
        self.headers.get(name).and_then(|v| v.to_str().ok())
    }
    /// The `"error"` message of a JSON error body.
    fn error(&self) -> String {
        let body = self.json();
        let message = body["error"].as_str().unwrap_or_default().to_string();
        assert!(
            !message.is_empty(),
            "expected {{\"error\": \"...\"}}, got {body}"
        );
        message
    }
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    Reply {
        status,
        headers,
        body,
    }
}

async fn get(app: &Router, uri: &str) -> Reply {
    send(app, Request::get(uri).body(Body::empty()).unwrap()).await
}

async fn delete(app: &Router, uri: &str) -> Reply {
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    send(app, request).await
}

async fn post_json(app: &Router, uri: &str, body: Value) -> Reply {
    let request = Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap();
    send(app, request).await
}

/// Creates a link and returns its code.
async fn shorten(app: &Router, body: Value) -> String {
    let reply = post_json(app, "/api/links", body).await;
    assert_eq!(reply.status, StatusCode::CREATED, "body: {:?}", reply.body);
    reply.json()["code"]
        .as_str()
        .expect("a \"code\" in the response")
        .to_string()
}

#[tokio::test]
async fn health_reports_ok() {
    let app = test_app().await;
    let reply = get(&app, "/health").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.json(), json!({ "status": "ok" }));
}

#[tokio::test]
async fn creating_a_link_returns_201_with_a_generated_code() {
    let app = test_app().await;
    let reply = post_json(&app, "/api/links", json!({ "url": RUST })).await;
    assert_eq!(reply.status, StatusCode::CREATED);
    let body = reply.json();
    let code = body["code"].as_str().expect("a \"code\" string");
    assert_eq!(code.len(), 7, "generated codes have 7 characters: {code:?}");
    assert!(
        code.chars().all(|c| c.is_ascii_alphanumeric()),
        "letters and digits only: {code:?}"
    );
    assert_eq!(body["url"], RUST);
    assert_eq!(body["short_url"], format!("{PUBLIC_URL}/{code}"));
    assert_eq!(
        reply.header(header::LOCATION),
        Some(format!("/api/links/{code}").as_str())
    );
}

#[tokio::test]
async fn generated_codes_are_unique() {
    let app = test_app().await;
    let mut codes = Vec::new();
    for _ in 0..20 {
        codes.push(shorten(&app, json!({ "url": RUST })).await);
    }
    codes.sort();
    codes.dedup();
    assert_eq!(
        codes.len(),
        20,
        "the same URL shortened twice gets two codes"
    );
}

#[tokio::test]
async fn a_custom_code_can_be_chosen() {
    let app = test_app().await;
    let code = shorten(&app, json!({ "url": RUST, "code": "rust-learn" })).await;
    assert_eq!(code, "rust-learn");
}

#[tokio::test]
async fn a_taken_code_is_a_409_conflict() {
    let app = test_app().await;
    shorten(&app, json!({ "url": RUST, "code": "rust_2024" })).await;
    let reply = post_json(
        &app,
        "/api/links",
        json!({ "url": "https://example.com/", "code": "rust_2024" }),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert!(
        reply.error().contains("rust_2024"),
        "say which code is taken: {}",
        reply.error()
    );
}

#[tokio::test]
async fn invalid_urls_are_rejected_with_422() {
    let app = test_app().await;
    let too_long = format!("https://example.com/{}", "a".repeat(2100));
    for url in [
        "",
        "not a url",
        "example.com/no-scheme",
        "ftp://example.com/file",
        "javascript:alert(1)",
        "https://",
        too_long.as_str(),
    ] {
        let reply = post_json(&app, "/api/links", json!({ "url": url })).await;
        assert_eq!(
            reply.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "url {url:?} should be rejected"
        );
        reply.error();
    }
}

#[tokio::test]
async fn invalid_custom_codes_are_rejected_with_422() {
    let app = test_app().await;
    let too_long = "x".repeat(33);
    for code in [
        "abc",
        "has space",
        "slash/es",
        "dot.dot",
        "émoji",
        too_long.as_str(),
        "api",
        "health",
    ] {
        let reply = post_json(&app, "/api/links", json!({ "url": RUST, "code": code })).await;
        assert_eq!(
            reply.status,
            StatusCode::UNPROCESSABLE_ENTITY,
            "code {code:?} should be rejected"
        );
        reply.error();
    }
    // The limits are inclusive.
    shorten(&app, json!({ "url": RUST, "code": "abcd" })).await;
    shorten(&app, json!({ "url": RUST, "code": "y".repeat(32) })).await;
}

#[tokio::test]
async fn malformed_requests_are_client_errors() {
    let app = test_app().await;
    // Missing field: valid JSON with the wrong shape.
    let reply = post_json(&app, "/api/links", json!({ "link": RUST })).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    // Not JSON at all.
    let request = Request::post("/api/links")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    assert_eq!(send(&app, request).await.status, StatusCode::BAD_REQUEST);
    // No JSON content type.
    let request = Request::post("/api/links")
        .body(Body::from(json!({ "url": RUST }).to_string()))
        .unwrap();
    assert_eq!(
        send(&app, request).await.status,
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
}

#[tokio::test]
async fn a_short_link_redirects_to_its_url() {
    let app = test_app().await;
    let code = shorten(&app, json!({ "url": RUST })).await;
    let reply = get(&app, &format!("/{code}")).await;
    assert_eq!(reply.status, StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(reply.header(header::LOCATION), Some(RUST));
}

#[tokio::test]
async fn unknown_codes_are_404() {
    let app = test_app().await;
    let reply = get(&app, "/nope123").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    reply.error();
    assert_eq!(
        get(&app, "/api/links/nope123").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        delete(&app, "/api/links/nope123").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn details_count_the_visits() {
    let app = test_app().await;
    let code = shorten(&app, json!({ "url": RUST, "code": "counted" })).await;

    let reply = get(&app, "/api/links/counted").await;
    assert_eq!(reply.status, StatusCode::OK);
    let body = reply.json();
    assert_eq!(body["code"], code);
    assert_eq!(body["url"], RUST);
    assert_eq!(body["short_url"], format!("{PUBLIC_URL}/counted"));
    assert_eq!(body["hits"], 0);
    assert!(
        body["created_at"].as_str().is_some_and(|t| !t.is_empty()),
        "a created_at time: {body}"
    );

    get(&app, "/counted").await;
    get(&app, "/counted").await;
    assert_eq!(get(&app, "/api/links/counted").await.json()["hits"], 2);
}

#[tokio::test]
async fn a_deleted_link_stops_working() {
    let app = test_app().await;
    let code = shorten(&app, json!({ "url": RUST })).await;
    let reply = delete(&app, &format!("/api/links/{code}")).await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert_eq!(
        get(&app, &format!("/{code}")).await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        delete(&app, &format!("/api/links/{code}")).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn the_server_stops_gracefully_when_asked() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stop_requested) = tokio::sync::oneshot::channel::<()>();
    let shutdown = async {
        let _ = stop_requested.await;
    };
    let server = tokio::spawn(shortener::serve(listener, test_app().await, shutdown));

    // A real HTTP request over a real TCP connection.
    let mut stream = TcpStream::connect(addr).await.unwrap();
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    assert!(response.starts_with("HTTP/1.1 200"), "got: {response}");

    stop.send(()).unwrap();
    let finished = tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("serve() should return within 5 seconds of the shutdown signal");
    finished.unwrap().expect("serve() returned an error");
}
