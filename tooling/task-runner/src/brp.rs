//! BRP over HTTP 的极简客户端（std-only，无外部 HTTP crate——依赖面口径见
//! Cargo.toml 注释）。
//!
//! 协议事实依据（bevy_remote-0.19.1，SKILL.md §6.4）：JSON-RPC 2.0 over HTTP
//! POST；请求必须含 `"jsonrpc":"2.0"` / `method` / `id`；成功响应含 `result`，
//! 失败含 `error`（`code`/`message` 恒在）。

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use serde_json::{json, Value};

/// BRP 客户端（每次调用独立短连接：`Connection: close`，回避 keep-alive 状态）。
pub struct BrpClient {
    host: String,
    port: u16,
    next_id: u64,
}

impl BrpClient {
    /// 从 `http://host:port` 形式构造（缺端口默认 15702，对齐 BRP 默认口）。
    pub fn new(url: &str) -> Result<Self, String> {
        let rest = url
            .strip_prefix("http://")
            .ok_or_else(|| format!("仅支持 http:// URL（BRP 仅回环明文）：{url}"))?;
        let (host, port) = match rest.rsplit_once(':') {
            Some((h, p)) => (
                h.to_string(),
                p.parse().map_err(|e| format!("端口非数字（{url}）：{e}"))?,
            ),
            None => (rest.to_string(), 15702),
        };
        Ok(Self {
            host,
            port,
            next_id: 1,
        })
    }

    /// 发起一次 JSON-RPC 调用；打印请求/响应逐字留痕（stdout 即证据日志）。
    /// 协议层失败（连接/解析/`error` 对象）返回 Err；正常返回 `result`。
    pub fn call(&mut self, method: &str, params: Value) -> Result<Value, String> {
        let id = self.next_id;
        self.next_id += 1;
        let body = json!({
            "jsonrpc": "2.0",
            "method": method,
            "id": id,
            "params": params,
        })
        .to_string();
        println!(">> [{id}] {method} params={params}");
        let raw = http_post(&self.host, self.port, &body)?;
        let value: Value =
            serde_json::from_str(&raw).map_err(|e| format!("响应不是有效 JSON：{e}\n{raw}"))?;
        println!("<< [{id}] {value}");
        if let Some(error) = value.get("error") {
            return Err(format!("BRP error（{method}）：{error}"));
        }
        Ok(value.get("result").cloned().unwrap_or(Value::Null))
    }
}

/// 极简 HTTP/1.1 POST（`Connection: close`，读至 EOF；支持 Content-Length 与
/// chunked 两种响应体——按头部决定，chunked 按字节解码避免 UTF-8 边界断裂）。
fn http_post(host: &str, port: u16, body: &str) -> Result<String, String> {
    let mut stream = TcpStream::connect((host, port))
        .map_err(|e| format!("连接 {host}:{port} 失败：{e}（游戏未启动？）"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("设置读超时失败：{e}"))?;
    let request = format!(
        "POST / HTTP/1.1\r\nHost: {host}:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.as_bytes().len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("发送请求失败：{e}"))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("读取响应失败：{e}"))?;

    let Some(split) = raw.windows(4).position(|w| w == b"\r\n\r\n") else {
        return Err(format!(
            "响应无头部分隔（{} 字节）：{:?}",
            raw.len(),
            String::from_utf8_lossy(&raw[..raw.len().min(120)])
        ));
    };
    let head = String::from_utf8_lossy(&raw[..split]).to_ascii_lowercase();
    let body_bytes = &raw[split + 4..];
    let body = if head.contains("transfer-encoding:") && head.contains("chunked") {
        dechunk(body_bytes)
    } else {
        body_bytes.to_vec()
    };
    Ok(String::from_utf8_lossy(&body).into_owned())
}

/// chunked 传输解码（字节级；结尾 0 大小块即止）。
fn dechunk(mut rest: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let Some(line_end) = rest.windows(2).position(|w| w == b"\r\n") else {
            break;
        };
        let size_str = String::from_utf8_lossy(&rest[..line_end]);
        let Ok(size) = usize::from_str_radix(size_str.trim(), 16) else {
            break;
        };
        if size == 0 {
            break;
        }
        let start = line_end + 2;
        let end = (start + size).min(rest.len());
        out.extend_from_slice(&rest[start..end]);
        rest = &rest[(end + 2).min(rest.len())..];
    }
    out
}
