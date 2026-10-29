# MCP 2025-06-18 规范原文摘录（T041 桥实现依据）

- 抓取日期：2026-10-28（UTC+8）
- 来源（官方规范站，2025-06-18 revision）：
  - Transports：<https://modelcontextprotocol.io/specification/2025-06-18/basic/transports>
  - Tools：<https://modelcontextprotocol.io/specification/2025-06-18/server/tools>
  - Schema Reference：<https://modelcontextprotocol.io/specification/2025-06-18/schema>
- 用途：`tooling/mcp-bridge` 的帧格式、initialize/tools 形态与工具名语法的唯一规范依据。
  与本地实录 `tooling/brp-logs/10-mcp-session-transcript.jsonl`（T004，rmcp 3.3.0，
  protocolVersion 2025-06-18）双源互证；两源无冲突（见文末对照）。

## 1. stdio 传输（transports 页「stdio」节，逐字摘录）

> MCP uses JSON-RPC to encode messages. JSON-RPC messages **MUST** be UTF-8 encoded.

> In the **stdio** transport:
>
> * The client launches the MCP server as a subprocess.
> * The server reads JSON-RPC messages from its standard input (`stdin`) and sends messages
>   to its standard output (`stdout`).
> * Messages are individual JSON-RPC requests, notifications, or responses.
> * Messages are delimited by newlines, and **MUST NOT** contain embedded newlines.
> * The server **MAY** write UTF-8 strings to its standard error (`stderr`) for logging
>   purposes. Clients **MAY** capture, forward, or ignore this logging.
> * The server **MUST NOT** write anything to its `stdout` that is not a valid MCP message.
> * The client **MUST NOT** write anything to the server's `stdin` that is not a valid MCP
>   message.

结论（预注册要点核实）：单行单消息（换行分隔 JSON-RPC、UTF-8、无内嵌换行）；stdio 节
**未出现** Content-Length 或其它帧头，即 stdio 不使用 Content-Length 分帧。桥实现按行读、
按行写，stdout 仅写合法 MCP 消息，诊断只走 stderr。

## 2. tools 能力与消息形态（tools 页）

能力声明：

> Servers that support tools **MUST** declare the `tools` capability:

```json
{
  "capabilities": {
    "tools": {
      "listChanged": true
    }
  }
}
```

（`listChanged` 为可选子字段；本桥不发送列表变更通知，声明 `{"tools":{}}` 即满足
「声明 tools 能力」。）

`tools/list` 响应形态（原文示例，节选）：

> ```json
> {
>   "jsonrpc": "2.0",
>   "id": 1,
>   "result": {
>     "tools": [
>       {
>         "name": "get_weather",
>         "title": "Weather Information Provider",
>         "description": "Get current weather information for a location",
>         "inputSchema": {
>           "type": "object",
>           "properties": {
>             "location": {
>               "type": "string",
>               "description": "City name or zip code"
>             }
>           },
>           "required": ["location"]
>         }
>       }
>     ],
>     "nextCursor": "next-page-cursor"
>   }
> }
> ```

`tools/call` 请求/响应形态（原文示例）：

> ```json
> {
>   "jsonrpc": "2.0",
>   "id": 2,
>   "method": "tools/call",
>   "params": {
>     "name": "get_weather",
>     "arguments": {
>       "location": "New York"
>     }
>   }
> }
> ```

> ```json
> {
>   "jsonrpc": "2.0",
>   "id": 2,
>   "result": {
>     "content": [
>       {
>         "type": "text",
>         "text": "Current weather in New York:\nTemperature: 72°F\nConditions: Partly cloudy"
>       }
>     ],
>     "isError": false
>   }
> }
> ```

错误处理（原文）：

> Tools use two error reporting mechanisms:
>
> 1. **Protocol Errors**: Standard JSON-RPC errors for issues like:
>    * Unknown tools
>    * Invalid arguments
>    * Server errors
>
> 2. **Tool Execution Errors**: Reported in tool results with `isError: true`:
>    * API failures
>    * Invalid input data
>    * Business logic errors

