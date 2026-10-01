# Rust AI Chat

Rust AI Chat 是一個以 **實際產品實作帶動學習** 的 Rust workspace。

這個 repository 不是單純示範「如何呼叫 AI API」，而是透過兩條會持續演進的產品主線，深入淺出地學習：

- **Rust**：ownership、type system、async、trait、domain design、application architecture、persistence、testing、observability
- **Leptos**：reactive UI、CSR、SSR、hydration、routing、browser / Tauri bridge
- **AI application architecture**：model abstraction、streaming、memory、RAG、MCP、Agent Runtime、multi-agent orchestration

目前有兩個產品方向：

1. **Web Chat**：多主題、多人、多 AI Persona 的即時聊天室
2. **Desktop AI Pet**：會記憶、學習、教學並逐步具備主動性的桌面 AI 角色

> README 的「目前狀態」只描述 `main` 分支已存在的程式碼。  
> 未實作能力會明確標示為「尚未完成」或「後續方向」。

---

# Repository-first 開發原則

每一個後續章節都以實際 repository 為起點：

```text
GitHub main
   ↓
Code Review
   ↓
確認目前真正缺口
   ↓
核對 latest stable
   ↓
完成一個最小可驗證實作
   ↓
native / wasm32 build & test
   ↓
更新 README / Notion
```

固定原則：

- 不為了配合舊 roadmap 假設程式結構。
- 每次先看目前 code，再決定下一步。
- 每章只完成一個可驗證的小目標。
- domain / application / adapter / UI 保持責任分離。
- 不提前把後續功能塞進目前章節。
- README、Notion 與程式碼不一致時，以 repository 為準。

## Cargo 版本規則

dependency requirement 只保留 **major/minor**：

```toml
leptos = "0.8"
sqlx = "0.9"
tokio = "1.53"
tauri = "2.12"
```

精確 patch 版本由 `Cargo.lock` 固定。

`[package].version` 則是專案自身 SemVer，仍使用完整三段式，例如：

```toml
version = "0.1.0"
```

主線只使用 latest stable；alpha / beta / RC 不直接進入正式實作。

---

# Workspace

目前 workspace 成員：

```text
rust_ai_chat/
├─ apps/
│  ├─ server/
│  └─ desktop_pet/
│     └─ frontend/
│
├─ crates/
│  ├─ ai_core/
│  ├─ chat_application/
│  ├─ chat_domain/
│  ├─ persistence/
│  ├─ pet_domain/
│  ├─ shared/
│  ├─ telemetry/
│  └─ ui/
│
├─ migrations/
├─ deploy/
├─ tests/
└─ .github/workflows/
```

## 各 crate / app 的實際責任

| 路徑 | 目前實際內容 |
| --- | --- |
| `apps/server` | Axum server、Leptos SSR、WebSocket、PostgreSQL bootstrap |
| `apps/desktop_pet` | Tauri composition root、Pet managed state、AI command |
| `apps/desktop_pet/frontend` | Leptos CSR UI、Tauri JS/WASM bridge |
| `crates/chat_domain` | `Room`、`ChatMessage`、`MessageAuthor` |
| `crates/chat_application` | `ChatService`、`RoomRepository` port |
| `crates/shared` | WebSocket client/server protocol |
| `crates/ui` | Web Leptos Router、Chat UI、browser WebSocket adapter |
| `crates/persistence` | PostgreSQL connection、migration、`PostgresRoomRepository` |
| `crates/ai_core` | `ChatModel` abstraction、`MockChatModel` |
| `crates/pet_domain` | Pet state machine、commands、snapshot、errors |
| `crates/telemetry` | crate 邊界已建立，目前尚未實作 telemetry |
| `migrations` | 目前只有 `rooms` table migration |
| `deploy/*` | placeholder，尚未建立正式 nginx/systemd/OTel 設定 |
| `tests` | placeholder，尚未建立獨立 integration test suite |

---

# Web Chat：目前真正可運作的部分

## HTTP / SSR

Server 啟動流程：

```text
dotenv
   ↓
Leptos configuration
   ↓
DATABASE_URL
   ↓
PgPool
   ↓
SQLx migrations
   ↓
AppState
   ↓
Axum Router
   ↓
127.0.0.1:3000
```

目前 routes：

```text
GET /health
GET /
GET /room/{room_id}
ANY /ws/{room_id}
```

Leptos 使用 SSR + hydration。

## Browser UI

目前 Router：

```text
/
└─ Create Room
   ↓
/room/:room_id
```

首頁目前直接以 `Uuid::new_v4()` 建立 room route。

Room UI 已有：

- message list
- input
- Send button
- browser WebSocket adapter
- `MessageAuthor` label rendering

