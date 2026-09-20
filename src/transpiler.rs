use serde_json::{json, Value};

/// Rewrites a target URL's endpoint path to a new endpoint while preserving scheme, host, port, and query string.
pub fn rewrite_transpiled_url(original_url: &str, new_endpoint: &str) -> String {
    let (url_without_query, query_part) = original_url.split_once('?').unwrap_or((original_url, ""));

    let base = if let Some(b) = url_without_query.strip_suffix("/v1/messages") {
        b
    } else if let Some(b) = url_without_query.strip_suffix("/messages") {
        b
    } else if let Some(b) = url_without_query.strip_suffix("/v1/chat/completions") {
        b
    } else if let Some(b) = url_without_query.strip_suffix("/chat/completions") {
        b
    } else if let Some(idx) = url_without_query.find("/v1/messages") {
        &url_without_query[..idx]
    } else if let Some(idx) = url_without_query.find("/v1/chat/completions") {
        &url_without_query[..idx]
    } else if let Some(idx) = url_without_query.find("/messages") {
        &url_without_query[..idx]
    } else if let Some(idx) = url_without_query.find("/chat/completions") {
        &url_without_query[..idx]
    } else {
        url_without_query.trim_end_matches('/')
    };

    let clean_endpoint = if new_endpoint.starts_with('/') {
        new_endpoint
    } else {
        &format!("/{}", new_endpoint)
    };

    let combined = format!("{}{}", base, clean_endpoint);
    if query_part.is_empty() {
        combined
    } else {
        format!("{}?{}", combined, query_part)
    }
}

/// Converts an Anthropic `/v1/messages` request payload into an OpenAI `/v1/chat/completions` payload.
pub fn anthropic_to_openai_request(
    anthropic_body: &Value,
    model_override: Option<&str>,
) -> Result<Value, String> {
    let mut openai_messages = Vec::new();

    // 1. Extract system prompt if present
    if let Some(sys) = anthropic_body.get("system") {
        let system_text = match sys {
            Value::String(s) => s.clone(),
            Value::Array(blocks) => {
                let mut combined = String::new();
                for b in blocks {
                    if let Some(t) = b.get("text").and_then(|x| x.as_str()) {
                        if !combined.is_empty() {
                            combined.push('\n');
                        }
                        combined.push_str(t);
                    }
                }
                combined
            }
            _ => sys.to_string(),
        };

        if !system_text.trim().is_empty() {
            openai_messages.push(json!({
                "role": "system",
                "content": system_text
            }));
        }
    }

    // 2. Convert Anthropic messages array
    if let Some(messages) = anthropic_body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content_val = msg.get("content").unwrap_or(&Value::Null);

            match content_val {
                Value::String(s) => {
                    openai_messages.push(json!({
                        "role": role,
                        "content": s
                    }));
                }
                Value::Array(blocks) => {
                    let mut text_parts = Vec::new();
                    let mut tool_calls = Vec::new();

                    for block in blocks {
                        let b_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        match b_type {
                            "text" => {
                                if let Some(txt) = block.get("text").and_then(|t| t.as_str()) {
                                    text_parts.push(txt);
                                }
                            }
                            "tool_use" => {
                                let id = block.get("id").and_then(|i| i.as_str()).unwrap_or("call_default");
                                let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("unknown_tool");
                                let fallback_input = json!({});
                                let input = block.get("input").unwrap_or(&fallback_input);
                                let input_str = serde_json::to_string(input).unwrap_or_else(|_| "{}".to_string());

                                tool_calls.push(json!({
                                    "id": id,
                                    "type": "function",
                                    "function": {
                                        "name": name,
                                        "arguments": input_str
                                    }
                                }));
                            }
                            "tool_result" => {
                                let tool_use_id = block.get("tool_use_id").and_then(|i| i.as_str()).unwrap_or("call_default");
                                let res_content = match block.get("content") {
                                    Some(Value::String(s)) => s.clone(),
                                    Some(other) => serde_json::to_string(other).unwrap_or_default(),
                                    None => String::new(),
                                };
                                openai_messages.push(json!({
                                    "role": "tool",
                                    "tool_call_id": tool_use_id,
                                    "content": res_content
                                }));
                            }
                            _ => {}
                        }
                    }

                    if !text_parts.is_empty() || !tool_calls.is_empty() {
                        let combined_text = text_parts.join("\n");
                        let mut msg_obj = json!({
                            "role": role,
                            "content": if combined_text.is_empty() && !tool_calls.is_empty() { Value::Null } else { Value::String(combined_text) }
                        });
                        if !tool_calls.is_empty() {
                            msg_obj["tool_calls"] = Value::Array(tool_calls);
                        }
                        openai_messages.push(msg_obj);
                    }
                }
                _ => {
                    let content_str = if content_val.is_null() {
                        String::new()
                    } else {
                        content_val.to_string()
                    };
                    openai_messages.push(json!({
                        "role": role,
                        "content": content_str
                    }));
                }
            }
        }
    }

    let model = model_override
        .filter(|m| !m.trim().is_empty())
        .or_else(|| anthropic_body.get("model").and_then(|m| m.as_str()))
        .unwrap_or("gpt-4o");

    let mut out = json!({
        "model": model,
        "messages": openai_messages
    });

    if let Some(max_tokens) = anthropic_body.get("max_tokens").and_then(|t| t.as_u64()) {
        out["max_tokens"] = json!(max_tokens);
    }
    if let Some(temp) = anthropic_body.get("temperature") {
        out["temperature"] = temp.clone();
    }
    if let Some(top_p) = anthropic_body.get("top_p") {
        out["top_p"] = top_p.clone();
    }
    if let Some(stream) = anthropic_body.get("stream") {
        out["stream"] = stream.clone();
    }

    // Convert tools if present
    if let Some(tools) = anthropic_body.get("tools").and_then(|t| t.as_array()) {
        let mut openai_tools = Vec::new();
        let fallback_schema = json!({
            "type": "object",
            "properties": {}
        });
        for t in tools {
            let name = t.get("name").and_then(|n| n.as_str()).unwrap_or("tool");
            let desc = t.get("description").and_then(|d| d.as_str()).unwrap_or("");
            let params = t.get("input_schema").unwrap_or(&fallback_schema);
            openai_tools.push(json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": desc,
                    "parameters": params.clone()
                }
            }));
        }
        if !openai_tools.is_empty() {
            out["tools"] = Value::Array(openai_tools);
        }
    }

    Ok(out)
}

