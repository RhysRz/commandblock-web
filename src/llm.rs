//! เรียก LLM ผ่าน API แบบ OpenAI-compatible + จัดการ tool calls
//! ใช้ได้ทั้ง cloud API (OpenRouter/DeepSeek...) และ Ollama ท้องถิ่น
//! รองรับ streaming (พิมพ์คำตอบทีละคำ เหมือนแชทจริง)

use serde_json::{json, Value};

use crate::config::Effective;

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
    /// ข้อมูลเพิ่มเติมที่ต้องส่งกลับไป API รอบถัดไป
    /// (เช่น `thought_signature` ของ Gemini — ถ้าทิ้งจะได้ HTTP 400)
    pub extra: Option<Value>,
}

#[derive(Debug)]
pub struct StreamedResult {
    /// เนื้อหาที่สะสมระหว่าง streaming (พิมพ์ออกจอแล้ว)
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TokenUsage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
    pub exact: bool,
}

fn parse_usage(value: &Value) -> Option<TokenUsage> {
    let usage = value.get("usage")?;
    let prompt_tokens = usage.get("prompt_tokens")?.as_u64()?;
    let completion_tokens = usage.get("completion_tokens")?.as_u64()?;
    let total_tokens = usage
        .get("total_tokens")
        .and_then(Value::as_u64)
        .unwrap_or(prompt_tokens.saturating_add(completion_tokens));
    Some(TokenUsage {
        prompt_tokens,
        completion_tokens,
        total_tokens,
        exact: true,
    })
}

pub fn estimate_usage(messages: &[Value], completion: &str) -> TokenUsage {
    let prompt_chars = serde_json::to_string(messages)
        .unwrap_or_default()
        .chars()
        .count() as u64;
    let completion_chars = completion.chars().count() as u64;
    let prompt_tokens = prompt_chars.div_ceil(4);
    let completion_tokens = completion_chars.div_ceil(4);
    TokenUsage {
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens.saturating_add(completion_tokens),
        exact: false,
    }
}

#[cfg(test)]
mod usage_tests {
    use super::parse_usage;
    use serde_json::json;

    #[test]
    fn usage_from_final_stream_event_is_exact() {
        let usage = parse_usage(&json!({
            "usage": {"prompt_tokens": 12, "completion_tokens": 8, "total_tokens": 20}
        }))
        .expect("usage is present");
        assert_eq!(usage.prompt_tokens, 12);
        assert_eq!(usage.completion_tokens, 8);
        assert_eq!(usage.total_tokens, 20);
        assert!(usage.exact);
    }
}

pub fn ollama_reachable(agent: &ureq::Agent, url: &str) -> bool {
    let u = format!("{}/models", url.trim_end_matches('/'));
    // ลอง 2 ครั้ง เผื่อ Ollama กำลังยุ่ง (กำลังโหลดโมเดล) ตอบช้า
    for attempt in 0..2 {
        let ok = agent
            .get(&u)
            .timeout(std::time::Duration::from_secs(6))
            .call()
            .is_ok();
        if ok {
            return true;
        }
        if attempt == 0 {
            std::thread::sleep(std::time::Duration::from_millis(800));
        }
    }
    false
}

/// เลือกโมเดลที่ดีที่สุดจากโมเดลที่ติดตั้งใน Ollama
/// (ชอบโมเดลสาย coder ก่อน แล้วในกลุ่มเดียวกันเลือกตัวเล็กสุด = เร็วสุดบน CPU)
pub fn pick_model(agent: &ureq::Agent, base_url: &str) -> Option<String> {
    let u = format!("{}/models", base_url.trim_end_matches('/'));
    let resp = agent
        .get(&u)
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .ok()?;
    let text = resp.into_string().ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;

    let mut best: Option<(i32, String)> = None;
    if let Some(arr) = v.get("data").and_then(|d| d.as_array()) {
        for m in arr {
            let id = m["id"].as_str().unwrap_or("").to_string();
            if id.is_empty() {
                continue;
            }
            let score = model_score(&id);
            if best.as_ref().map(|(s, _)| score > *s).unwrap_or(true) {
                best = Some((score, id));
            }
        }
    }
    best.map(|(_, id)| id)
}