## WebSocket realtime flow

目前真正接通：

```text
Leptos ChatPage
    ↓
ClientEvent::SendMessage
    ↓
browser WebSocket
    ↓
/ws/{room_id}
    ↓
ChatService::send_message
    ↓
RoomHub::publish
    ↓
ServerEvent::MessageCreated
    ↓
同 room 的 WebSocket clients
```

`RoomHub` 使用：

```rust
HashMap<Uuid, broadcast::Sender<ServerEvent>>
```

並在最後一個 receiver 離開後 cleanup room channel。

## WebSocket protocol

Client：

```rust
ClientEvent::SendMessage { room_id, content }
ClientEvent::Typing { room_id, active }
ClientEvent::Ping
```

Server：

```rust
ServerEvent::MessageCreated(...)
ServerEvent::AiDelta { message_id, delta }
ServerEvent::AiCompleted { message_id }
ServerEvent::PresenceChanged { user_id, online }
ServerEvent::Error { code, message }
ServerEvent::Pong
```

目前 runtime 真正有處理：

- `SendMessage`
- `MessageCreated`
- `Ping / Pong`
- room mismatch error

目前尚未處理：

- `Typing`：server branch 存在，但目前是 no-op
- `PresenceChanged`
- `AiDelta`
- `AiCompleted`

## 目前 WebSocket 限制

browser client 目前直接使用：

```text
ws://localhost:3000/ws/{room_id}
```

因此現在仍是 local-development wiring。

正式部署前需要改成依 page origin / runtime configuration 建立 `ws://` 或 `wss://` endpoint。

---

# Chat Domain / Application

## Message model

目前 `MessageAuthor`：

```rust
AnonymousUser
User { user_id: Uuid }
Ai { persona_id: Uuid }
System
```

因此 protocol 已經能表示 AI Persona 作者，但目前還沒有完整 AI Persona runtime。

`ChatService::send_message()` 現在只建立：

```text
AnonymousUser ChatMessage
```

尚未負責 persistence、identity、AI orchestration。

## Room model

目前 `Room`：

```rust
Room {
    id: String,
    category_room_id: String,
    name: String,
}
```

這是目前 persistence/domain 嘗試降低 database-specific ID coupling 的基線。

---

# PostgreSQL / Persistence

目前 server 啟動時會：

```text
DATABASE_URL
    ↓
PgPool::connect
    ↓
persistence::migrate
```

目前 migration：

```sql
CREATE TABLE rooms (
    id varchar(40) PRIMARY KEY,
    category_room_id varchar(40) NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

## Room Repository

application layer 已定義：

```rust
trait RoomRepository {
    async fn find_by_id(
        &self,
        room_id: &str
    ) -> Result<Option<Room>, RoomRepositoryError>;
}
```

PostgreSQL adapter 已實作：

```text
PostgresRoomRepository
    ↓
SELECT id, category_room_id, name
FROM rooms
WHERE id = $1
    ↓
RoomRow
    ↓
Room
```

### 重要：目前還沒有接進 runtime

現在 server 的 `AppState` 仍持有：

- `ChatService`
- `RoomHub`
- `LeptosOptions`
- `PgPool`

`PostgresRoomRepository` 尚未被注入 application use case。

所以目前：

- WebSocket 不會透過 repository 驗證 room
- UI 建立 UUID route 並不會 INSERT room
- room repository 尚未參與正常 request flow
- chat messages 仍不會寫入 PostgreSQL
- reload 不會載入 message history

## Room ID 型別仍在過渡

目前 ID 邊界尚未統一：

| Layer | 型別 |
| --- | --- |
| Leptos room route | `Uuid` |
| WebSocket path | `Uuid` |
| `RoomHub` key | `Uuid` |
| `ChatMessage.room_id` | `Uuid` |
| `ClientEvent::SendMessage.room_id` | `Uuid` |
| `Room.id` | `String` |
| `RoomRepository::find_by_id` | `&str` |
| PostgreSQL `rooms.id` | `varchar(40)` |

這是目前 repository 最明顯的 architecture transition 之一。

後續應統一 identifier boundary，而不是讓 realtime domain 與 persistence domain 長期使用兩套 ID 表示。

---

# AI Core

目前 `crates/ai_core` 已建立最小 provider-independent abstraction：

```text
ChatRequest
ChatResponse
AiError
ChatModel
MockChatModel
```

核心 trait：

```rust
#[async_trait]
pub trait ChatModel: Send + Sync {
    async fn chat(
        &self,
        request: ChatRequest
    ) -> Result<ChatResponse, AiError>;
}
```

目前只有 `MockChatModel`。

尚未實作：

- real provider
- streaming
- Structured Output
- Function Calling
- embeddings
- RAG
- MCP
- Agent Runtime

---

# Desktop AI Pet：目前真正可運作的部分

Desktop Pet 已經不是只有 UI shell；目前 state 與 non-streaming mock chat 的垂直切片都已接通。

## Tauri managed state

Tauri application 目前註冊：

```text
Mutex<PetMachine>
AiState {
    model: Arc<dyn ChatModel>
}
```

已註冊 commands：

```text
get_pet_state
send_pet_command
chat_with_pet
```

`PetMachine` 因此已經正確透過 `.manage()` 進入 Tauri state。

## Pet state machine

目前 domain state：

```text
Idle
Interacting
Sleeping
```

commands：

```text
Interact
FinishInteraction
Sleep
Wake
```

有效 transition：

```text
Idle --Interact--> Interacting
Interacting --FinishInteraction--> Idle
Idle --Sleep--> Sleeping
Interacting --Sleep--> Sleeping
Sleeping --Wake--> Idle
```

非法 transition 回傳 `PetError::InvalidTransition`。

每次成功 transition：

```text
revision += 1
```

frontend 使用 revision 避免較舊 snapshot 覆蓋新 state。

## Tauri → Leptos state flow

```text
Leptos action
    ↓
