//! Tests every endpoint by calling the router directly with tower's `oneshot`:
//! no port, no network, so they are fast and run in parallel.
//! Don't edit this file: the grader checks that it is unchanged.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

struct Reply {
    status: StatusCode,
    location: Option<String>,
    body: Value,
}

impl Reply {
    /// The message of a JSON error body: {"error": "..."}.
    fn error(&self) -> &str {
        self.body["error"].as_str().unwrap_or_else(|| {
            panic!(
                "expected a JSON body like {{\"error\": \"...\"}}, got {}",
                self.body
            )
        })
    }
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    // A Router is a tower Service: oneshot sends it one request.
    // Cloning a Router is cheap and shares its state.
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let location = response
        .headers()
        .get(header::LOCATION)
        .map(|value| value.to_str().unwrap().to_string());
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            panic!(
                "{status}: the body is not JSON: {:?}",
                String::from_utf8_lossy(&bytes)
            )
        })
    };
    Reply {
        status,
        location,
        body,
    }
}

async fn call(app: &Router, method: &str, uri: &str, json: Option<Value>) -> Reply {
    let request = Request::builder().method(method).uri(uri);
    let request = match json {
        Some(json) => request
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json.to_string())),
        None => request.body(Body::empty()),
    };
    send(app, request.unwrap()).await
}

async fn get(app: &Router, uri: &str) -> Reply {
    call(app, "GET", uri, None).await
}

async fn post(app: &Router, uri: &str, json: Value) -> Reply {
    call(app, "POST", uri, Some(json)).await
}

async fn patch(app: &Router, uri: &str, json: Value) -> Reply {
    call(app, "PATCH", uri, Some(json)).await
}

async fn delete(app: &Router, uri: &str) -> Reply {
    call(app, "DELETE", uri, None).await
}

/// Creates a bookmark and returns its JSON.
async fn create(app: &Router, url: &str, title: &str, tags: &[&str]) -> Value {
    let reply = post(
        app,
        "/bookmarks",
        json!({ "url": url, "title": title, "tags": tags }),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.body);
    reply.body
}

#[tokio::test]
async fn health_check() {
    let app = bookmarks::app();
    let reply = get(&app, "/health").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, json!({ "status": "ok" }));
}

#[tokio::test]
async fn create_returns_201_with_a_location() {
    let app = bookmarks::app();
    let reply = post(
        &app,
        "/bookmarks",
        json!({ "url": " https://www.rust-lang.org ", "title": "  Rust  ", "tags": ["rust", "lang", "rust"] }),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.body);
    assert_eq!(reply.location.as_deref(), Some("/bookmarks/1"));
    assert_eq!(
        reply.body,
        json!({ "id": 1, "url": "https://www.rust-lang.org", "title": "Rust", "tags": ["lang", "rust"] })
    );
}

#[tokio::test]
async fn tags_are_optional() {
    let app = bookmarks::app();
    let reply = post(
        &app,
        "/bookmarks",
        json!({ "url": "http://example.com", "title": "Example" }),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.body);
    assert_eq!(reply.body["tags"], json!([]));
}

#[tokio::test]
async fn read_a_bookmark() {
    let app = bookmarks::app();
    let created = create(&app, "https://docs.rs", "Docs.rs", &["docs"]).await;
    let reply = get(&app, "/bookmarks/1").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, created);
}

#[tokio::test]
async fn ids_increase_and_are_never_reused() {
    let app = bookmarks::app();
    create(&app, "https://a.example", "A", &[]).await;
    create(&app, "https://b.example", "B", &[]).await;
    assert_eq!(
        delete(&app, "/bookmarks/2").await.status,
        StatusCode::NO_CONTENT
    );
    let c = create(&app, "https://c.example", "C", &[]).await;
    assert_eq!(c["id"], 3);
}

