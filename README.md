# Rust AI Chat

Rust AI Chat 是一個以 **實作帶動學習** 的 Rust workspace。

這個 repository 的第一目標，是透過可持續演進的真實專案，**深入淺出地學習 Rust、Leptos 與 AI**；聊天室與桌面 AI Pet 都是學習載體，而不是為了堆功能而堆功能。

三條核心主線是：

1. **Rust**：ownership、type system、async、domain design、application architecture、persistence、observability
2. **Leptos**：reactive system、SSR / hydration、routing、Resource / Suspense、ErrorBoundary、server functions、desktop UI integration
3. **AI**：model abstraction、streaming、Structured Output、Function Calling、memory、RAG、MCP、Agent Runtime、multi-agent orchestration

這些能力會落在兩個長期產品目標上：

1. **AI 多主題、多人、多角色即時聊天室**
2. **會透過學習與教學主動詢問、主動提問、主動引導學習的 Desktop AI Pet**

整個 repository 會從最基礎的 Rust domain model、async、WebSocket、Leptos reactive UI 開始，一步一步加入 PostgreSQL、AI model abstraction、streaming、memory、RAG、MCP、Agent Runtime 與 OpenTelemetry。

> README 與後續教學內容以目前 `main` 分支的實際程式碼為準。  
> Roadmap 是學習方向，不會把尚未落地的功能描述成已完成。

---

## 專案核心：不是堆功能，而是學會怎麼設計 Rust AI 系統

這個 repository 希望回答的不只是：

> 「怎麼用 Rust 呼叫 AI API？」

而是從實際系統中逐步理解：

- Rust ownership / borrowing / type system 如何影響 application architecture
- trait 如何把 domain、application 與 infrastructure 解耦
- async Rust、Tokio、channel、broadcast 如何組成即時系統
- Leptos signal、effect、resource、SSR、hydration 如何運作
- Axum、WebSocket 與 Leptos 如何整合
- PostgreSQL + SQLx 如何建立可維護的 persistence layer
- AI model abstraction 如何避免 application 綁死單一 provider
- Structured Output、Function Calling、RAG、MCP、Agent Runtime 如何逐層建立
- AI memory、persona、context 與 orchestration 如何設計
- OpenTelemetry 如何追蹤 `agent.run → model.turn → tool.call → rag.retrieve → database`
- Tauri + Leptos 如何把同一套 Rust domain / AI core 帶到桌面應用

學習方式不是一次把所有 framework 塞進專案，而是：

```text
先理解問題
    ↓
建立最小可運作模型
    ↓
看清楚 Rust / Leptos / AI 的原理
    ↓
加入下一層能力
    ↓
重構成可延伸架構
    ↓
回頭 code review 真實 repository
```

每一階段都應該能回答三件事：

1. **現在解決什麼問題？**
2. **Rust / Leptos / AI 在這裡各自負責什麼？**
3. **為什麼下一步需要新的 abstraction？**

---

# 最終目標一：多主題、多人、多 AI 角色聊天室

目前的 Web Chat 不是只要做到「使用者輸入一句、AI 回一句」。

最終目標是一個真正的：

> **Multi-topic + Multi-user + Multi-AI-Persona realtime chat system**

同一個平台可以存在多個主題房間，例如：

```text
Rust
Leptos
AI
Database
Game
Daily Life
Project Discussion
...```

每個房間可以同時包含：

- 多位真人使用者
- 多個 AI Persona
- 不同角色設定
- 不同知識、記憶與行為模式
- 不同的發言頻率與主動程度

概念上會變成：

```text
                         ┌── Human A
                         ├── Human B
Room / Topic ────────────┼── AI Persona A
                         ├── AI Persona B
                         └── AI Persona C
                                │
                                ▼
                         Persona Runtime
                                │
                  ┌─────────────┼─────────────┐
                  ▼             ▼             ▼
               Memory          RAG          Tools
                  │             │             │
                  └─────────────┼─────────────┘
                                ▼
                            AI Model
```

## AI Persona 的目標

AI 不應該只是聊天室旁邊的一個固定「AI 助手按鈕」。

