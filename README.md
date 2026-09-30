# Rust AI Chat

Rust AI Chat 是一個以 Rust workspace 組成的學習／實作專案，核心包含：

- **Axum + Leptos 0.8** 的即時聊天室
- **WebSocket per-room broadcast** 即時訊息
- **PostgreSQL + SQLx** 基礎持久化層
- **Tauri 2 + Leptos** 桌面 AI 小寵物
- 可擴充到 **AI Persona、多角色聊天室、streaming、presence、typing、observability** 的共用事件模型

目前專案已完成 Day 1～Day 12 的核心路線；AI 多角色聊天室的資料模型與 WebSocket event contract 已開始預留，但真正的多角色 orchestrator、AI streaming 與訊息持久化仍屬下一階段。

> README 內容以目前 `main` 分支實際程式碼為準，不把 roadmap 中尚未落地的功能列為已完成。

## 目前狀態

### Web 聊天室

已實作：

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

目前聊天室訊息仍只存在記憶體 broadcast 流程中，**尚未寫入 PostgreSQL**，重新整理頁面也不會載入歷史訊息。

### AI 能力

`crates/ai_core` 已建立：

- `ChatModel` trait
- `ChatRequest`
- `ChatResponse`
- `AiError`
- `MockChatModel`

目前 Web 聊天室尚未把 `ChatModel` 接到 WebSocket message flow，因此使用者訊息不會自動產生 AI 回覆。

共用 protocol 已預留：

- `ServerEvent::AiDelta`
- `ServerEvent::AiCompleted`
- `ServerEvent::PresenceChanged`
- `ClientEvent::Typing`

這些 event 目前只是協定層準備，尚未完整實作 server-side 行為。

### AI 多角色聊天室

Domain 已把訊息作者從單純的 user/assistant role 擴充成：

- `AnonymousUser`
- `User { user_id }`
- `Ai { persona_id }`
- `System`

因此一則 AI 訊息可以帶有不同的 `persona_id`，這是未來同一房間存在多個 AI 角色的基礎。

預計的資料流會是：

```text
User Message
    │
    ▼
Room
    │
    ├── Persona A ──► AI Model
    ├── Persona B ──► AI Model
    └── Persona C ──► AI Model
             │
             ▼
      MessageAuthor::Ai
        { persona_id }
             │
             ▼
        Room WebSocket
```

目前尚未實作：

- `AiPersona` 設定模型
- room 與 AI persona membership
- 多 persona 排程／orchestrator
- persona-specific system prompt
- AI streaming 真正串入 WebSocket
- AI 訊息資料庫持久化

因此目前 README 將它列為**架構方向與已具備的基礎能力**，而不是已完成的聊天室功能。

### Desktop AI Pet

桌面 AI 寵物不是單純的 UI demo，而是目前 repository 中另一條獨立的 application path：

```text
Leptos WASM UI
    │
    ▼
JavaScript bridge
    │
    ▼
Tauri invoke / event
    │
    ├── PetMachine
    │     └── pet_domain
    │
    └── AiState
          └── ChatModel
                └── MockChatModel
```

目前已經具備 **桌面視窗、寵物狀態機、Tauri command/event、Leptos reactive UI，以及 AI model abstraction**；但 Pet state 的 Tauri state registration 與聊天 frontend bridge 還沒有完全接好，所以目前仍屬「架構已成形、整合尚未完成」的狀態。

#### 1. Tauri Desktop Shell

`apps/desktop_pet/tauri.conf.json` 目前定義：

- product name：`Rust AI Desktop Pet`
- identifier：`dev.rustai.desktop-pet`
- 視窗大小：`320 × 360`
- `resizable: false`
- `decorations: false`
- `transparent: true`
- `alwaysOnTop: true`
- `shadow: false`
- dev frontend：`http://localhost:1420`
- frontend build：Trunk

因此這個 app 的設計方向是「漂浮在桌面上的小寵物」，而不是一般有標題列的桌面視窗。

Capability 目前只開放：

```text
core:default
core:window:allow-start-dragging
```

Leptos UI 透過 `data-tauri-drag-region` 實作視窗拖曳。

#### 2. Pet Domain State Machine

`crates/pet_domain` 把寵物行為和 UI 分離。

目前 domain state：

```rust
PetState::Idle
PetState::Interacting
PetState::Sleeping
```

可接受的 command：

```rust
PetCommand::Interact
PetCommand::FinishInteraction
PetCommand::Sleep
PetCommand::Wake
```