#[tokio::test]
async fn invalid_bookmarks_get_422_and_a_message() {
    let app = bookmarks::app();
    let long_url = format!("https://example.com/{}", "a".repeat(2000));
    let long_title = "é".repeat(201);
    let eleven_tags: Vec<String> = (0..11).map(|i| format!("tag{i}")).collect();
    let cases = [
        (
            json!({ "url": "ftp://example.com", "title": "x" }),
            "url must be an http:// or https:// URL",
        ),
        (
            json!({ "url": "https://", "title": "x" }),
            "url must be an http:// or https:// URL",
        ),
        (
            json!({ "url": "https://exa mple.com", "title": "x" }),
            "url must be an http:// or https:// URL",
        ),
        (
            json!({ "url": long_url, "title": "x" }),
            "url must be at most 2000 characters",
        ),
        (
            json!({ "url": "https://example.com", "title": "   " }),
            "title must not be empty",
        ),
        (
            json!({ "url": "https://example.com", "title": long_title }),
            "title must be at most 200 characters",
        ),
        (
            json!({ "url": "https://example.com", "title": "x", "tags": eleven_tags }),
            "at most 10 tags",
        ),
        (
            json!({ "url": "https://example.com", "title": "x", "tags": ["Rust"] }),
            "invalid tag \"Rust\"",
        ),
        (
            json!({ "url": "https://example.com", "title": "x", "tags": ["ok", ""] }),
            "invalid tag \"\"",
        ),
        (
            json!({ "url": "https://example.com", "title": "x", "tags": ["has space"] }),
            "invalid tag \"has space\"",
        ),
    ];
    for (body, message) in cases {
        let reply = post(&app, "/bookmarks", body.clone()).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "for {body}");
        assert_eq!(reply.error(), message, "for {body}");
    }
    // Exactly at the limits is fine.
    let title = "é".repeat(200);
    let ten_tags: Vec<String> = (0..10).map(|i| format!("tag-{i}")).collect();
    let reply = post(
        &app,
        "/bookmarks",
        json!({ "url": "https://example.com", "title": title, "tags": ten_tags }),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.body);
    assert_eq!(get(&app, "/bookmarks").await.body["total"], 1);
}

