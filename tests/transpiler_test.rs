use serde_json::json;
use tokenectomy::transpiler::{
    anthropic_to_openai_request, anthropic_to_openai_response,
    openai_chunk_to_anthropic_sse, openai_to_anthropic_request,
    openai_to_anthropic_response,
};

#[test]
fn test_anthropic_to_openai_simple_conversion() {
    let ant_req = json!({
        "model": "claude-3-5-sonnet-20241022",
        "max_tokens": 1024,
        "system": "You are a helpful coding assistant.",
        "messages": [
            {"role": "user", "content": "How do I reverse a string in Rust?"}
        ]
    });

    let openai_req = anthropic_to_openai_request(&ant_req).expect("transpile failed");
    assert_eq!(openai_req["model"], "claude-3-5-sonnet-20241022");
    assert_eq!(openai_req["max_tokens"], 1024);

    let msgs = openai_req["messages"].as_array().expect("messages array");
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0]["role"], "system");
    assert_eq!(msgs[0]["content"], "You are a helpful coding assistant.");
    assert_eq!(msgs[1]["role"], "user");
    assert_eq!(msgs[1]["content"], "How do I reverse a string in Rust?");
}

#[test]
fn test_anthropic_to_openai_content_blocks_and_tools() {
    let ant_req = json!({
        "model": "claude-3-7-sonnet",
        "messages": [
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "Execute the tool please"}
                ]
            },
            {
                "role": "assistant",
                "content": [
                    {"type": "text", "text": "Running tool..."},
                    {
                        "type": "tool_use",
                        "id": "tool_123",
                        "name": "get_weather",
                        "input": {"location": "Jakarta"}
                    }
                ]
            },
            {
                "role": "user",
                "content": [
                    {
                        "type": "tool_result",
                        "tool_use_id": "tool_123",
                        "content": "Sunny 30C"
                    }
                ]
            }
        ],
        "tools": [
            {
                "name": "get_weather",
                "description": "Get current weather",
                "input_schema": {
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                }
            }
        ]
    });

    let openai_req = anthropic_to_openai_request(&ant_req).expect("transpile failed");
    let msgs = openai_req["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 3);

    // Check tool_calls
    let asst = &msgs[1];
    assert_eq!(asst["role"], "assistant");
    assert_eq!(asst["content"], "Running tool...");
    let tool_calls = asst["tool_calls"].as_array().unwrap();
    assert_eq!(tool_calls[0]["id"], "tool_123");
    assert_eq!(tool_calls[0]["function"]["name"], "get_weather");

    // Check tool response
    let tool_res = &msgs[2];
    assert_eq!(tool_res["role"], "tool");
    assert_eq!(tool_res["tool_call_id"], "tool_123");
    assert_eq!(tool_res["content"], "Sunny 30C");

    // Check tool schema mapping
    let tools = openai_req["tools"].as_array().unwrap();
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["function"]["name"], "get_weather");
}

#[test]
fn test_openai_to_anthropic_response_mapping() {
    let openai_resp = json!({
        "id": "chatcmpl-999",
        "model": "gpt-4o",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Here is the solution to your bug."
                },
                "finish_reason": "stop"
            }
        ],
        "usage": {
            "prompt_tokens": 120,
            "completion_tokens": 45,
            "total_tokens": 165
        }
    });

    let ant_resp = openai_to_anthropic_response(&openai_resp, None).expect("transpile failed");
    assert_eq!(ant_resp["id"], "msg_chatcmpl-999");
    assert_eq!(ant_resp["type"], "message");
    assert_eq!(ant_resp["role"], "assistant");
    assert_eq!(ant_resp["stop_reason"], "end_turn");
    assert_eq!(ant_resp["usage"]["input_tokens"], 120);
    assert_eq!(ant_resp["usage"]["output_tokens"], 45);

    let blocks = ant_resp["content"].as_array().unwrap();
    assert_eq!(blocks[0]["type"], "text");
    assert_eq!(blocks[0]["text"], "Here is the solution to your bug.");
}

#[test]
fn test_openai_to_anthropic_request_conversion() {
    let openai_req = json!({
        "model": "gpt-4o",
        "messages": [
            {"role": "system", "content": "Act as a senior security researcher."},
            {"role": "user", "content": "Analyze this code"}
        ],
        "max_tokens": 2048,
        "temperature": 0.2
    });

    let ant_req = openai_to_anthropic_request(&openai_req).expect("transpile failed");
    assert_eq!(ant_req["system"], "Act as a senior security researcher.");
    assert_eq!(ant_req["max_tokens"], 2048);
    assert_eq!(ant_req["temperature"], 0.2);

    let msgs = ant_req["messages"].as_array().unwrap();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0]["role"], "user");
    assert_eq!(msgs[0]["content"], "Analyze this code");
}

#[test]
fn test_anthropic_to_openai_response_mapping() {
    let ant_resp = json!({
        "id": "msg_01A",
        "model": "claude-3-7-sonnet",
        "content": [
            {"type": "text", "text": "Function generated."}
        ],
        "stop_reason": "end_turn",
        "usage": {
            "input_tokens": 50,
            "output_tokens": 25
        }
    });

    let openai_resp = anthropic_to_openai_response(&ant_resp).expect("transpile failed");
    assert_eq!(openai_resp["object"], "chat.completion");
    assert_eq!(openai_resp["choices"][0]["message"]["content"], "Function generated.");
    assert_eq!(openai_resp["choices"][0]["finish_reason"], "stop");
    assert_eq!(openai_resp["usage"]["prompt_tokens"], 50);
    assert_eq!(openai_resp["usage"]["completion_tokens"], 25);
}

#[test]
fn test_openai_sse_chunk_to_anthropic_events() {
    let first_chunk = json!({
        "id": "chatcmpl-stream-1",
        "model": "gpt-4o",
        "choices": [
            {
                "index": 0,
                "delta": {
                    "content": "Hello"
                },
                "finish_reason": null
            }
        ]
    });

    let events = openai_chunk_to_anthropic_sse(&first_chunk, true);
    assert!(events.iter().any(|e| e.contains("event: message_start")));
    assert!(events.iter().any(|e| e.contains("event: content_block_start")));
    assert!(events.iter().any(|e| e.contains("event: content_block_delta") && e.contains("Hello")));

    let final_chunk = json!({
        "id": "chatcmpl-stream-1",
        "choices": [
            {
                "index": 0,
                "delta": {},
                "finish_reason": "stop"
            }
        ]
    });

    let stop_events = openai_chunk_to_anthropic_sse(&final_chunk, false);
    assert!(stop_events.iter().any(|e| e.contains("event: content_block_stop")));
    assert!(stop_events.iter().any(|e| e.contains("event: message_delta") && e.contains("end_turn")));
    assert!(stop_events.iter().any(|e| e.contains("event: message_stop")));
}