目前允許的 transition：

```text
Idle
 ├── Interact ─────────────► Interacting
 └── Sleep ────────────────► Sleeping

Interacting
 ├── FinishInteraction ────► Idle
 └── Sleep ────────────────► Sleeping

Sleeping
 └── Wake ─────────────────► Idle
```

其他 transition 會回傳：

```rust
PetError::InvalidTransition
```

例如 sleeping 狀態直接執行 `Interact` 目前會被拒絕。

#### 3. PetSnapshot 與 revision

UI 不直接持有 `PetMachine`，而是接收：

```rust
PetSnapshot {
    state,
    revision,
}
```

每次合法 state transition：

```text
dispatch command
    │
    ▼
change state
    │
    ▼
revision += 1
    │
    ▼
return PetSnapshot
```

frontend 的 `apply_snapshot` 只接受 revision 不小於目前 snapshot 的資料：

```text
incoming.revision >= current.revision
```

這是目前用來避免較舊狀態覆蓋較新狀態的基礎機制。

#### 4. Tauri Pet Commands

Desktop backend 已定義兩個寵物 command：

```rust
get_pet_state()
send_pet_command(command)
```

預期資料流：

```text
Leptos UI
    │
    ▼
window.sendPetCommand(...)
    │
    ▼
Tauri invoke("send_pet_command")
    │
    ▼
PetMachine::dispatch(...)
    │
    ▼
PetSnapshot
    │
    ├── command return value
    │
    └── emit("pet://state-changed")
```

frontend 同時會：

- 啟動時呼叫 `get_pet_state`
- 訂閱 `pet://state-changed`
- 收到 snapshot 後經過 revision 檢查再更新 UI

目前有一個重要的 runtime wiring 尚未完成：

`get_pet_state` 與 `send_pet_command` 都要求：

```rust
State<Mutex<PetMachine>>
```

但目前 `tauri::Builder` 只註冊：

```rust
.manage(AiState::new(Arc::new(MockChatModel)))
```

尚未看到：

```rust
.manage(Mutex::new(PetMachine::default()))
```

因此 PetMachine 的 managed state 還需要補上，才能讓這兩個 command 的 state injection 完整成立。

#### 5. Leptos Pet UI

Desktop frontend 使用 Leptos CSR。

目前主要 component：

```text
App
├── DragHandle
├── BlinkController
├── AnimationClock
├── PetStatus
├── PetControls
├── ChatBubble
└── ErrorMessage
```

UI 使用 signals 管理：

- pet snapshot
- error
- blink state
- animation phase
- chat bubble open/close
- chat input

#### 6. Pet Pose Projection

Domain 只知道：

```text
Idle
Interacting
Sleeping
```

動畫 pose 則留在 UI：

```rust
PetPose::Idle
PetPose::Blink
PetPose::InteractA
PetPose::InteractB
PetPose::SleepA
PetPose::SleepB
```

也就是：

```text
PetState
    +
Blink flag
    +
Animation phase
    │
    ▼
PetPose
    │
    ▼
visual representation
```

這樣可以避免把 animation frame 塞進 domain model。

目前畫面暫時使用 emoji：

- 😺 Idle
- 😻 Blink
- 😸 / 😹 Interacting
- 😴 / 😪 Sleeping

未來可以直接把 `PetPose` mapping 換成 sprite、WebP、SVG、Lottie 或其他 animation asset，而不需要改 `pet_domain`。

#### 7. Blink Controller

`BlinkController` 每 4 秒檢查一次：

```text
PetState == Idle ?
    │
    ├── no  → blinking = false
    │
    └── yes → blinking = true
               │
               └── 180 ms 後回 false
```

因此只有 idle pet 會自動眨眼。

#### 8. Animation Clock

`AnimationClock` 每 650 ms 切換一次：

```text
animation_phase = !animation_phase
```

Interacting 和 Sleeping 根據這個 phase 在 A / B pose 之間切換。

目前這是最簡單的 frame animation clock；之後可以替換成更完整的 animation timeline。

#### 9. Pet Controls

目前 UI 提供：

- 互動
- 結束
- 睡覺
- 醒來

每個操作都走同一條 command flow：

```text
button click
    │
    ▼
dispatch_command(...)
    │
    ▼
bridge::send_pet_command(...)
    │
    ▼
Tauri
    │
    ▼
PetMachine
```

