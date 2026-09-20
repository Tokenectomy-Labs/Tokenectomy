use serde_json::json;
use tokenectomy::transpiler::{
    anthropic_to_openai_request, anthropic_to_openai_response,
    detect_transpile_direction, openai_to_anthropic_response,
    TranspileDirection,
};

#[test]
fn test_adversary_openai_error_response_transpilation() {
    // Attack vector: Upstream OpenAI returns 401/400 error payload
    let openai_err = json!({
        "error": {
            "message": "Invalid API key provided",
            "type": "invalid_request_error",
            "param": null,
            "code": "invalid_api_key"
        }
    });

    let ant_resp = openai_to_anthropic_response(&openai_err, None)
        .expect("Must handle OpenAI error payload gracefully");

    // Must be mapped to Anthropic error format!
    assert_eq!(ant_resp["type"], "error");
    assert_eq!(ant_resp["error"]["message"], "Invalid API key provided");
    assert_eq!(ant_resp["error"]["type"], "authentication_error");
}

#[test]
fn test_adversary_anthropic_error_response_transpilation() {
    // Attack vector: Upstream Anthropic returns error payload
    let ant_err = json!({
        "type": "error",
        "error": {
            "type": "rate_limit_error",
            "message": "Number of request tokens has exceeded your daily limit"
        }
    });

    let openai_resp = anthropic_to_openai_response(&ant_err)
        .expect("Must handle Anthropic error payload gracefully");

    // Must be mapped to OpenAI error format!
    assert!(openai_resp.get("error").is_some());
    assert_eq!(
        openai_resp["error"]["message"],
        "Number of request tokens has exceeded your daily limit"
    );
    assert_eq!(openai_resp["error"]["type"], "rate_limit_error");
}

#[test]
fn test_adversary_empty_and_pathological_messages() {
    // Attack vector: empty messages array, null content, missing role
    let pathological_ant = json!({
        "messages": [
            {"content": null},
            {"role": "user", "content": []},
            {"role": "assistant", "content": ""},
            {"content": "plain string"}
        ]
    });

    let res = anthropic_to_openai_request(&pathological_ant, None);
    assert!(res.is_ok(), "Must survive pathological message contents without panic");
}

#[test]
fn test_adversary_target_model_override_header() {
    // Attack vector: Claude Code requests claude-3-5-sonnet, but upstream is DeepSeek
    let ant_req = json!({
        "model": "claude-3-5-sonnet-20241022",
        "messages": [{"role": "user", "content": "hi"}]
    });

    let openai_req = anthropic_to_openai_request(&ant_req, Some("deepseek-chat")).unwrap();
    assert_eq!(openai_req["model"], "deepseek-chat");

    // When model override is provided, OpenAI request must use the override!
    let headers = vec![("X-Tokenectomy-Target-Model".to_string(), "deepseek-chat".to_string())];
    let transpile_dir = detect_transpile_direction("/v1/messages", "https://api.deepseek.com/v1", &headers);
    assert_eq!(transpile_dir, TranspileDirection::AnthropicToOpenAi);
}