/// Converts an OpenAI `/v1/chat/completions` response into an Anthropic `/v1/messages` response.
pub fn openai_to_anthropic_response(openai_resp: &Value, model_override: Option<&str>) -> Result<Value, String> {
    // 1. Handle OpenAI error payloads gracefully
    if let Some(err_obj) = openai_resp.get("error") {
        let msg = err_obj.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown upstream OpenAI error");
        let o_type = err_obj.get("type").and_then(|t| t.as_str()).unwrap_or("");
        let o_code = err_obj.get("code").and_then(|c| c.as_str()).unwrap_or("");

        let ant_type = if o_code == "invalid_api_key" || o_type.contains("auth") || msg.contains("API key") {
            "authentication_error"
        } else if o_code == "rate_limit_exceeded" || o_type.contains("rate_limit") {
            "rate_limit_error"
        } else if o_type == "invalid_request_error" {
            "invalid_request_error"
        } else {
            "api_error"
        };

        return Ok(json!({
            "type": "error",
            "error": {
                "type": ant_type,
                "message": msg
            }
        }));
    }

    let id = openai_resp
        .get("id")
        .and_then(|i| i.as_str())
        .unwrap_or("msg_tokenectomy_transpiled");
    let clean_id = if id.starts_with("msg_") {
        id.to_string()
    } else {
        format!("msg_{}", id)
    };

    let model = model_override
        .or_else(|| openai_resp.get("model").and_then(|m| m.as_str()))
        .unwrap_or("claude-transpiled");

    let choice = openai_resp
        .get("choices")
        .and_then(|c| c.as_array())
        .and_then(|a| a.first());

    let mut content_blocks = Vec::new();
    let mut stop_reason = "end_turn";

    if let Some(ch) = choice {
        if let Some(finish) = ch.get("finish_reason").and_then(|f| f.as_str()) {
            stop_reason = match finish {
                "stop" => "end_turn",
                "tool_calls" => "tool_use",
                "length" => "max_tokens",
                _ => "end_turn",
            };
        }

        if let Some(msg) = ch.get("message") {
            if let Some(text) = msg.get("content").and_then(|c| c.as_str()) {
                if !text.is_empty() {
                    content_blocks.push(json!({
                        "type": "text",
                        "text": text
                    }));
                }
            }

            if let Some(tool_calls) = msg.get("tool_calls").and_then(|tc| tc.as_array()) {
                stop_reason = "tool_use";
                let empty_json = json!({});
                for tc in tool_calls {
                    let tc_id = tc.get("id").and_then(|i| i.as_str()).unwrap_or("call_default");
                    let func = tc.get("function").unwrap_or(&empty_json);
                    let name = func.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                    let args_raw = func.get("arguments").and_then(|a| a.as_str()).unwrap_or("{}");
                    let parsed_args: Value = serde_json::from_str(args_raw).unwrap_or(json!({}));
                    let input_val = if parsed_args.is_object() {
                        parsed_args
                    } else {
                        json!({ "raw": parsed_args })
                    };

                    content_blocks.push(json!({
                        "type": "tool_use",
                        "id": tc_id,
                        "name": name,
                        "input": input_val
                    }));
                }
            }
        }
    }

    let usage = openai_resp.get("usage");
    let input_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|t| t.as_u64()).unwrap_or(0);
    let output_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|t| t.as_u64()).unwrap_or(0);

    Ok(json!({
        "id": clean_id,
        "type": "message",
        "role": "assistant",
        "model": model,
        "content": content_blocks,
        "stop_reason": stop_reason,
        "stop_sequence": Value::Null,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens
        }
    }))
}