錯誤會寫入 `error` signal，再由 `ErrorMessage` 顯示。

#### 10. Context Menu

對寵物按右鍵會呼叫：

```text
showPetContextMenu(x, y)
```

JavaScript 使用 Tauri menu API 建立 context menu，目前包含：

- 互動
- 睡覺
- 醒來

menu action 仍然呼叫同一個 `sendPetCommand`，因此 UI button 與 context menu 共用同一個 domain command path。

#### 11. AI Core

AI pet backend 已經接入 `crates/ai_core`。

目前 abstraction：

```rust
#[async_trait]
pub trait ChatModel: Send + Sync {
    async fn chat(
        &self,
        request: ChatRequest,
    ) -> Result<ChatResponse, AiError>;
}
```

Tauri application 使用：

```rust
AiState {
    model: Arc<dyn ChatModel>
}
```

目前實際注入：

```text
Arc<dyn ChatModel>
        │
        ▼
MockChatModel
```

所以 application layer 已經不是直接依賴特定 AI provider，而是依賴 `ChatModel` interface。

未來可把 `MockChatModel` 替換成真正 provider，而不必改 Tauri command 的呼叫方式。

#### 12. chat_with_pet

Desktop backend 已定義：

```rust
#[tauri::command]
async fn chat_with_pet(
    message: String,
    state: State<'_, AiState>,
) -> Result<String, String>
```

目前 backend flow：

```text
message
   │
   ▼
ChatRequest::new(message)
   │
   ▼
AiState.model
   │
   ▼
ChatModel::chat
   │
   ▼
MockChatModel
   │
   ▼
"mock: {message}"
```

空白訊息會由 `AiError::EmptyMessage` 拒絕。

這表示 **AI pet backend 的 chat abstraction 已經存在**。

#### 13. ChatBubble 現況

Leptos frontend 已經有：

- 「聊天」按鈕
- ChatBubble
- input
- close button
- `chat_input` signal

但目前：

```rust
prop:disabled=true
```

所以送出按鈕還不能使用。

而且 `frontend/src/bridge.rs` 目前只有：

- `get_pet_state`
- `send_pet_command`
- `listen_pet_state`
- `show_pet_context_menu`

還沒有：

```rust
chat_with_pet(...)
```

`index.html` 也尚未提供：

```javascript
window.chatWithPet = (...) =>
    invoke("chat_with_pet", ...)
```

因此目前狀態是：

```text
ChatBubble UI                    ✅
chat input state                 ✅
Tauri chat_with_pet command      ✅
ChatModel abstraction            ✅
MockChatModel                    ✅

JS invoke bridge                 ❌
Rust WASM bridge                 ❌
Send button                      ❌ disabled
AI response state                ❌
Conversation history             ❌
Streaming                        ❌
Real model provider              ❌
```

#### 14. AI Pet 完整架構方向

目前 architecture 可以自然演進成：

```mermaid
flowchart LR
    UI["Leptos Pet UI"]
    Bubble["ChatBubble"]
    Bridge["WASM / JS Bridge"]
    Tauri["Tauri Commands"]
    Pet["PetMachine"]
    AI["AiState"]
    Model["ChatModel"]
    Provider["AI Provider"]

    UI --> Pet
    UI --> Bubble
    Bubble --> Bridge
    Bridge --> Tauri

    Tauri --> Pet
    Tauri --> AI
    AI --> Model
    Model --> Provider
```

未來 AI response 還可以反過來影響 pet behavior：

```text
AI response
    │
    ├── emotion / intent
    │
    ▼
PetCommand
    │
    ▼
PetMachine
    │
    ▼
PetState
    │
    ▼
PetPose / animation
```

這樣 AI pet 才會從「有聊天框的桌面寵物」進一步變成真正由 AI 對話驅動行為的 desktop agent。

#### 15. Desktop AI Pet 已完成 / 未完成

| 能力 | 狀態 |
| --- | --- |
| Tauri 2 desktop shell | ✅ |
| Transparent / always-on-top window | ✅ |
| Window drag region | ✅ |
| Leptos CSR frontend | ✅ |
| Pet domain state machine | ✅ |
| Pet commands | ✅ |
| Pet snapshot revision | ✅ |
| Blink behavior | ✅ |
| Simple frame animation | ✅ |
| Context menu | ✅ |
| Tauri pet commands | ✅ |
| Tauri state-change event | ✅ |
| `ChatModel` abstraction | ✅ |
| `MockChatModel` | ✅ |
| `chat_with_pet` backend command | ✅ |
| Managed `PetMachine` registration | ⚠️ 尚需補上 |
| ChatBubble UI | ✅ |
| Frontend chat invoke bridge | ❌ |
| Chat send action | ❌ |
| AI response rendering | ❌ |
| Conversation history | ❌ |
| Streaming response | ❌ |
| Real AI provider | ❌ |
| AI-driven pet emotion/state | ❌ |
| Persistence | ❌ |
| Desktop AI observability | ❌ |