bridge.rs
    ↓
window.sendPetCommand
    ↓
Tauri invoke
    ↓
PetMachine::dispatch
    ↓
PetSnapshot
    ↓
pet://state-changed
    ↓
frontend listener
    ↓
Leptos signal
```

## UI / animation

目前 frontend 已有：

- drag region
- Idle / Interacting / Sleeping UI projection
- blink timer
- Interacting / Sleeping animation phase
- context menu
- error rendering

動畫狀態目前留在 frontend，不塞進 `pet_domain`。

## Context menu

目前 native menu 提供：

- 互動
- 睡覺
- 醒來

menu action 最後仍走同一個 `send_pet_command` flow。

## Desktop chat flow

non-streaming chat bridge 已經實際完成：

```text
ChatBubble
    ↓
bridge::chat_with_pet
    ↓
window.chatWithPet
    ↓
invoke("chat_with_pet")
    ↓
AiState
    ↓
Arc<dyn ChatModel>
    ↓
MockChatModel
    ↓
String response
    ↓
Leptos UI
```

ChatBubble 目前已有：

- input
- Send button
- empty input disable
- loading state
- error state
- mock response rendering
- successful response 後清空 input

因此 **frontend → Tauri → ai_core → UI 的 non-streaming 垂直切片已完成**。

## Desktop window config

目前 `tauri.conf.json`：

```text
size          320 × 360
resizable     false
decorations   false
transparent   true
alwaysOnTop   true
shadow        false
bundle.active false
```

Tauri global JS API：

```text
withGlobalTauri = true
```

frontend dev server：

```text
http://localhost:1420
```

由 Tauri 自動執行：

```bash
trunk serve --port 1420
```

release frontend：

```bash
trunk build --release
```

目前尚未實作 packaging / installer / updater。

---

# Desktop AI Pet 尚未完成

目前下一階段仍需要：

- autonomous roaming
- monitor / work-area bounds
- conversation history
- real AI provider
- streaming
- persistent memory
- AI intent → pet behavior
- activity loop
- proactive trigger
- learner model
- knowledge state
- question generation
- active recall
- spaced repetition
- teaching strategy
- persistence
- RAG
- MCP
- Agent Runtime
- OpenTelemetry

---

# Web Chat 尚未完成

目前主要缺口：

- 統一 room/message/user/persona identifier strategy
- 真正的 create room use case
- RoomRepository runtime injection
- MessageRepository
- messages table
- users table
- ai_personas table
- room_ai_members table
- message persistence
- history loading
- user/session identity
- typing runtime
- presence runtime
- Web Chat → `ChatModel`
- AI streaming
- persona configuration
- persona memory
- room/persona membership
- speaker selection
- turn-taking
- reply target
- cooldown / silence policy
- multi-persona orchestrator

---

# 目前架構圖

```mermaid
flowchart TB
    subgraph WebChat["Web Chat"]
        Browser["Leptos SSR/Hydrate UI"]
        Socket["Browser ChatSocket"]
        Axum["Axum Server"]
        WS["WebSocket Handler"]
        Hub["RoomHub"]
        ChatService["ChatService"]
    end

    subgraph ChatCore["Chat Core"]
        ChatDomain["chat_domain"]
        Shared["shared protocol"]
        RoomPort["RoomRepository port"]
    end

    subgraph Infra["Infrastructure"]
        PgAdapter["PostgresRoomRepository"]
        PG["PostgreSQL"]
    end

    subgraph AI["AI Core"]
        ChatModel["ChatModel"]
        Mock["MockChatModel"]
    end

    subgraph Desktop["Desktop AI Pet"]
        PetUI["Leptos CSR"]
        Bridge["JS/WASM bridge"]
        Tauri["Tauri application"]
        PetDomain["pet_domain"]
    end

    Browser --> Socket
    Socket <--> WS
    WS --> Hub
    WS --> ChatService
    ChatService --> ChatDomain
    WS --> Shared

    RoomPort --> ChatDomain
    PgAdapter -. implements .-> RoomPort
    PgAdapter --> PG

    PetUI --> Bridge
    Bridge --> Tauri
    Tauri --> PetDomain
    Tauri --> ChatModel
    ChatModel --> Mock

    Axum --> WS
    Axum --> PG