未知工具示例（原文）：

> ```json
> {
>   "jsonrpc": "2.0",
>   "id": 3,
>   "error": {
>     "code": -32602,
>     "message": "Unknown tool: invalid_tool_name"
>   }
> }
> ```

## 3. 工具名语法（Schema Reference `Tool` 接口，逐字摘录）

> interface Tool {
>   [name](#tool-name): string;
>   [title](#tool-title)?: string;
>   [description](#tool-description)?: string;
>   [inputSchema](#tool-inputschema): {
>     type: "object";
>     properties?: { [key: string]: object };
>     required?: string[];
>   };
>   ...
> }

> name: string — Intended for programmatic or logical use, but used as a display name in
> past specs or fallback (if title isn't present). Inherited from BaseMetadata.name

**结论（B4 分支判定）**：2025-06-18 的 `Tool.name` 只声明为 `string`，schema 与 tools 页
均未限制字符集（未出现「仅 alphanumeric/underscore/hyphen/dot」一类条款）。因此点号合法，
本桥采用**工具名 = BRP 方法名原样**分支（如 `game.run_tests`），不做点→下划线映射，
也不需要「BRP method: <原名>」首行注。T004 实录里的 `rpc_discover` / `world_query`
是 rmcp 服务端自己的命名选择，不构成对点号禁止的反证。

## 4. initialize 与 ping（Schema Reference）

> interface InitializeRequest {
>   [method](#initializerequest-method): "initialize";
>   [params](#initializerequest-params): {
>     protocolVersion: string;
>     capabilities: ClientCapabilities;
>     clientInfo: Implementation;
>   };
> }

> interface InitializeResult {
>   [protocolVersion](#initializeresult-protocolversion): string;
>   [capabilities](#initializeresult-capabilities): ServerCapabilities;
>   [serverInfo](#initializeresult-serverinfo): Implementation;
>   [instructions](#initializeresult-instructions)?: string;
>   ...
> }

> protocolVersion: string — The version of the Model Context Protocol that the server wants
> to use. This may not match the version that the client requested. If the client cannot
> support this version, it **MUST** disconnect.

> interface PingRequest {
>   [method](#pingrequest-method): "ping";
> }

`PingRequest` 的结果类型为 `EmptyResult`（即空对象 `{}`）。

本桥按派工单口径实现：initialize **回显客户端请求的 protocolVersion**，capabilities
`{"tools":{}}`，serverInfo 附 name/version；ping 回 `result: {}`。

## 5. 通知（Schema Reference）

> `JSONRPCNotification`: A notification which does not expect a response.

> `InitializedNotification`: This notification is sent from the client to the server after
> initialization has finished.

## 6. 双源互证（规范 ↔ T004 实录）

实录路径：`tooling/brp-logs/10-mcp-session-transcript.jsonl`（29 行；rmcp 3.3.0）。

| 点 | 规范原文 | T004 实录 | 判定 |
|---|---|---|---|
| 帧格式 | newline-delimited，MUST NOT 内嵌换行 | 每行一条完整 JSON，无跨行消息 | 一致 |
| initialize 回显 | protocolVersion 可为服务端选择版本 | 请求 2025-06-18 → 响应 2025-06-18 | 一致 |
| capabilities | tools 能力须声明 | 响应 `"capabilities":{"tools":{}}` | 一致 |
| serverInfo | InitializeResult 必含 | `{"name":"rmcp","version":"3.3.0"}` | 一致（本桥换自有标识） |
| 通知无响应 | notification 不期待响应 | `notifications/initialized`（第 3 行）后直接是下一个请求，无对应响应行 | 一致 |
| tools/list | result.tools 数组 | 第 5 行 `result.tools`（107676 字节） | 一致 |
| tools/call | params.name + arguments | 第 6 行起 12 次调用均同形 | 一致 |
| 工具名 | schema 未限制字符集 | rmcp 自选 snake_case（`rpc_discover`） | 不冲突（服务端命名自由） |

**冲突检查结论：无冲突。** 降级路径未触发（规范抓取成功，两源均在档）。