/// Converts an OpenAI `/v1/chat/completions` request payload into an Anthropic `/v1/messages` payload.
pub fn openai_to_anthropic_request(
    openai_body: &Value,
    model_override: Option<&str>,
) -> Result<Value, String> {
    let mut system_messages = Vec::new();
    let mut anthropic_messages = Vec::new();

    if let Some(messages) = openai_body.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            let role = msg.get("role").and_then(|r| r.as_str()).unwrap_or("user");
            let content = msg.get("content");

            if role == "system" {
                if let Some(s) = content.and_then(|c| c.as_str()) {
                    system_messages.push(s.to_string());
                }
                continue;
            }

            if role == "tool" {
                let tool_use_id = msg.get("tool_call_id").and_then(|id| id.as_str()).unwrap_or("call_default");
                let text = content.and_then(|c| c.as_str()).unwrap_or("");
                anthropic_messages.push(json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": tool_use_id,
                        "content": text
                    }]
                }));
                continue;
            }

            if let Some(tool_calls) = msg.get("tool_calls").and_then(|tc| tc.as_array()) {
                let mut blocks = Vec::new();
                if let Some(text) = content.and_then(|c| c.as_str()) {
                    if !text.is_empty() {
                        blocks.push(json!({
                            "type": "text",
                            "text": text
                        }));
                    }
                }
                let empty_json = json!({});
                for tc in tool_calls {
                    let id = tc.get("id").and_then(|i| i.as_str()).unwrap_or("call_default");
                    let func = tc.get("function").unwrap_or(&empty_json);
                    let name = func.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                    let args_raw = func.get("arguments").and_then(|a| a.as_str()).unwrap_or("{}");
                    let parsed: Value = serde_json::from_str(args_raw).unwrap_or(json!({}));
                    let input_val = if parsed.is_object() {
                        parsed
                    } else {
                        json!({ "raw": parsed })
                    };

                    blocks.push(json!({
                        "type": "tool_use",
                        "id": id,
                        "name": name,
                        "input": input_val
                    }));
                }
                anthropic_messages.push(json!({
                    "role": "assistant",
                    "content": blocks
                }));
                continue;
            }

            let ant_role = match role {
                "assistant" => "assistant",
                _ => "user",
            };

            let safe_content = match content {
                Some(Value::String(s)) => Value::String(s.clone()),
                Some(Value::Array(arr)) => Value::Array(arr.clone()),
                _ => Value::String(String::new()),
            };

            anthropic_messages.push(json!({
                "role": ant_role,
                "content": safe_content
            }));
        }
    }

    let model = model_override
        .filter(|m| !m.trim().is_empty())
        .or_else(|| openai_body.get("model").and_then(|m| m.as_str()))
        .unwrap_or("claude-3-5-sonnet-latest");

    let max_tokens = openai_body
        .get("max_tokens")
        .or_else(|| openai_body.get("max_completion_tokens"))
        .and_then(|t| t.as_u64())
        .unwrap_or(4096);

    let mut out = json!({
        "model": model,
        "max_tokens": max_tokens,
        "messages": anthropic_messages
    });

    if !system_messages.is_empty() {
        out["system"] = json!(system_messages.join("\n\n"));
    }
    if let Some(temp) = openai_body.get("temperature") {
        out["temperature"] = temp.clone();
    }
    if let Some(top_p) = openai_body.get("top_p") {
        out["top_p"] = top_p.clone();
    }
    if let Some(stream) = openai_body.get("stream") {
        out["stream"] = stream.clone();
    }

    if let Some(tools) = openai_body.get("tools").and_then(|t| t.as_array()) {
        let mut ant_tools = Vec::new();
        let fallback_schema = json!({
            "type": "object",
            "properties": {}
        });
        for t in tools {
            if let Some(func) = t.get("function") {
                let name = func.get("name").and_then(|n| n.as_str()).unwrap_or("tool");
                let desc = func.get("description").and_then(|d| d.as_str()).unwrap_or("");
                let params = func.get("parameters").unwrap_or(&fallback_schema);
                ant_tools.push(json!({
                    "name": name,
                    "description": desc,
                    "input_schema": params.clone()
                }));
            }
        }
        if !ant_tools.is_empty() {
            out["tools"] = Value::Array(ant_tools);
        }
    }

    Ok(out)
}

