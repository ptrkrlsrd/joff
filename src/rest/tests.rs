use super::StorableResponse;
use serde_json::json;
use std::collections::HashMap;
use super::RestClient;
use httpmock::MockServer;
use url::Url;


#[test]
fn storable_response_from() {
    let body = "This is a test response".to_string();
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let response = StorableResponse::from(body.clone(), headers.clone());

    assert_eq!(response.body, body);
    assert_eq!(response.headers, headers);
}

#[test]
fn storable_response_from_json() {
    let body = "This is a test response".to_string();
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let json_data = json!({
        "body": body,
        "headers": headers
    });

    let response_result = StorableResponse::from_json(json_data.to_string());

    assert!(response_result.is_ok());
    let response = response_result.unwrap();

    assert_eq!(response.body, body);
    assert_eq!(response.headers, headers);
}

#[test]
fn storable_response_from_json_invalid() {
    let json_data = json!({
        "body": 123,
        "headers": "Invalid headers"
    });

    let response_result = StorableResponse::from_json(json_data.to_string());

    assert!(response_result.is_err());
}

#[tokio::test]
async fn rest_client_get_json() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method("GET").path("/test");
        then.status(200).json_body(json!(
        {
            "message": "success"
        }
        ));
    });

    // Call the RestClient's get_json function
    let url = Url::parse(&format!("{}{}", server.base_url(), "/test")).unwrap();
    let result = RestClient::get_json(url).await;

    // Assert the result
    assert!(result.is_ok());
    let json_result = result.unwrap();
    let expected_body = r#"{"message":"success"}"#;
    assert_eq!(json_result["body"], expected_body);

    mock.assert_hits(1);
}

#[tokio::test]
async fn rest_client_get_json_error_response() {
    let server = MockServer::start();

    let mock = server.mock(|when, then| {
        when.method("GET").path("/test");
        then.status(404);
    });

    let url = Url::parse(&format!("{}{}", server.base_url(), "/test")).unwrap();
    let result = RestClient::get_json(url).await;

    assert!(result.is_err());

    mock.assert_hits(1);
}

