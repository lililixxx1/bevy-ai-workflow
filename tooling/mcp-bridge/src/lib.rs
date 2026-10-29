//! MCP 薄桥（stdio → BRP 转发）核心库（窗口前置增强 T041 / A1）。
//!
//! 定位：MCP 客户端经 stdio 说话，桥把白名单内的 `tools/call` 逐次转成一次
//! 到本机 BRP（默认 `127.0.0.1:15702`）的 HTTP POST；**仅出站连接、不新增
//! 监听端口**（BRP 无鉴权，硬约束 4 无衰减）。本 crate 不链接 bevy，属游戏外
//! 进程（`bevy-dev/SKILL.md` §3.6），依赖面只有 `serde_json` + std。
//!
//! MCP 帧格式依据（2025-06-18 revision，transports 页 stdio 节）：
//! 消息为换行分隔的 JSON-RPC，UTF-8 编码，单条消息不得含内嵌换行；stdio
//! 不携带 Content-Length 一类帧头。initialize 回显客户端请求的 protocolVersion，
//! capabilities 声明 tools 能力，serverInfo 附本桥标识；`notifications/*` 一律
//! 无响应；未知请求方法回 JSON-RPC -32601；ping 回空结果对象。
//!
//! 工具面 = 精确 12 个（8 个 `game.*` + 4 个只读 BRP），顺序即
//! [`ALLOWED_TOOLS`]；白名单外方法名一律拒绝转发（含一切写通道与 +watch
//! 流式方法）。工具名与 BRP 方法名逐字相同——2025-06-18 schema 的 Tool.name
//! 只声明为字符串，未限制字符集，点号合法。
//!
//! 转发语义：MCP `arguments` 缺失 / null / 空对象三态 → BRP 请求体省略
//! `params` 键（`game.snapshot` 对 `params:null` 报 -32602，T040 审核轮 S2 注）；
//! 其余对象原样作 `params`（不增删改键）。BRP HTTP 响应体逐字节原文进
//! `content[0].text`（成功与 JSON-RPC 错误同规），`isError` 由响应体是否为
//! 含 `error` 键的 JSON 判定；桥自身传输层失败（连接拒绝 / 超时 / 非 JSON
//! 响应）→ `isError:true` + 桥侧命名空间 JSON（`bridge_error` / `code` -32000）。
//! 桥启动后直接进入 stdio 循环，不探测 BRP——游戏未启动时桥进程存活、逐调用
//! 报 bridge_error。
//!
//! 依赖与安全：BRP 地址默认 `127.0.0.1:15702`，可用环境变量
//! [`BRP_ADDR_ENV`] 覆盖，但主机字面必须是 `127.0.0.1` 或 `localhost`，
//! 否则启动即报错退出非 0（只允许回环，禁任何其它主机）。

use std::io::{BufRead, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use serde_json::{json, Value};

/// BRP 默认地址（bevy_remote 主 app 端口；`bevy-dev/SKILL.md` §6.1）。
pub const DEFAULT_BRP_ADDR: &str = "127.0.0.1:15702";

/// 覆盖 BRP 地址的环境变量（host:port 形式，主机限回环字面）。
pub const BRP_ADDR_ENV: &str = "MCP_BRIDGE_BRP_ADDR";

/// initialize 请求未携带可用 protocolVersion 时的回退值（2025-06-18 revision）。
pub const PROTOCOL_VERSION_FALLBACK: &str = "2025-06-18";

/// serverInfo 的 name 字段（`.mcp.json` 注册名同源语义）。
pub const SERVER_NAME: &str = "bevy-brp-bridge";

/// serverInfo 的 version 字段（取本 crate 包版本，不手写版本字面量）。
pub const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 桥侧传输错误的命名空间码（`bridge_error` JSON 的 code 字段）。
pub const BRIDGE_ERROR_CODE: i64 = -32000;

/// 工具白名单（精确 12 个；顺序 = `tools/list` 输出顺序 = 证据断言顺序）。
///
/// 前 8 个为游戏专属方法（来源 `game/src/rpc/mod.rs` 的 8 个 `pub const`，
/// 计数由 claim-lint 锚定）；后 4 个为 bevy_remote 内置只读方法
/// （`bevy-dev/SKILL.md` §6.2 的 23 内置清单中的只读子集）。
pub const ALLOWED_TOOLS: [&str; 12] = [
    "game.run_tests",
    "game.screenshot",
    "game.screenshot_log",
    "game.launch_level",
    "game.move_unit",
    "game.attack",
    "game.end_turn",
    "game.snapshot",
    "rpc.discover",
    "world.query",
    "world.get_components",
    "registry.schema",
];

/// 是否在白名单内（逐字精确匹配；大小写与前后缀均不放行）。
pub fn is_allowed(method: &str) -> bool {
    ALLOWED_TOOLS.contains(&method)
}

/// BRP 目标地址（仅回环字面）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrpAddr {
    host: String,
    port: u16,
}