/// Converts an Anthropic response to an OpenAI response.
pub fn anthropic_to_openai_response(ant_resp: &Value) -> Result<Value, String> {
    // 1. Handle Anthropic error payloads gracefully
    if let Some(err_obj) = ant_resp.get("error") {
        let msg = err_obj.get("message").and_then(|m| m.as_str()).unwrap_or("Unknown upstream Anthropic error");
        let ant_type = err_obj.get("type").and_then(|t| t.as_str()).unwrap_or("api_error");

        return Ok(json!({
            "error": {
                "message": msg,
                "type": ant_type,
                "param": Value::Null,
                "code": Value::Null
            }
        }));
    }

    let id = ant_resp.get("id").and_then(|i| i.as_str()).unwrap_or("chatcmpl-tokenectomy");
    let model = ant_resp.get("model").and_then(|m| m.as_str()).unwrap_or("gpt-4o");

    let mut text_parts = Vec::new();
    let mut tool_calls = Vec::new();

    if let Some(content) = ant_resp.get("content").and_then(|c| c.as_array()) {
        let empty_json = json!({});
        for block in content {
            let b_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if b_type == "text" {
                if let Some(t) = block.get("text").and_then(|x| x.as_str()) {
                    text_parts.push(t);
                }
            } else if b_type == "tool_use" {
                let id = block.get("id").and_then(|i| i.as_str()).unwrap_or("call_default");
                let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("tool");
                let input = block.get("input").unwrap_or(&empty_json);
                let input_str = serde_json::to_string(input).unwrap_or_else(|_| "{}".to_string());
                tool_calls.push(json!({
                    "id": id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": input_str
                    }
                }));
            }
        }
    }

    let finish_reason = match ant_resp.get("stop_reason").and_then(|s| s.as_str()).unwrap_or("end_turn") {
        "end_turn" => "stop",
        "tool_use" => "tool_calls",
        "max_tokens" => "length",
        _ => "stop",
    };

    let mut message = json!({
        "role": "assistant",
        "content": if text_parts.is_empty() && !tool_calls.is_empty() { Value::Null } else { Value::String(text_parts.join("\n")) }
    });

    if !tool_calls.is_empty() {
        message["tool_calls"] = Value::Array(tool_calls);
    }

    let usage = ant_resp.get("usage");
    let prompt_tokens = usage.and_then(|u| u.get("input_tokens")).and_then(|t| t.as_u64()).unwrap_or(0);
    let completion_tokens = usage.and_then(|u| u.get("output_tokens")).and_then(|t| t.as_u64()).unwrap_or(0);

    Ok(json!({
        "id": id,
        "object": "chat.completion",
        "created": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),
        "model": model,
        "choices": [{
            "index": 0,
            "message": message,
            "finish_reason": finish_reason
        }],
        "usage": {
            "prompt_tokens": prompt_tokens,
            "completion_tokens": completion_tokens,
            "total_tokens": prompt_tokens + completion_tokens
        }
    }))
}