#### 16. AI Pet 下一步

以目前 repository 實作來看，AI pet 最合理的下一段不是重寫架構，而是把已經存在的 pieces 串起來：

1. 在 Tauri Builder 註冊 `Mutex<PetMachine>`
2. 在 `index.html` 加入 `window.chatWithPet`
3. 在 `frontend/src/bridge.rs` 加入 `chat_with_pet`
4. 啟用 ChatBubble send button
5. 增加 loading / error / response signals
6. 顯示 MockChatModel response
7. 再替換成 real `ChatModel` provider
8. 增加 streaming abstraction
9. 讓 AI intent / emotion 驅動 `PetCommand`
10. 最後再加入 persistence 與 telemetry


## 系統架構

```mermaid
flowchart TB
    Browser["Browser / Leptos UI"]
    Router["Leptos Router"]
    WSClient["ChatSocket"]
    Axum["Axum Server"]
    WS["WebSocket Handler"]
    Hub["RoomHub"]
    ChatService["ChatService"]
    Domain["chat_domain"]
    Protocol["shared events"]
    PG["PostgreSQL"]
    Persist["persistence / SQLx"]

    Desktop["Tauri Desktop Pet"]
    PetUI["Leptos WASM Frontend"]
    Bridge["JS / Tauri Bridge"]
    PetDomain["pet_domain"]
    AiCore["ai_core"]
    Mock["MockChatModel"]

    Browser --> Router
    Router --> WSClient
    WSClient <-->|ClientEvent / ServerEvent| WS
    WS --> Axum
    WS --> Hub
    WS --> ChatService
    ChatService --> Domain
    WS --> Protocol

    Axum --> Persist
    Persist --> PG

    Desktop --> PetUI
    PetUI --> Bridge
    Bridge --> Desktop
    Desktop --> PetDomain
    Desktop --> AiCore
    AiCore --> Mock
```

### Workspace 結構

| 路徑 | 職責 |
| --- | --- |
| `apps/server` | Axum server、SSR、WebSocket、DB bootstrap |
| `apps/desktop_pet` | Tauri desktop backend、Pet state、AI command |
| `apps/desktop_pet/frontend` | Leptos WASM desktop UI |
| `crates/ui` | Web Leptos UI、Router、WebSocket client |
| `crates/chat_domain` | Chat message 與 `MessageAuthor` domain |
| `crates/chat_application` | `ChatService` application layer |
| `crates/shared` | Client/Server WebSocket event contract |
| `crates/ai_core` | AI abstraction、`ChatModel`、mock model |
| `crates/persistence` | PostgreSQL connection / SQLx migrations |
| `crates/pet_domain` | Desktop pet state machine |
| `crates/telemetry` | Observability 預留 crate，目前尚未實作 |
| `migrations` | SQLx database migrations |
| `deploy` | nginx / systemd / OTel collector 預留設定 |

## Day 1～Day 12

目前 Day 1～Day 12 對應的主線如下。

| Day | 主題 | Repository 現況 |
| --- | --- | --- |
| Day 1 | Rust workspace | ✅ 多 crate workspace |
| Day 2 | Chat domain / shared protocol | ✅ `ChatMessage`、`MessageAuthor`、Client/Server events |
| Day 3 | Axum HTTP server | ✅ Axum server、`/health` |
| Day 4 | WebSocket 基礎 | ✅ `/ws/{room_id}` |
| Day 5 | ChatService | ✅ application service 建立訊息 |
| Day 6 | Broadcast | ✅ Tokio broadcast |
| Day 7 | Per-room RoomHub | ✅ UUID room 隔離 |
| Day 8 | Leptos reactive UI | ✅ room UI、signals、message list |
| Day 9 | Browser WebSocket adapter | ✅ `ChatSocket` |
| Day 10 | SSR + hydration | ✅ Leptos SSR / WASM hydration |
| Day 11 | Router | ✅ home + room route |
| Day 12 | PostgreSQL + SQLx | ✅ PgPool、migration runner、`rooms` table |