impl BrpAddr {
    /// 解析 host:port 并校验主机字面（只允许 `127.0.0.1` 或 `localhost`）。
    pub fn parse(raw: &str) -> Result<Self, String> {
        let (host, port_str) = raw
            .rsplit_once(':')
            .ok_or_else(|| format!("地址须为 host:port 形式：{raw}"))?;
        if host.is_empty() {
            return Err(format!("地址缺主机：{raw}"));
        }
        // 硬约束：BRP 无鉴权，只允许回环字面；不解析 DNS，不放行任何其它主机。
        if host != "127.0.0.1" && host != "localhost" {
            return Err(format!(
                "非回环主机被拒绝：{host}（仅允许 127.0.0.1 或 localhost 字面）"
            ));
        }
        let port: u16 = port_str
            .parse()
            .map_err(|e| format!("端口非 u16（{raw}）：{e}"))?;
        if port == 0 {
            return Err(format!("端口 0 不可用：{raw}"));
        }
        Ok(Self {
            host: host.to_string(),
            port,
        })
    }

    /// 从环境变量读取（缺省用 [`DEFAULT_BRP_ADDR`]）；非法值返回 Err，由 main 退出非 0。
    pub fn from_env() -> Result<Self, String> {
        match std::env::var(BRP_ADDR_ENV) {
            Ok(raw) => Self::parse(&raw),
            Err(std::env::VarError::NotPresent) => Self::parse(DEFAULT_BRP_ADDR),
            Err(e) => Err(format!("读取环境变量 {BRP_ADDR_ENV} 失败：{e}")),
        }
    }

    /// 主机字面。
    pub fn host(&self) -> &str {
        &self.host
    }

    /// 端口。
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Host 头形态（保留用户给的字面，`localhost` 不被改写成 IP）。
    pub fn host_header(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn socket_addr(&self) -> SocketAddr {
        // 两个合法主机字面都显式落到 IPv4 回环，避免解析顺序引入 ::1 一类非目标面。
        SocketAddr::from(([127, 0, 0, 1], self.port))
    }
}

impl std::fmt::Display for BrpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}

/// BRP 调用传输面（便于单测注入 mock；生产实现见 [`HttpBrp`]）。
pub trait BrpTransport {
    /// 发起一次 BRP 调用；`params` 为 None 表示请求体省略 `params` 键。
    /// 成功返回 HTTP 响应体原文（UTF-8）；失败返回桥侧原因（进 bridge_error）。
    fn call(&mut self, method: &str, params: Option<&Value>) -> Result<String, String>;
}

/// std-only 的 BRP over HTTP 客户端（每次调用独立短连接，`Connection: close`）。
pub struct HttpBrp {
    addr: BrpAddr,
    next_id: u64,
    connect_timeout: Duration,
    read_timeout: Duration,
}

impl HttpBrp {
    /// 以目标地址构造（超时口径：连接 5s / 读 30s，均硬上限）。
    pub fn new(addr: BrpAddr) -> Self {
        Self {
            addr,
            next_id: 1,
            connect_timeout: Duration::from_secs(5),
            read_timeout: Duration::from_secs(30),
        }
    }
}

impl BrpTransport for HttpBrp {
    fn call(&mut self, method: &str, params: Option<&Value>) -> Result<String, String> {
        let id = self.next_id;
        self.next_id += 1;
        let mut request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "id": id,
        });
        if let Some(params) = params {
            request["params"] = params.clone();
        }
        let body = request.to_string();
        http_post(
            &self.addr,
            &body,
            self.connect_timeout,
            self.read_timeout,
        )
    }
}

/// MCP 会话状态机（传输注入；stdio 循环与单测共用同一路径）。
pub struct Session<T: BrpTransport> {
    transport: T,
}