/// Transpiles an OpenAI SSE streaming chunk to Anthropic SSE event string format.
pub fn openai_chunk_to_anthropic_sse(chunk: &Value, is_first: bool) -> Vec<String> {
    let mut events = Vec::new();

    if is_first {
        let msg_start = json!({
            "type": "message_start",
            "message": {
                "id": chunk.get("id").and_then(|i| i.as_str()).unwrap_or("msg_stream"),
                "type": "message",
                "role": "assistant",
                "model": chunk.get("model").and_then(|m| m.as_str()).unwrap_or("transpiled-stream"),
                "content": [],
                "stop_reason": Value::Null,
                "stop_sequence": Value::Null,
                "usage": { "input_tokens": 0, "output_tokens": 0 }
            }
        });
        events.push(format!("event: message_start\ndata: {}\n\n", msg_start));

        let block_start = json!({
            "type": "content_block_start",
            "index": 0,
            "content_block": { "type": "text", "text": "" }
        });
        events.push(format!("event: content_block_start\ndata: {}\n\n", block_start));
    }

    if let Some(choices) = chunk.get("choices").and_then(|c| c.as_array()) {
        for ch in choices {
            if let Some(delta) = ch.get("delta") {
                if let Some(txt) = delta.get("content").and_then(|c| c.as_str()) {
                    if !txt.is_empty() {
                        let delta_ev = json!({
                            "type": "content_block_delta",
                            "index": 0,
                            "delta": {
                                "type": "text_delta",
                                "text": txt
                            }
                        });
                        events.push(format!("event: content_block_delta\ndata: {}\n\n", delta_ev));
                    }
                }
            }

            if let Some(finish) = ch.get("finish_reason").and_then(|f| f.as_str()) {
                let stop_reason = match finish {
                    "stop" => "end_turn",
                    "tool_calls" => "tool_use",
                    "length" => "max_tokens",
                    _ => "end_turn",
                };

                let block_stop = json!({
                    "type": "content_block_stop",
                    "index": 0
                });
                events.push(format!("event: content_block_stop\ndata: {}\n\n", block_stop));

                let msg_delta = json!({
                    "type": "message_delta",
                    "delta": {
                        "stop_reason": stop_reason,
                        "stop_sequence": Value::Null
                    },
                    "usage": { "output_tokens": 0 }
                });
                events.push(format!("event: message_delta\ndata: {}\n\n", msg_delta));

                let msg_stop = json!({
                    "type": "message_stop"
                });
                events.push(format!("event: message_stop\ndata: {}\n\n", msg_stop));
            }
        }
    }

    events
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TranspileDirection {
    None,
    AnthropicToOpenAi,
    OpenAiToAnthropic,
}

/// Automatically identifies whether an incoming client request and upstream target URL
/// necessitate bidirectional protocol transpilation.
pub fn detect_transpile_direction(
    path: &str,
    target_url: &str,
    headers: &[(String, String)],
) -> TranspileDirection {
    let target_header = headers.iter().find_map(|(k, v)| {
        if k.eq_ignore_ascii_case("x-tokenectomy-target-provider")
            || k.eq_ignore_ascii_case("x-target-provider")
        {
            Some(v.to_ascii_lowercase())
        } else {
            None
        }
    });

    let force_openai = target_header.as_deref() == Some("openai")
        || target_header.as_deref() == Some("deepseek")
        || target_header.as_deref() == Some("ollama");

    let force_anthropic = target_header.as_deref() == Some("anthropic");

    let target_is_anthropic = target_url.contains("api.anthropic.com") || force_anthropic;
    let target_is_openai_compat = target_url.contains("api.openai.com")
        || target_url.contains("deepseek")
        || target_url.contains("11434")
        || target_url.contains("groq")
        || target_url.contains("openrouter")
        || force_openai;

    if path.starts_with("/v1/messages") || path.starts_with("/v1/complete") {
        if target_is_openai_compat || (!target_is_anthropic && !target_url.is_empty()) {
            return TranspileDirection::AnthropicToOpenAi;
        }
    } else if path.starts_with("/v1/chat/completions") {
        if target_is_anthropic {
            return TranspileDirection::OpenAiToAnthropic;
        }
    }

    TranspileDirection::None
}