AI Persona 應該是聊天室中的參與者：

- 有自己的名字與角色設定
- 有不同專長
- 有不同語氣與互動方式
- 可以理解目前房間主題
- 可以理解最近的多人對話
- 可以判斷現在是否適合加入
- 可以主動發言
- 可以回應特定使用者
- 可以與其他 AI Persona 互動
- 可以記住與房間、使用者、主題相關的重要資訊

希望達到的體驗是：

> AI 自然地融入群組對話，讓人類先感受到「正在和房間裡的參與者聊天」，而不是每一次互動都被 UI 與流程刻意框成「現在要向 AI 發問」。

也就是說，AI 不是獨立的問答入口，而是與真人一起存在於 room conversation 裡：它可以等待、插話、回應特定對象、延續主題，也可以和其他角色互動。

系統仍應保留可查詢的 AI 身份與透明度資訊；目標是降低「AI 助手介面感」，讓角色自然融入，而不是用 AI 冒充某個真實的人。

技術上會特別研究：

- speaker selection
- turn-taking
- silence / cooldown
- persona memory
- room context
- topic context
- reply target
- proactive message
- multi-agent coordination
- duplicate-response suppression
- streaming
- persistence
- observability

---

# 最終目標二：會成長的 Desktop AI Pet

Desktop AI Pet 也不是「把 ChatGPT 放進一個小視窗」，更不是只等待主人提出問題再回答。

最終目標是一個能透過**學習與教學循環**逐步形成記憶、知識狀態、互動策略與教學能力的桌面 AI 角色；它要能判斷什麼時候應該主動詢問、主動提問、主動複習或換一種方式教學。

初期：

```text
主人提問
   ↓
AI Pet 回答
```

但這只是起點。

真正希望做到的是：

```text
觀察互動 / 活動 / 學習紀錄
            │
            ▼
         Memory
            │
            ▼
       Learning Model
            │
    ┌───────┼────────┐
    ▼       ▼        ▼
主動詢問  主動提問  主動教學
    │       │        │
    └───────┼────────┘
            ▼
       與主人互動
            │
            ▼
      更新理解與記憶
```

## AI Pet 不只是回答問題

未來 AI Pet 應該能夠：

- 主動詢問主人目前在做什麼
- 根據之前的學習內容追問
- 發現長時間沒有理解的概念
- 用不同方式重新解釋
- 主動出題
- 根據回答調整題目難度
- 建立複習節奏
- 記住正在學習的主題
- 主動提醒尚未完成的學習內容
- 從「回答問題」逐步變成「陪伴學習」
- 從「被動 assistant」逐步變成「proactive learning agent」

例如學 Rust 時，不只是：

```text
User:
什麼是 ownership？

Pet:
ownership 是……
```

而可以演進成：

```text
Pet:
昨天我們講到 ownership 和 borrowing。

如果一個 String 被 move 進 function，
你覺得原本的變數還能不能使用？

為什麼？
```

接著依照回答：

```text
回答正確
    ↓
增加難度
    ↓
borrow / mutable borrow
    ↓
lifetime
```

或：

```text
回答不完整
    ↓
換一種解釋
    ↓
給最小 Rust example
    ↓
再問一次
```

因此 AI Pet 後續會需要研究：

- conversation memory
- long-term memory
- learner model
- knowledge state
- question generation
- teaching strategy
- active recall
- spaced repetition
- activity loop
- proactive trigger
- tool usage
- RAG
- MCP
- Agent Runtime
- desktop event integration
- AI observability

---

# Rust / Leptos / AI 三條學習主線

## Rust

從 repository 本身學習：

- workspace
- crate boundary
- domain model
- enum / struct
- ownership
- borrowing
- `Arc`
- `Mutex`
- traits
- trait object
- error handling
- async / await
- Tokio
- broadcast channel
- application service
- repository pattern
- dependency inversion
- state machine
- testing

## Leptos

逐步理解：

- CSR
- SSR
- hydration
- routing
- dynamic routes
- signals
- derived state
- effects
- event handling
- component composition
- browser WebSocket bridge
- Tauri bridge
- Resource / Suspense
- ErrorBoundary
- server functions
- islands / partial hydration