impl<T: BrpTransport> Session<T> {
    /// 以给定传输面构造。
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// 只读访问传输面（单测观察转发记录）。
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// 处理一行输入；返回待输出的一行 JSON 消息（None = 通知/无响应）。
    pub fn handle_line(&mut self, line: &str) -> Option<Value> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }
        let message: Value = match serde_json::from_str(trimmed) {
            Ok(value) => value,
            Err(e) => {
                return Some(error_response(
                    Value::Null,
                    -32700,
                    format!("Parse error: {e}"),
                ))
            }
        };
        let Some(object) = message.as_object() else {
            return Some(error_response(
                Value::Null,
                -32600,
                "Invalid Request: 顶层须为 JSON 对象".to_string(),
            ));
        };
        let id = object.get("id").cloned();
        let Some(method) = object.get("method").and_then(Value::as_str) else {
            // 桥不主动发起请求；客户端回传的响应/畸形消息一律忽略。
            return None;
        };
        // 通知一律无响应（含 notifications/initialized 与未知 notifications/*）。
        if method.starts_with("notifications/") {
            return None;
        }
        let Some(id) = id else {
            // 无 id 的请求形态按 JSON-RPC 通知处理（不得回包）。
            return None;
        };
        let params = object.get("params").cloned().unwrap_or(Value::Null);
        match method {
            "initialize" => Some(ok_response(id, initialize_result(&params))),
            "ping" => Some(ok_response(id, json!({}))),
            "tools/list" => Some(ok_response(id, json!({ "tools": tool_definitions() }))),
            "tools/call" => Some(self.tools_call(id, &params)),
            _ => Some(error_response(
                id,
                -32601,
                format!("Method not found: {method}"),
            )),
        }
    }

    /// stdio 主循环：逐行读入、逐行写出（stdout 只写合法 MCP 消息）。
    pub fn run<R: BufRead, W: Write>(
        &mut self,
        mut input: R,
        output: &mut W,
    ) -> std::io::Result<()> {
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if input.read_line(&mut buffer)? == 0 {
                break;
            }
            if let Some(response) = self.handle_line(&buffer) {
                let line = serde_json::to_string(&response).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, e)
                })?;
                output.write_all(line.as_bytes())?;
                output.write_all(b"\n")?;
                output.flush()?;
            }
        }
        Ok(())
    }

    fn tools_call(&mut self, id: Value, params: &Value) -> Value {
        let Some(params) = params.as_object() else {
            return error_response(
                id,
                -32602,
                "Invalid params: tools/call 的 params 须为对象".to_string(),
            );
        };
        let Some(name) = params.get("name").and_then(Value::as_str) else {
            return error_response(id, -32602, "Invalid params: 缺 name".to_string());
        };
        if !is_allowed(name) {
            return error_response(
                id,
                -32602,
                format!("白名单拒绝：{name} 不在 12 工具白名单内（桥不转发）"),
            );
        }
        // arguments 三态（缺失 / null / 空对象）→ 省略 BRP params；其余对象原样转发。
        let forwarded: Option<&Value> = match params.get("arguments") {
            None => None,
            Some(Value::Null) => None,
            Some(Value::Object(map)) if map.is_empty() => None,
            Some(value @ Value::Object(_)) => Some(value),
            Some(_) => {
                return error_response(
                    id,
                    -32602,
                    "Invalid params: arguments 须为对象或缺省/null".to_string(),
                )
            }
        };
        match self.transport.call(name, forwarded) {
            Ok(text) => match serde_json::from_str::<Value>(&text) {
                Ok(Value::Object(map)) => {
                    let is_error = map.contains_key("error");
                    ok_response(id, tool_result(text, is_error))
                }
                Ok(_) => {
                    let text = bridge_error_text("BRP 响应不是 JSON 对象");
                    ok_response(id, tool_result(text, true))
                }
                Err(e) => {
                    let text = bridge_error_text(&format!("BRP 响应不是有效 JSON：{e}"));
                    ok_response(id, tool_result(text, true))
                }
            },
            Err(reason) => {
                let text = bridge_error_text(&reason);
                ok_response(id, tool_result(text, true))
            }
        }
    }
}

fn initialize_result(params: &Value) -> Value {
    let version = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or(PROTOCOL_VERSION_FALLBACK);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION },
    })
}

fn tool_definitions() -> Vec<Value> {
    ALLOWED_TOOLS
        .iter()
        .map(|name| {
            json!({
                "name": name,
                "description": format!(
                    "原样转发到本机 Bevy BRP 方法 {name}（arguments 即 params；缺省或空对象省略 params）"
                ),
                "inputSchema": { "type": "object", "properties": {} },
            })
        })
        .collect()
}