fn model_score(id: &str) -> i32 {
    let lower = id.to_lowercase();
    // ระดับ (tier): โมเดลสาย coder ดีที่สุด
    let tier = if lower.contains("qwen2.5-coder")
        || lower.contains("qwen3-coder")
        || lower.contains("codestral")
    {
        100
    } else if lower.contains("deepseek") {
        80
    } else if lower.contains("code") {
        70
    } else if lower.contains("qwen") {
        60
    } else if lower.contains("llama3") || lower.contains("llama-3") {
        50
    } else {
        30
    };
    // ในระดับเดียวกัน เลือกตัวเล็กสุด (เร็วสุดบน CPU)
    let size_b: f32 = [
        "70", "32", "27", "14", "9", "8", "7", "4", "3", "1.5", "0.5",
    ]
    .iter()
    .find(|s| lower.contains(&format!("{}b", s)))
    .and_then(|s| s.parse().ok())
    .unwrap_or(100.0);
    tier - (size_b * 3.0) as i32
}

/// เรียกแชทแบบ streaming — ส่งเนื้อหาทีละคำผ่าน `on_content` และสะสม tool_calls
/// (ผู้เรียกเป็นคนจัดการแสดงผลเอง: CLI พิมพ์จอ / GUI ส่ง SSE)
pub fn chat_stream(
    agent: &ureq::Agent,
    eff: &Effective,
    messages: &[Value],
    tools: &[Value],
    on_content: &mut dyn FnMut(&str),
    on_think: &mut dyn FnMut(&str),
) -> Result<StreamedResult, String> {
    use std::io::BufRead;

    let url = format!("{}/chat/completions", eff.base_url.trim_end_matches('/'));

    let mut body = json!({
        "model": eff.model,
        "messages": messages,
        "temperature": 0.2,
        "stream": true,
    });
    if !tools.is_empty() {
        body["tools"] = json!(tools);
        body["tool_choice"] = json!("auto");
    }
    if eff.base_url.contains("api.deepseek.com") {
        body["stream_options"] = json!({"include_usage": true});
    }

    // ลองส่งใหม่ได้ถ้า API ติดชั่วคราว (429/5xx/เน็ตหลุด) — สำคัญมากสำหรับโมเดลฟรีที่ถูกจำกัด rate
    let mut attempt = 0u32;
    let resp = loop {
        let mut req = agent.post(&url);
        if !eff.api_key.is_empty() {
            req = req.set("Authorization", &format!("Bearer {}", eff.api_key));
        }
        match req.send_json(&body) {
            Ok(r) => break r,
            Err(ureq::Error::Status(code, r)) => {
                // อ่านเวลารอจาก header retry-after (Groq/ทั่วไป) ก่อน แล้วค่อยอ่านจากข้อความ
                let retry_header = r.header("retry-after").and_then(|h| h.parse::<f64>().ok());
                let text = r.into_string().unwrap_or_default();
                let transient = matches!(code, 429 | 500 | 502 | 503 | 504);
                if transient && attempt < 3 {
                    attempt += 1;
                    // ถ้า API บอก "retry in Xs" หรือ header retry-after ให้รอตามนั้น ไม่ใช่รอเดา
                    let wait = retry_header
                        .map(|s| (s.ceil() as u64) + 1)
                        .or_else(|| extract_retry_seconds(&text))
                        .unwrap_or(2u64 * u64::from(attempt)) // 2, 4, 6 วินาที
                        .min(30);
                    println!("(API ติดชั่วคราว HTTP {code} — ลองใหม่ใน {wait} วิ)");
                    std::thread::sleep(std::time::Duration::from_secs(wait));
                    continue;
                }
                let hint = match code {
                    401 => " (ตรวจสอบ API key ใน config.json หรือตัวแปร BUFF_API_KEY)",
                    402 => " (บัญชีไม่มีเครดิต/ยอดเงิน — ต้องเติมเงินก่อน)",
                    404 => " (URL หรือ model ไม่ถูกต้อง — ตรวจ base_url/model)",
                    429 => " (ถูกจำกัดจำนวนคำขอ — รอสักครู่แล้วลองใหม่)",
                    _ => "",
                };
                return Err(format!("[API] HTTP {code}{hint}: {}", truncate(&text, 400)));
            }
            Err(e) => {
                // เน็ตหลุด/เซิร์ฟเวอร์ไม่ตอบ — ลองใหม่ 2 ครั้ง
                if attempt < 2 {
                    attempt += 1;
                    let wait = 2u64 * u64::from(attempt);
                    println!("(เชื่อมต่อขัดข้อง — ลองใหม่ใน {wait} วิ)");
                    std::thread::sleep(std::time::Duration::from_secs(wait));
                    continue;
                }
                return Err(format!(
                    "[API] เชื่อมต่อไม่สำเร็จ: {e}\n   ตรวจการตั้งค่าใน config.json หรือ env (BUFF_BASE_URL / BUFF_API_KEY / BUFF_MODEL) และอินเทอร์เน็ต"
                ));
            }
        }
    };

    let mut reader = std::io::BufReader::new(resp.into_reader());
    let mut content = String::new();
    // tool_calls แบบ streaming: (id, name, arguments, extra_content) ต่อกันทีละชิ้น
    let mut tool_deltas: Vec<(String, String, String, String)> = Vec::new();
    let mut finish = String::new();
    let mut usage = None;

    let mut line = String::new();

    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(e) => return Err(format!("[API] อ่าน stream ไม่ได้: {e}")),
        }
        let l = line.trim();
        if l.is_empty() {
            continue;
        }
        let Some(data) = l.strip_prefix("data: ") else {
            continue; // บรรทัดอื่น (comment/event) ข้ามไป
        };
        if data == "[DONE]" {
            break;
        }
        let Ok(v) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        if let Some(err) = v.get("error") {
            return Err(format!("[API] error: {err}"));
        }

        if let Some(parsed) = parse_usage(&v) {
            usage = Some(parsed);
        }

        let Some(choice) = v.get("choices").and_then(|c| c.get(0)) else {
            continue;
        };
        if let Some(f) = choice.get("finish_reason").and_then(|f| f.as_str()) {
            if !f.is_empty() {
                finish = f.to_string();
            }
        }
        let Some(delta) = choice.get("delta") else {
            continue;
        };

        if let Some(c) = delta.get("content").and_then(|c| c.as_str()) {
            if !c.is_empty() {
                on_content(c);
                content.push_str(c);
            }
        }

        // ความคิดของโมเดล (เช่น Gemini reasoning_content) — ส่งให้ UI แสดงแบบเรียลไทม์
        if let Some(r) = delta
            .get("reasoning_content")
            .or_else(|| delta.get("reasoning"))
            .and_then(|r| r.as_str())
        {
            if !r.is_empty() {
                on_think(r);
            }
        }

        if let Some(arr) = delta.get("tool_calls").and_then(|t| t.as_array()) {
            for tc in arr {
                let idx = tc["index"].as_u64().unwrap_or(0) as usize;
                while tool_deltas.len() <= idx {
                    tool_deltas.push((String::new(), String::new(), String::new(), String::new()));
                }
                if let Some(id) = tc["id"].as_str() {
                    tool_deltas[idx].0.push_str(id);
                }
                if let Some(n) = tc["function"]["name"].as_str() {
                    tool_deltas[idx].1.push_str(n);
                }
                if let Some(a) = tc["function"]["arguments"].as_str() {
                    tool_deltas[idx].2.push_str(a);
                }
                // extra_content (เช่น thought_signature ของ Gemini) — ต่อชิ้นแล้วเก็บไว้
                if let Some(ec) = tc.get("extra_content") {
                    if let Some(s) = ec
                        .get("google")
                        .and_then(|g| g["thought_signature"].as_str())
                    {
                        tool_deltas[idx].3.push_str(s);
                    }
                }
            }
        }
    }

    /// อ่านเวลารอจากข้อความ error เช่น "Please retry in 11.8s" / "try again in 5 seconds"
    fn extract_retry_seconds(text: &str) -> Option<u64> {
        let lower = text.to_lowercase();
        for pat in ["retry in ", "try again in ", "try in "] {
            if let Some(pos) = lower.find(pat) {
                let rest = &lower[pos + pat.len()..];
                let num: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if let Ok(secs) = num.parse::<f64>() {
                    if secs > 0.0 {
                        return Some((secs.ceil() as u64) + 1); // เผื่ออีก 1 วิ
                    }
                }
            }
        }
        None
    }

    let mut tool_calls = Vec::new();
    for (id, name, args, extra_str) in &tool_deltas {
        if name.is_empty() {
            continue;
        }
        let arguments = serde_json::from_str(args).unwrap_or(json!({}));
        let extra = if extra_str.is_empty() {
            None
        } else {
            Some(json!({"google": {"thought_signature": extra_str}}))
        };
        tool_calls.push(ToolCall {
            id: id.clone(),
            name: name.clone(),
            arguments,
            extra,
        });
    }

    Ok(StreamedResult {
        content,
        tool_calls,
        finish,
        usage,
    })
}