## AI

不是從「prompt engineering 技巧」開始，而是從 application architecture 建立：

```text
ChatModel abstraction
        ↓
Real Provider
        ↓
Streaming
        ↓
Structured Output
        ↓
Function Calling
        ↓
RAG
        ↓
Memory
        ↓
MCP
        ↓
Agent Runtime
        ↓
Multi Persona / Proactive Agent
        ↓
OpenTelemetry AI Observability
```

---

# 目前 Repository 狀態

## Web 聊天室

目前已實作：

- Leptos 0.8 SSR + hydration
- Leptos Router：`/`、`/room/:room_id`
- UUID room
- Axum WebSocket：`/ws/{room_id}`
- per-room `RoomHub` broadcast
- shared client/server event protocol
- PostgreSQL `PgPool`
- SQLx migration 啟動流程
- `rooms` table migration
- health check：`/health`

目前聊天室訊息仍只存在記憶體 broadcast flow：

- 尚未寫入 PostgreSQL
- reload 後不會載入 history
- 尚未建立 users / messages / personas persistence
- Web chat 尚未真正接入 AI model

## AI Core

`crates/ai_core` 已建立：

- `ChatModel` trait
- `ChatRequest`
- `ChatResponse`
- `AiError`
- `MockChatModel`

這代表 application layer 可以依賴抽象介面，而不是直接依賴某一家 AI provider。

## AI Persona 基礎

Chat domain 的作者模型已支援：

- `AnonymousUser`
- `User { user_id }`
- `Ai { persona_id }`
- `System`

因此 AI 訊息已經可以用 `persona_id` 區分不同角色。

目前尚未完整實作：

- `AiPersona` 設定模型
- room / persona membership
- persona-specific system prompt
- persona memory
- speaker selection
- turn-taking
- proactive response policy
- multi-persona orchestrator
- AI streaming WebSocket flow
- AI message persistence

## Desktop AI Pet

目前已有：

- Tauri 2 desktop shell
- Leptos CSR frontend
- transparent / always-on-top window
- Pet domain state machine
- `PetSnapshot` revision
- Tauri commands / events
- blink / simple animation
- context menu
- `ChatModel` abstraction
- `MockChatModel`
- `chat_with_pet` backend command
- ChatBubble UI

目前主要整合缺口：

- `PetMachine` managed state registration
- frontend `chat_with_pet` bridge
- chat send action
- AI response rendering
- conversation history
- real AI provider
- streaming
- AI-driven pet behavior
- memory
- proactive activity loop
- learning / teaching model
- persistence
- observability

---

# 系統架構

```mermaid
flowchart TB
    Browser["Browser / Leptos UI"]
    Router["Leptos Router"]
    WSClient["ChatSocket"]
    Server["Axum Server"]
    WS["WebSocket Handler"]
    Hub["RoomHub"]
    ChatApp["chat_application"]
    ChatDomain["chat_domain"]
    Shared["shared protocol"]
    Persist["persistence / SQLx"]
    PG["PostgreSQL"]

    Persona["AI Persona Runtime"]
    AI["ai_core / ChatModel"]
    Memory["Memory / RAG"]
    Agent["Agent Runtime / MCP"]

    Desktop["Tauri Desktop Pet"]
    PetUI["Leptos WASM UI"]
    PetDomain["pet_domain"]
    Learning["Learning / Teaching Runtime"]

    Browser --> Router
    Router --> WSClient
    WSClient <-->|ClientEvent / ServerEvent| WS
    WS --> Server
    WS --> Hub
    WS --> ChatApp
    ChatApp --> ChatDomain
    WS --> Shared

    Server --> Persist
    Persist --> PG

    ChatApp -. future .-> Persona
    Persona -. future .-> AI
    Persona -. future .-> Memory
    Persona -. future .-> Agent

    Desktop --> PetUI
    Desktop --> PetDomain
    Desktop --> AI
    Desktop -. future .-> Learning
    Learning -. future .-> Memory
    Learning -. future .-> Agent
```