fn ok_response(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error_response(id: Value, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

fn tool_result(text: String, is_error: bool) -> Value {
    json!({
        "content": [ { "type": "text", "text": text } ],
        "isError": is_error,
    })
}

fn bridge_error_text(reason: &str) -> String {
    json!({ "bridge_error": reason, "code": BRIDGE_ERROR_CODE }).to_string()
}

/// 极简 HTTP/1.1 POST（`Connection: close`，读至 EOF；解析 Content-Length 与
/// chunked 两种响应体；非 200 视为传输层失败）。
fn http_post(
    addr: &BrpAddr,
    body: &str,
    connect_timeout: Duration,
    read_timeout: Duration,
) -> Result<String, String> {
    let mut stream = TcpStream::connect_timeout(&addr.socket_addr(), connect_timeout)
        .map_err(|e| format!("连接 {addr} 失败：{e}（游戏未启动？）"))?;
    stream
        .set_read_timeout(Some(read_timeout))
        .map_err(|e| format!("设置读超时失败：{e}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| format!("设置写超时失败：{e}"))?;
    let request = format!(
        "POST / HTTP/1.1\r\nHost: {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        addr.host_header(),
        body.len()
    );
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("发送请求失败：{e}"))?;
    stream
        .flush()
        .map_err(|e| format!("刷新请求失败：{e}"))?;
    let mut raw = Vec::new();
    stream
        .read_to_end(&mut raw)
        .map_err(|e| format!("读取响应失败：{e}"))?;
    parse_http_response(&raw)
}

fn parse_http_response(raw: &[u8]) -> Result<String, String> {
    let Some(split) = raw.windows(4).position(|w| w == b"\r\n\r\n") else {
        return Err(format!(
            "响应无头部分隔（{} 字节）：{:?}",
            raw.len(),
            String::from_utf8_lossy(&raw[..raw.len().min(120)])
        ));
    };
    let head = String::from_utf8_lossy(&raw[..split]);
    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("响应状态行不可解析：{status_line}"))?;
    if status != 200 {
        return Err(format!("BRP HTTP 状态 {status}（期望 200）"));
    }
    let mut chunked = false;
    let mut content_length: Option<usize> = None;
    for line in lines {
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("transfer-encoding:") {
            if value.contains("chunked") {
                chunked = true;
            }
        }
        if let Some(value) = lower.strip_prefix("content-length:") {
            content_length = value.trim().parse().ok();
        }
    }
    let body = &raw[split + 4..];
    let bytes = if chunked {
        dechunk(body)
    } else if let Some(length) = content_length {
        body[..body.len().min(length)].to_vec()
    } else {
        body.to_vec()
    };
    String::from_utf8(bytes).map_err(|e| format!("响应体非 UTF-8：{e}"))
}

/// chunked 传输解码（字节级；chunk 扩展忽略；0 大小块即止）。
fn dechunk(mut rest: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let Some(line_end) = rest.windows(2).position(|w| w == b"\r\n") else {
            break;
        };
        let size_str = String::from_utf8_lossy(&rest[..line_end]);
        let size_part = size_str.split(';').next().unwrap_or("").trim();
        let Ok(size) = usize::from_str_radix(size_part, 16) else {
            break;
        };
        if size == 0 {
            break;
        }
        let start = line_end + 2;
        if start + size > rest.len() {
            out.extend_from_slice(&rest[start..]);
            break;
        }
        out.extend_from_slice(&rest[start..start + size]);
        let mut next = start + size;
        if rest[next..].starts_with(b"\r\n") {
            next += 2;
        }
        rest = &rest[next..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpListener;
    use std::sync::mpsc::{self, Receiver};
    use std::thread::{self, JoinHandle};

    /// 记录型传输面：断言转发参数、或在拒绝分支证明零调用。
    struct Recording {
        calls: Vec<(String, Option<Value>)>,
        response: Result<String, String>,
    }

    impl Default for Recording {
        fn default() -> Self {
            Self {
                calls: Vec::new(),
                response: Ok(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()),
            }
        }
    }

    impl Recording {
        fn ok(body: &str) -> Self {
            Self {
                calls: Vec::new(),
                response: Ok(body.to_string()),
            }
        }
    }

    impl BrpTransport for Recording {
        fn call(&mut self, method: &str, params: Option<&Value>) -> Result<String, String> {
            self.calls.push((method.to_string(), params.cloned()));
            self.response.clone()
        }
    }

    fn call_line(id: u64, name: &str, arguments: Option<Value>) -> String {
        let mut params = json!({ "name": name });
        if let Some(arguments) = arguments {
            params["arguments"] = arguments;
        }
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "tools/call",
            "params": params,
        })
        .to_string()
    }

    /// 启动一次性的 mock BRP：读完整请求后回给定原始响应字节。
    fn start_mock(response: String) -> (u16, Receiver<String>, JoinHandle<()>) {
        let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind mock");
        let port = listener.local_addr().expect("local_addr").port();
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            stream
                .set_read_timeout(Some(Duration::from_secs(5)))
                .expect("set_read_timeout");
            let request = read_http_request(&mut stream);
            tx.send(String::from_utf8(request).expect("request utf8"))
                .expect("send request");
            stream.write_all(response.as_bytes()).expect("write response");
            stream.flush().expect("flush response");
        });
        (port, rx, handle)
    }

    fn read_http_request(stream: &mut TcpStream) -> Vec<u8> {
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        let header_end = loop {
            let n = stream.read(&mut chunk).expect("read request");
            assert!(n > 0, "请求在读满头部前结束");
            buf.extend_from_slice(&chunk[..n]);
            if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                break pos;
            }
        };
        let head = String::from_utf8_lossy(&buf[..header_end]).to_ascii_lowercase();
        let content_length: usize = head
            .split("\r\n")
            .find_map(|line| line.strip_prefix("content-length:"))
            .and_then(|v| v.trim().parse().ok())
            .expect("Content-Length");
        while buf.len() < header_end + 4 + content_length {
            let n = stream.read(&mut chunk).expect("read body");
            assert!(n > 0, "请求体在声明长度前结束");
            buf.extend_from_slice(&chunk[..n]);
        }
        buf
    }

    fn http_response(body: &str) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn chunked_response(body: &str) -> String {
        let bytes = body.as_bytes();
        let mut framed = String::new();
        for piece in bytes.chunks(7) {
            framed.push_str(&format!("{:x}\r\n", piece.len()));
            framed.push_str(std::str::from_utf8(piece).expect("utf8 piece"));
            framed.push_str("\r\n");
        }
        framed.push_str("0\r\n\r\n");
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n{framed}"
        )
    }

    #[test]
    fn brp_addr_accepts_only_loopback_literals() {
        let ok = BrpAddr::parse("127.0.0.1:15702").expect("loopback ip");
        assert_eq!(ok.host(), "127.0.0.1");
        assert_eq!(ok.port(), 15702);
        let ok = BrpAddr::parse("localhost:15702").expect("localhost literal");
        assert_eq!(ok.host(), "localhost");

        for bad in [
            "0.0.0.0:15702",
            "192.168.1.10:15702",
            "example.com:15702",
            "127.0.0.2:15702",
            "[::1]:15702",
            "127.0.0.1",
            "127.0.0.1:notaport",
            "127.0.0.1:0",
        ] {
            let err = BrpAddr::parse(bad).expect_err(bad);
            assert!(!err.is_empty(), "{bad} 拒绝原因为空");
        }
    }

    #[test]
    fn initialize_echoes_protocol_version_and_declares_tools() {
        let mut session = Session::new(Recording::default());
        let line = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "probe", "version": "0.0.0" }
            }
        })
        .to_string();
        let response = session.handle_line(&line).expect("initialize 须有响应");
        assert_eq!(response["result"]["protocolVersion"], "2025-06-18");
        assert!(response["result"]["capabilities"]["tools"].is_object());
        assert_eq!(response["result"]["serverInfo"]["name"], SERVER_NAME);
    }

    #[test]
    fn ping_returns_empty_result_object() {
        let mut session = Session::new(Recording::default());
        let response = session
            .handle_line(r#"{"jsonrpc":"2.0","id":7,"method":"ping"}"#)
            .expect("ping 须有响应");
        assert_eq!(response["id"], 7);
        assert_eq!(response["result"], json!({}));
    }

    #[test]
    fn notifications_get_no_response_and_unknown_method_is_32601() {
        let mut session = Session::new(Recording::default());
        assert!(session
            .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#)
            .is_none());
        assert!(session
            .handle_line(r#"{"jsonrpc":"2.0","method":"notifications/whatever"}"#)
            .is_none());
        let response = session
            .handle_line(r#"{"jsonrpc":"2.0","id":9,"method":"foo/bar"}"#)
            .expect("未知请求须有错误响应");
        assert_eq!(response["error"]["code"], -32601);
        let response = session
            .handle_line(r#"{"jsonrpc":"2.0","id":10,"method":"foo/bar","params":{}}"#)
            .expect("未知请求须有错误响应");
        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn tools_list_is_exactly_the_twelve_allowlisted_names_in_order() {
        let mut session = Session::new(Recording::default());
        let response = session
            .handle_line(r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#)
            .expect("tools/list 须有响应");
        let tools = response["result"]["tools"].as_array().expect("tools 数组");
        assert_eq!(tools.len(), 12);
        let names: Vec<&str> = tools
            .iter()
            .map(|tool| tool["name"].as_str().expect("tool name"))
            .collect();
        assert_eq!(names, ALLOWED_TOOLS.to_vec());
        for tool in tools {
            assert!(tool["inputSchema"]["type"] == "object");
        }
    }

    #[test]
    fn params_three_states_are_omitted_and_objects_forwarded_verbatim() {
        let mut session = Session::new(Recording::ok(r#"{"jsonrpc":"2.0","id":1,"result":{}}"#));
        let missing = json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": { "name": "game.snapshot" }
        })
        .to_string();
        session.handle_line(&missing);
        let null_args = call_line(2, "game.snapshot", Some(Value::Null));
        session.handle_line(&null_args);
        let empty_args = call_line(3, "game.snapshot", Some(json!({})));
        session.handle_line(&empty_args);
        let object_args = call_line(4, "game.run_tests", Some(json!({ "suite": "ts-01" })));
        session.handle_line(&object_args);

        let calls = &session.transport().calls;
        assert_eq!(calls.len(), 4);
        assert_eq!(calls[0].1, None);
        assert_eq!(calls[1].1, None);
        assert_eq!(calls[2].1, None);
        assert_eq!(calls[3].1, Some(json!({ "suite": "ts-01" })));
    }

    #[test]
    fn non_object_arguments_are_rejected_without_forwarding() {
        let mut session = Session::new(Recording::default());
        let line = call_line(5, "game.run_tests", Some(json!([1, 2, 3])));
        let response = session.handle_line(&line).expect("须有错误响应");
        assert_eq!(response["error"]["code"], -32602);
        assert!(session.transport().calls.is_empty());
    }

    #[test]
    fn whitelist_rejection_does_not_reach_brp() {
        let mut session = Session::new(Recording::default());
        for name in [
            "world.mutate_components",
            "world.spawn_entity",
            "world.insert_components",
            "world.remove_components",
            "world.despawn_entity",
            "world.reparent_entities",
            "world.trigger_event",
            "world.write_message",
            "world.observe+watch",
            "world.query+watch",
            "world.get_components+watch",
            "world.insert_resources",
            "world.remove_resources",
            "world.mutate_resources",
        ] {
            let response = session
                .handle_line(&call_line(6, name, Some(json!({}))))
                .expect("白名单外须有错误响应");
            assert_eq!(response["error"]["code"], -32602, "{name}");
            let message = response["error"]["message"].as_str().expect("message");
            assert!(message.contains("白名单拒绝"), "{name}: {message}");
        }
        assert!(session.transport().calls.is_empty());
    }

    #[test]
    fn success_response_is_forwarded_byte_for_byte() {
        let body = r#"{"jsonrpc":"2.0","id":1,"result":{"passed":45,"failed":0}}"#;
        let (port, request_rx, handle) = start_mock(http_response(body));
        let addr = BrpAddr::parse(&format!("127.0.0.1:{port}")).expect("mock addr");
        let mut session = Session::new(HttpBrp::new(addr));
        let response = session
            .handle_line(&call_line(
                11,
                "game.run_tests",
                Some(json!({ "suite": "ts-01" })),
            ))
            .expect("须有响应");
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("text");
        assert_eq!(text, body);
        assert_eq!(response["result"]["isError"], false);
        assert_eq!(response["result"]["content"][0]["type"], "text");
        let request = request_rx.recv_timeout(Duration::from_secs(5)).expect("request");
        assert!(request.contains("\"method\":\"game.run_tests\""), "{request}");
        assert!(request.contains("\"params\":{\"suite\":\"ts-01\"}"), "{request}");
        handle.join().expect("mock join");
    }

    #[test]
    fn jsonrpc_error_body_is_passed_through_with_iserror_true() {
        let body =
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32602,"message":"params 须为空对象或缺省"}}"#;
        let (port, _request_rx, handle) = start_mock(http_response(body));
        let addr = BrpAddr::parse(&format!("127.0.0.1:{port}")).expect("mock addr");
        let mut session = Session::new(HttpBrp::new(addr));
        let response = session
            .handle_line(&call_line(12, "game.snapshot", Some(json!({ "bad": true }))))
            .expect("须有响应");
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("text");
        assert_eq!(text, body);
        assert_eq!(response["result"]["isError"], true);
        assert!(text.contains("-32602"), "{text}");
        handle.join().expect("mock join");
    }

    #[test]
    fn chunked_response_is_decoded() {
        let body = r#"{"jsonrpc":"2.0","id":1,"result":{"methods":["rpc.discover"]}}"#;
        let (port, _request_rx, handle) = start_mock(chunked_response(body));
        let addr = BrpAddr::parse(&format!("127.0.0.1:{port}")).expect("mock addr");
        let mut session = Session::new(HttpBrp::new(addr));
        let response = session
            .handle_line(&call_line(13, "rpc.discover", None))
            .expect("须有响应");
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("text");
        assert_eq!(text, body);
        assert_eq!(response["result"]["isError"], false);
        handle.join().expect("mock join");
    }

    #[test]
    fn non_json_response_is_bridge_error() {
        let (port, _request_rx, handle) = start_mock(http_response("not-json"));
        let addr = BrpAddr::parse(&format!("127.0.0.1:{port}")).expect("mock addr");
        let mut session = Session::new(HttpBrp::new(addr));
        let response = session
            .handle_line(&call_line(15, "game.snapshot", None))
            .expect("须有响应");
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("text");
        let parsed: Value = serde_json::from_str(text).expect("bridge error json");
        assert_eq!(parsed["code"], BRIDGE_ERROR_CODE);
        assert!(parsed["bridge_error"].as_str().is_some());
        assert_eq!(response["result"]["isError"], true);
        handle.join().expect("mock join");
    }

    #[test]
    fn connection_refused_is_bridge_error() {
        let port = {
            let listener = TcpListener::bind(("127.0.0.1", 0)).expect("probe bind");
            listener.local_addr().expect("probe addr").port()
        };
        let addr = BrpAddr::parse(&format!("127.0.0.1:{port}")).expect("mock addr");
        let mut session = Session::new(HttpBrp::new(addr));
        let response = session
            .handle_line(&call_line(21, "game.snapshot", None))
            .expect("须有响应");
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("text");
        let parsed: Value = serde_json::from_str(text).expect("bridge error json");
        assert_eq!(parsed["code"], BRIDGE_ERROR_CODE);
        assert_eq!(response["result"]["isError"], true);
    }

    #[test]
    fn stdio_loop_handles_multiple_lines_and_skips_notifications() {
        let body = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
        let mut session = Session::new(Recording::ok(body));
        let input = concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"protocolVersion\":\"2025-06-18\",\"capabilities\":{},\"clientInfo\":{\"name\":\"p\",\"version\":\"0\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"foo/bar\"}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":4,\"method\":\"tools/call\",\"params\":{\"name\":\"game.snapshot\"}}\n",
        );
        let mut output = Vec::new();
        session
            .run(std::io::Cursor::new(input), &mut output)
            .expect("stdio loop");
        let text = String::from_utf8(output).expect("utf8 output");
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 4, "通知不得产生响应行：{text}");
        let ids: Vec<Value> = lines
            .iter()
            .map(|line| serde_json::from_str::<Value>(line).expect("json line")["id"].clone())
            .collect();
        assert_eq!(ids, vec![json!(1), json!(2), json!(3), json!(4)]);
        assert_eq!(session.transport().calls.len(), 1);
        assert_eq!(session.transport().calls[0].0, "game.snapshot");
        assert_eq!(session.transport().calls[0].1, None);
    }

    #[test]
    fn malformed_line_returns_parse_error_with_null_id() {
        let mut session = Session::new(Recording::default());
        let response = session.handle_line("{not json").expect("须有错误响应");
        assert_eq!(response["error"]["code"], -32700);
        assert!(response["id"].is_null());
    }
}