### Day 12 目前做到哪裡？

Server 啟動時會：

```text
DATABASE_URL
    │
    ▼
PgPool::connect
    │
    ▼
persistence::migrate
    │
    ▼
Axum AppState
```

目前 migration 已建立 `rooms` table，但尚未加入：

- messages table
- users table
- AI personas table
- room AI membership table
- message repository
- room repository application flow

所以 PostgreSQL 已進入 runtime bootstrap，但還不是完整 persistence architecture。

## WebSocket Protocol

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

目前真正處理完成的是：

- SendMessage
- MessageCreated
- Ping / Pong
- room mismatch error

Typing、Presence 與 AI streaming 仍待後續實作。

## 開發需求

- Rust stable
- Rust edition 2024
- `wasm32-unknown-unknown`
- PostgreSQL
- `cargo-leptos`
- `trunk`
- Tauri CLI 2
- Linux 桌面開發需 WebKitGTK 等系統套件

Rust toolchain 已由 `rust-toolchain.toml` 設定 stable、rustfmt、clippy 與 WASM target。

安裝主要工具：

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
cargo install trunk --locked
cargo install tauri-cli --version "^2"
```

Ubuntu / Debian 的 Tauri build dependencies 可參考 CI：

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

## PostgreSQL 設定

建立 database，例如：

```sql
CREATE DATABASE rust_ai_chat;
```

Server 實際讀取的環境變數名稱是 **`DATABASE_URL`**。

在 repository root 建立 `.env`：

```dotenv
RUST_LOG=debug
DATABASE_URL=postgres://username:password@localhost:5432/rust_ai_chat
```

啟動 server 時會自動呼叫 SQLx migration runner。

## 執行 Web 聊天室

在 repository root：

```bash
cargo leptos watch
```

開啟：

```text
http://127.0.0.1:3000
```

首頁點擊 **Create Room** 後會產生 UUID，並導向：

```text
/room/{UUID}
```

同一 room URL 的多個瀏覽器連線會共享該 room 的即時 broadcast。

Health check：

```text
GET http://127.0.0.1:3000/health
```

Release build：

```bash
cargo leptos build --release
```

## 執行 Desktop AI Pet

```bash
cd apps/desktop_pet
cargo tauri dev
```

Tauri 的 `beforeDevCommand` 會在 `apps/desktop_pet/frontend` 啟動：

```bash
trunk serve --port 1420
```

桌面視窗預設：

- 320 × 360
- transparent
- decorations disabled
- always on top
- non-resizable

## 測試與檢查

常用指令：

```bash
cargo check -p server --features ssr
cargo check -p ui --features ssr
cargo check -p ui --features hydrate --target wasm32-unknown-unknown
cargo check --workspace --exclude desktop-pet-frontend

cargo test -p pet-domain
cargo test -p ai-core
cargo test -p shared
```

Desktop frontend：

```bash
cargo check -p desktop-pet-frontend --target wasm32-unknown-unknown
```

GitHub Actions 目前會檢查：

- server SSR
- UI SSR
- UI WASM hydration
- native workspace
- desktop WASM frontend
- pet domain tests
- release builds

## 已知限制

目前專案仍處於逐步建立架構的階段：

- Web chat 尚未接 AI model
- Web chat message 尚未 persistence
- history 尚未實作
- authentication / session 尚未實作
- typing event 尚未實作 server behavior
- presence 尚未實作
- AI streaming 尚未實作
- multi-persona orchestrator 尚未實作
- telemetry crate 尚未實作
- desktop pet frontend 尚未接 `chat_with_pet`
- desktop pet 尚未接 real AI provider

## 下一階段方向

接下來合理的演進順序是：

1. 建立 Room / Message repository
2. 完成 message persistence 與 history loading
3. 建立 `AiPersona` 與 room membership
4. 把 `ChatModel` 注入 Web chat application flow
5. 完成 `AiDelta` / `AiCompleted` streaming
6. 建立多 AI Persona orchestrator
7. 完成 typing / presence
8. 串接 desktop pet frontend → `chat_with_pet`
9. 接入 real provider
10. 加入 OpenTelemetry tracing

---

此專案的重點不只是完成聊天室，而是逐步建立一套可延伸到 **Web 即時聊天室、AI Persona、多 Agent 對話、桌面 AI 寵物與 observability** 的 Rust application architecture。