虛線代表 roadmap 中尚未完整落地的能力。

---

# Workspace 結構

| 路徑 | 職責 |
| --- | --- |
| `apps/server` | Axum server、SSR、WebSocket、DB bootstrap |
| `apps/desktop_pet` | Tauri desktop backend、Pet state、AI command |
| `apps/desktop_pet/frontend` | Leptos WASM desktop UI |
| `crates/ui` | Web Leptos UI、Router、WebSocket client |
| `crates/chat_domain` | Chat message、message author domain |
| `crates/chat_application` | Chat application service |
| `crates/shared` | Client / Server WebSocket event contract |
| `crates/ai_core` | AI abstraction、`ChatModel`、mock model |
| `crates/persistence` | PostgreSQL / SQLx |
| `crates/pet_domain` | Desktop Pet state machine |
| `crates/telemetry` | Observability 預留 crate |
| `migrations` | SQLx migrations |
| `deploy` | nginx / systemd / OTel collector 預留設定 |

---

# 已完成的 Day 1～Day 12 主線

| Day | 主題 | Repository 現況 |
| --- | --- | --- |
| Day 1 | Rust workspace | ✅ 多 crate workspace |
| Day 2 | Chat domain / shared protocol | ✅ |
| Day 3 | Axum HTTP server | ✅ |
| Day 4 | WebSocket 基礎 | ✅ |
| Day 5 | ChatService | ✅ |
| Day 6 | Tokio broadcast | ✅ |
| Day 7 | Per-room RoomHub | ✅ |
| Day 8 | Leptos reactive UI | ✅ |
| Day 9 | Browser WebSocket adapter | ✅ |
| Day 10 | SSR + hydration | ✅ |
| Day 11 | Router | ✅ |
| Day 12 | PostgreSQL + SQLx bootstrap | ✅ |

Day 12 已經讓 PostgreSQL 進入 runtime：

```text
DATABASE_URL
    ↓
PgPool::connect
    ↓
SQLx migrations
    ↓
Axum AppState
```

但目前還不是完整 persistence architecture。

尚未完成：

- messages table
- users table
- AI personas table
- room AI membership table
- repositories
- message history loading

---

# 接下來的學習與實作方向

後續章節不應只照舊 roadmap 往下寫。

流程固定為：

```text
實際 GitHub repository
        ↓
Code Review
        ↓
確認目前真正缺口
        ↓
決定下一個最小學習目標
        ↓
深入淺出解釋原理
        ↓
實作
        ↓
測試
        ↓
再進下一章
```

## Chat 主線

接下來會逐步建立：

1. Room / Message repository
2. message persistence
3. history loading
4. user identity / session
5. typing / presence
6. `AiPersona`
7. room persona membership
8. Web chat → `ChatModel`
9. streaming
10. persona memory
11. speaker selection / turn-taking
12. multi-persona orchestrator
13. multi-topic room context
14. RAG
15. MCP / tools
16. Agent Runtime
17. OpenTelemetry

## Desktop AI Pet 主線

接下來會逐步建立：

1. 完成 frontend ↔ Tauri ↔ `ChatModel` chat bridge
2. real model provider
3. streaming
4. conversation persistence
5. pet memory
6. AI intent → pet behavior
7. activity loop
8. proactive trigger
9. 主動詢問
10. 主動提問
11. learner knowledge state
12. teaching strategy
13. active recall / spaced repetition
14. RAG
15. MCP / tools
16. Agent Runtime
17. OpenTelemetry

---

# WebSocket Protocol

Client events：

```rust
ClientEvent::SendMessage { room_id, content }
ClientEvent::Typing { room_id, active }
ClientEvent::Ping
```

Server events：

```rust
ServerEvent::MessageCreated(...)
ServerEvent::AiDelta { message_id, delta }
ServerEvent::AiCompleted { message_id }
ServerEvent::PresenceChanged { user_id, online }
ServerEvent::Error { code, message }
ServerEvent::Pong
```

目前真正完成的主要 flow：

- SendMessage
- MessageCreated
- Ping / Pong
- room mismatch error