/// โมเดลบางตัว (เช่น qwen2.5-coder:3b) ไม่ส่ง structured tool_calls
/// แต่เขียน JSON ในข้อความแทน — ฟังก์ชันนี้จับ JSON แบบนั้นมาใช้ได้
pub fn extract_content_tool_calls(content: &str) -> Option<Vec<ToolCall>> {
    let cleaned = strip_code_fences(content);
    let bytes = cleaned.as_bytes();
    let mut start = 0usize;
    while let Some(i) = cleaned[start..].find('{') {
        let abs = start + i;
        if let Some(end) = find_matching_brace(bytes, abs) {
            let candidate = &cleaned[abs..=end];
            if let Ok(v) = serde_json::from_str::<Value>(candidate) {
                if let Some(tc) = parse_tool_call_obj(&v) {
                    return Some(vec![tc]);
                }
            }
            start = abs + 1;
        } else {
            break;
        }
    }
    None
}

fn strip_code_fences(s: &str) -> String {
    let t = s.trim();
    let t = t.strip_prefix("```json").unwrap_or(t);
    let t = t.strip_prefix("```JSON").unwrap_or(t);
    let t = t.strip_prefix("```").unwrap_or(t);
    t.strip_suffix("```").unwrap_or(t).trim().to_string()
}

fn find_matching_brace(bytes: &[u8], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_str = false;
    let mut esc = false;
    for i in open..bytes.len() {
        let b = bytes[i];
        if in_str {
            if esc {
                esc = false;
            } else if b == b'\\' {
                esc = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_tool_call_obj(v: &Value) -> Option<ToolCall> {
    let name = v
        .get("name")
        .or_else(|| v.get("tool"))
        .or_else(|| v.get("function"))
        .and_then(|n| n.as_str())
        .map(|s| s.trim().to_string())?;
    if !crate::tools::TOOL_NAMES.contains(&name.as_str()) {
        return None;
    }
    // รับ key arguments/args เท่านั้น (parameters คือคำนิยาม schema ไม่ใช่การเรียกใช้)
    let args = v.get("arguments").or_else(|| v.get("args")).cloned();
    let arguments = match args {
        Some(Value::String(s)) => serde_json::from_str(&s).unwrap_or(json!({})),
        Some(o) => o,
        None => json!({}),
    };
    Some(ToolCall {
        id: "content_call".to_string(),
        name,
        arguments,
        extra: None,
    })
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        return s.to_string();
    }
    let mut end = max;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &s[..end])
}