```

目前最重要的未接通邊界之一是：

```text
PostgresRoomRepository
        -X->
Chat application runtime
```

也就是 repository adapter 已存在，但還沒有 composition 到正常 use case。

---

# CI

GitHub Actions 目前分成 `check` 與 `build`。

## Check job

目前會執行：

```bash
cargo check -p server --features ssr
cargo check -p ui --features ssr
cargo check -p ui --features hydrate --target wasm32-unknown-unknown

cargo check --workspace --exclude desktop-pet-frontend
cargo check -p desktop-pet-frontend --target wasm32-unknown-unknown

cargo test -p ai-core
cargo test -p chat-application
cargo test -p chat-domain
cargo test -p pet-domain
cargo test -p shared
cargo test -p ui
```

## Build job

目前會執行：

```bash
cargo build --workspace --exclude desktop-pet-frontend --verbose
cargo leptos build --release

cd apps/desktop_pet/frontend
trunk build --release
```

目前 CI **沒有 PostgreSQL service**，所以 `PostgresRoomRepository::find_by_id()` 的 DB integration test 不在 CI 主線裡。

---

# 開發環境

需要：

- Rust stable
- edition 2024
- `wasm32-unknown-unknown`
- PostgreSQL
- cargo-leptos
- Trunk
- Tauri CLI 2

安裝：

```bash
rustup target add wasm32-unknown-unknown

cargo install cargo-leptos --locked
cargo install trunk --locked
cargo install tauri-cli --version "^2"
```

Ubuntu / Debian 的 Tauri dependencies：

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

建立 `.env`：

```dotenv
RUST_LOG=debug
DATABASE_URL=postgres://username:password@localhost:5432/rust_ai_chat
```

server 啟動時會自動執行 SQLx migrations。

---

# 執行 Web Chat

```bash
cargo leptos watch
```

預設：

```text
http://127.0.0.1:3000
```

health：

```text
GET /health
```

release：

```bash
cargo leptos build --release
```

---

# 執行 Desktop AI Pet

```bash
cd apps/desktop_pet
cargo tauri dev
```

Tauri 會透過 `beforeDevCommand` 啟動 frontend Trunk server。

---

# 專案最終方向

## Web Chat

目標是建立：

> **Multi-topic + Multi-user + Multi-AI-Persona realtime chat system**

AI 不應只是獨立的「問 AI」入口，而是 room participant。

未來需要讓 AI Persona 能依照：

- persona
- room context
- conversation context
- memory
- speaker selection
- turn-taking
- reply target
- cooldown / policy

決定是否參與、何時發言、回應誰。

AI 身份仍應可查詢與辨識；目標是自然融入多人對話，而不是冒充某個真實的人。

## Desktop AI Pet

目標不是只有：

```text
Question → Answer
```

而是逐步演進成：

```text
Observe
   ↓
Remember
   ↓
Model learner state
   ↓
Decide whether to ask / teach
   ↓
Ask / Teach
   ↓
Receive feedback
   ↓
Learn
   ↓
Review again
```

未來會加入：

- learner model
- knowledge state
- 主動詢問
- 主動提問
- question generation
- active recall
- spaced repetition
- difficulty adaptation
- teaching strategy
- memory
- RAG
- MCP
- Agent Runtime
- observability

---

# 下一步怎麼決定

README 不把 roadmap 當成事實。

下一步固定由 code review 決定：

```text
目前 repository
    ↓
找出最小 architecture gap
    ↓
確認它是否阻擋後續能力
    ↓
只解決這一個 gap
    ↓
測試
    ↓
再進下一章
```

因此後續不會因為文件寫著「Day N」就直接進 Day N+1。

**程式碼進度決定章節，章節不反過來決定程式碼。**