Typing、Presence、AI streaming 仍待後續實作。

---

# 版本策略

這個 repository 的教學與實作一律以 **目前最新穩定版（latest stable）** 為基準，而不是沿用舊章節的版本號。

固定規則：

- 每次新增章節或進行 code review 前，先核對 Rust、Leptos、Axum、Tauri、SQLx、OpenTelemetry、PostgreSQL 與主要工具的最新穩定版。
- 若目前 `Cargo.lock` 已解析到較新的穩定版本，教學內容與 `Cargo.toml` 必須同步，不得繼續示範舊 API。
- 升級不能只改版本號；遇到 API / feature / runtime 行為改變時，要同步修改實作、測試與章節說明。
- alpha / beta / RC 不進入主線實作，除非該章明確是「下一版 migration / preview」實驗。
- PostgreSQL、nginx 等基礎設施優先採最新 stable / production channel，而不是 development / beta / mainline channel。
- 每次版本調整後，以實際 repository 的 build / test 結果決定是否完成升級。

# 開發需求

- Rust stable（目前 1.98.1）
- Rust edition 2024
- `wasm32-unknown-unknown`
- PostgreSQL 18.6（PostgreSQL 19 目前仍為 beta，不進主線）
- `cargo-leptos` 0.3.10
- `trunk`
- Tauri CLI 2

安裝主要工具：

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
cargo install trunk --locked
cargo install tauri-cli --version "^2"
```

Ubuntu / Debian Tauri dependencies：

```bash
sudo apt-get install -y \
  build-essential \
  pkg-config \
  curl \
  wget \
  file \
  libssl-dev \
  libxdo-dev \
  librsvg2-dev \
  libwebkit2gtk-4.1-dev
```

---

# PostgreSQL

建立 database：

```sql
CREATE DATABASE rust_ai_chat;
```

repository root 建立 `.env`：

```dotenv
RUST_LOG=debug
DATABASE_URL=postgres://username:password@localhost:5432/rust_ai_chat
```

server 啟動時會執行 SQLx migrations。

---

# 執行 Web Chat

```bash
cargo leptos watch
```

預設：

```text
http://127.0.0.1:3000
```

首頁建立 room 後會導向：

```text
/room/{UUID}
```

同一 room URL 的多個 browser connection 會共享該 room 的 realtime broadcast。

Health check：

```text
GET http://127.0.0.1:3000/health
```

Release build：

```bash
cargo leptos build --release
```

---

# 執行 Desktop AI Pet

```bash
cd apps/desktop_pet
cargo tauri dev
```

frontend 使用 Trunk，dev server 預設為：

```text
http://localhost:1420
```

---

# 測試與檢查

```bash
cargo check -p server --features ssr
cargo check -p ui --features ssr
cargo check -p ui --features hydrate --target wasm32-unknown-unknown
cargo check --workspace --exclude desktop-pet-frontend

cargo test -p pet-domain
cargo test -p ai-core
cargo test -p shared

cargo check -p desktop-pet-frontend --target wasm32-unknown-unknown
```

---

# 專案真正想完成的事情

這個 repository 的核心不是「完成兩個 AI App」，而是：

> **透過真正會持續長大的產品，深入淺出地學會 Rust、Leptos 與 AI，並理解它們如何共同組成可維護、可演進、可觀測的 AI 系統。**

### Chat

> 用 Rust + Leptos 建立一個真正的多主題、多使用者、多 AI Persona 即時聊天室。AI 不以獨立助手入口存在，而是像其他 room participant 一樣依照角色、上下文、turn-taking、memory 與 policy 自然參與群組互動；介面不需要不斷提醒使用者「現在正在問 AI」，但 AI 身份仍可被查詢與辨識。

### AI Pet

> 用 Tauri + Leptos + Rust AI Core 建立一個會記憶、會學習，也會教學的桌面 AI 角色。它不只回答主人的問題，而會根據學習紀錄與互動狀態主動詢問、主動提問、主動出題、安排複習、調整難度與改變解釋方式，逐步從被動 assistant 演進成 proactive learning / teaching agent。