#[tokio::test]
async fn duplicate_urls_get_409() {
    let app = bookmarks::app();
    create(&app, "https://tokio.rs", "Tokio", &[]).await;
    let reply = post(
        &app,
        "/bookmarks",
        json!({ "url": "https://tokio.rs", "title": "Tokio again" }),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert_eq!(reply.error(), "a bookmark with this url already exists");
}

#[tokio::test]
async fn bad_requests_get_json_errors_with_axums_status() {
    let app = bookmarks::app();
    // Not JSON at all.
    let request = Request::post("/bookmarks")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{not json"))
        .unwrap();
    let reply = send(&app, request).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(!reply.error().is_empty());
    // JSON, but a required field is missing.
    let reply = post(&app, "/bookmarks", json!({ "url": "https://example.com" })).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert!(reply.error().contains("title"), "{}", reply.error());
    // No content-type header.
    let request = Request::post("/bookmarks")
        .body(Body::from(r#"{"url":"https://example.com","title":"x"}"#))
        .unwrap();
    let reply = send(&app, request).await;
    assert_eq!(reply.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert!(!reply.error().is_empty());
    // An id that is not a number.
    let reply = get(&app, "/bookmarks/abc").await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert!(!reply.error().is_empty());
}

#[tokio::test]
async fn missing_bookmarks_get_404() {
    let app = bookmarks::app();
    let reply = get(&app, "/bookmarks/99").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.error(), "bookmark 99 not found");
    let reply = patch(&app, "/bookmarks/99", json!({ "title": "x" })).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.error(), "bookmark 99 not found");
    let reply = delete(&app, "/bookmarks/99").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.error(), "bookmark 99 not found");
}

#[tokio::test]
async fn unknown_routes_get_a_json_404_and_wrong_methods_405() {
    let app = bookmarks::app();
    let reply = get(&app, "/nope").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert_eq!(reply.error(), "no route for GET /nope");
    let reply = call(&app, "PUT", "/bookmarks/1", Some(json!({}))).await;
    assert_eq!(reply.status, StatusCode::METHOD_NOT_ALLOWED);
}

#[tokio::test]
async fn patch_changes_only_the_fields_given() {
    let app = bookmarks::app();
    create(&app, "https://serde.rs", "Serde", &["serde"]).await;
    create(&app, "https://tokio.rs", "Tokio", &[]).await;

    let reply = patch(&app, "/bookmarks/1", json!({ "title": " Serde docs " })).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(
        reply.body,
        json!({ "id": 1, "url": "https://serde.rs", "title": "Serde docs", "tags": ["serde"] })
    );

    let reply = patch(&app, "/bookmarks/1", json!({ "tags": ["b", "a", "a"] })).await;
    assert_eq!(reply.body["tags"], json!(["a", "b"]));

    // Its own url is not a conflict; another bookmark's is.
    let reply = patch(&app, "/bookmarks/1", json!({ "url": "https://serde.rs" })).await;
    assert_eq!(reply.status, StatusCode::OK);
    let reply = patch(&app, "/bookmarks/1", json!({ "url": "https://tokio.rs" })).await;
    assert_eq!(reply.status, StatusCode::CONFLICT);

    // A failed update changes nothing.
    let reply = patch(&app, "/bookmarks/1", json!({ "title": "" })).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reply.error(), "title must not be empty");
    assert_eq!(
        get(&app, "/bookmarks/1").await.body,
        json!({ "id": 1, "url": "https://serde.rs", "title": "Serde docs", "tags": ["a", "b"] })
    );
}

#[tokio::test]
async fn delete_returns_204_with_no_body() {
    let app = bookmarks::app();
    create(&app, "https://crates.io", "crates.io", &[]).await;
    let reply = delete(&app, "/bookmarks/1").await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT);
    assert_eq!(reply.body, Value::Null);
    assert_eq!(
        get(&app, "/bookmarks/1").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        delete(&app, "/bookmarks/1").await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn list_returns_items_in_id_order_with_a_total() {
    let app = bookmarks::app();
    let empty = get(&app, "/bookmarks").await;
    assert_eq!(empty.status, StatusCode::OK);
    assert_eq!(empty.body, json!({ "items": [], "total": 0 }));
    let a = create(&app, "https://a.example", "A", &[]).await;
    let b = create(&app, "https://b.example", "B", &[]).await;
    let reply = get(&app, "/bookmarks").await;
    assert_eq!(reply.body, json!({ "items": [a, b], "total": 2 }));
}

#[tokio::test]
async fn list_filters_by_tag_and_title() {
    let app = bookmarks::app();
    create(
        &app,
        "https://tokio.rs",
        "Tokio: async Rust",
        &["rust", "async"],
    )
    .await;
    create(&app, "https://serde.rs", "Serde", &["rust"]).await;
    create(
        &app,
        "https://nodejs.org",
        "Node.js ASYNC I/O",
        &["js", "async"],
    )
    .await;
    let titles = |reply: &Reply| -> Vec<String> {
        reply.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["title"].as_str().unwrap().to_string())
            .collect()
    };
    let reply = get(&app, "/bookmarks?tag=rust").await;
    assert_eq!(titles(&reply), ["Tokio: async Rust", "Serde"]);
    assert_eq!(reply.body["total"], 2);
    let reply = get(&app, "/bookmarks?q=Async").await;
    assert_eq!(titles(&reply), ["Tokio: async Rust", "Node.js ASYNC I/O"]);
    let reply = get(&app, "/bookmarks?q=async&tag=js").await;
    assert_eq!(titles(&reply), ["Node.js ASYNC I/O"]);
    let reply = get(&app, "/bookmarks?tag=go").await;
    assert_eq!(reply.body, json!({ "items": [], "total": 0 }));
}

#[tokio::test]
async fn list_paginates() {
    let app = bookmarks::app();
    for i in 1..=25 {
        create(
            &app,
            &format!("https://example.com/{i}"),
            &format!("Page {i}"),
            &[],
        )
        .await;
    }
    let ids = |reply: &Reply| -> Vec<u64> {
        reply.body["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|item| item["id"].as_u64().unwrap())
            .collect()
    };
    let reply = get(&app, "/bookmarks").await;
    assert_eq!(
        ids(&reply),
        (1..=20).collect::<Vec<_>>(),
        "the default limit is 20"
    );
    assert_eq!(reply.body["total"], 25);
    let reply = get(&app, "/bookmarks?limit=10&offset=20").await;
    assert_eq!(ids(&reply), [21, 22, 23, 24, 25]);
    assert_eq!(reply.body["total"], 25);
    let reply = get(&app, "/bookmarks?offset=100").await;
    assert_eq!(reply.body, json!({ "items": [], "total": 25 }));
    let reply = get(&app, "/bookmarks?limit=100").await;
    assert_eq!(ids(&reply).len(), 25);
}

#[tokio::test]
async fn bad_query_parameters_get_400() {
    let app = bookmarks::app();
    for uri in ["/bookmarks?limit=0", "/bookmarks?limit=101"] {
        let reply = get(&app, uri).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(reply.error(), "limit must be between 1 and 100", "{uri}");
    }
    for uri in ["/bookmarks?limit=ten", "/bookmarks?offset=-1"] {
        let reply = get(&app, uri).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{uri}");
        assert!(!reply.error().is_empty(), "{uri}");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_requests_share_one_store() {
    let app = bookmarks::app();
    let mut tasks = tokio::task::JoinSet::new();
    for i in 0..50 {
        let app = app.clone();
        tasks.spawn(async move {
            let url = format!("https://example.com/{i}");
            post(&app, "/bookmarks", json!({ "url": url, "title": "x" })).await
        });
    }
    let mut ids = Vec::new();
    while let Some(reply) = tasks.join_next().await {
        let reply = reply.unwrap();
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.body);
        ids.push(reply.body["id"].as_u64().unwrap());
    }
    ids.sort();
    assert_eq!(ids, (1..=50).collect::<Vec<_>>());
    assert_eq!(get(&app, "/bookmarks").await.body["total"], 50);
}
