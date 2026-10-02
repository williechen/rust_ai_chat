# Rust AI Chat

透過兩個持續演進的產品，深入淺出學習 **Rust、Leptos 與 AI 應用架構**：

- **Web Chat**：多主題、多人、多 AI 角色共同參與的即時聊天室。
- **Desktop AI Pet**：透過學習與教學，逐步具備記憶、主動詢問、提問與教學能力的桌面角色。

以上是最終目標。目前 Web Chat 是匿名即時訊息基線，Desktop Pet 是狀態機、桌面移動與 mock chat 基線，尚未完成完整 AI 產品。

> 本文件依據 `main` 的程式碼快照 `07d3813c78ef71d4c0ee0b6ece61b2613686a32d` 整理。
> 「已實作」表示程式碼與 wiring 已存在；不代表本次重新完成端到端操作或編譯驗證。
> README 記錄現況與架構；Notion 記錄每日教學與演進歷程。Day 編號不作為功能完成證據。

## 目前功能總覽

| 項目 | 實際狀態 | 主要來源 |
| --- | --- | --- |
| Web SSR / hydration | Axum SSR、hydration entry、靜態 pkg service 已接入 | `apps/server/src/lib.rs`、`crates/ui/src/lib.rs` |
| 即時群聊 | 同 room WebSocket broadcast，作者皆為匿名 | `apps/server/src/ws.rs`、`hub.rs` |
| 建立房間 | 首頁生成 UUID route；未建立資料庫 room | `crates/ui/src/lib.rs` |
| ID 表示 | domain、protocol、WebSocket、RoomHub 使用字串 | `chat_domain`、`shared`、`server` |
| Room repository | trait、PostgreSQL adapter、row mapping 已存在；未注入 runtime | `chat_application`、`persistence` |
| 訊息儲存 / 歷史 | 尚未實作 | 目前 migration 只有 `rooms` |
| Typing / presence / AI delta | protocol 有宣告；typing 為 no-op，其餘未接入 | `shared`、`server/src/ws.rs` |
| AI provider | `ChatModel` trait 與 `MockChatModel`，只有 Pet 使用 | `ai_core`、`desktop_pet/src/lib.rs` |
| Pet 狀態 | managed state、合法轉移、revision、事件同步 | `pet_domain`、Pet frontend |
| Pet 聊天 | frontend → Tauri → mock model → UI，非 streaming | Pet bridge、`chat_with_pet` |
| Pet 移動 | 每 50ms 固定往右下移動；尚無邊界控制 | `roam_desktop` |
| Observability / 部署 | telemetry 空 crate；deploy 與 tests 目錄為 placeholder | `telemetry`、`deploy/*`、`tests` |

## Workspace 與責任

| 路徑 | 責任 |
| --- | --- |
| `apps/server` | Axum composition root、SSR、WebSocket、RoomHub、DB 啟動 |
| `apps/desktop_pet` | Tauri composition root、managed state、commands、移動迴圈 |
| `apps/desktop_pet/frontend` | Leptos CSR、表情動畫、控制、聊天與 JS/WASM bridge |
| `crates/chat_domain` | `Room`、`ChatMessage`、`MessageAuthor` |
| `crates/chat_application` | `ChatService`、`RoomRepository` port 與 error |
| `crates/shared` | serde WebSocket client/server event |
| `crates/ui` | Web Router、SSR shell、hydration、browser socket adapter |
| `crates/persistence` | PostgreSQL connection helper、migration、room adapter |
| `crates/ai_core` | model abstraction、request/response、mock provider |
| `crates/pet_domain` | Pet state machine、snapshot、command、error |
| `crates/telemetry` | 尚未實作 |
| `migrations` | `rooms` table |
| `deploy/nginx`、`deploy/systemd`、`deploy/otel_collector` | placeholder |
| `tests` | placeholder；現有測試位於 crate 內 |

## Web Chat 的實際流程

### 啟動與路由

`main` 載入 dotenv，`application()` 初始化 Tokio executor、讀取 Leptos configuration 與 `DATABASE_URL`，連接 PostgreSQL、執行 migration、建立 AppState，再監聽 `127.0.0.1:3000`。

| Route | 行為 |
| --- | --- |
| `GET /health` | 回傳 status/service；不執行 DB 健康檢查 |
| `GET /` | SSR 首頁 |
| `GET /room/{room_id}` | SSR room 頁面 |
| `ANY /ws/{room_id}` | WebSocket upgrade |
| Leptos pkg route | 提供編譯後的 client assets |

首頁「Create Room」只產生 UUID 並導航，沒有 INSERT、room name、category 選擇或 repository 驗證。RoomPage 直接取得字串參數，不再解析成 `Uuid`。

### 訊息與廣播

```mermaid
flowchart TD
    UI["Leptos ChatPage"] --> Socket["Browser ChatSocket"]
    Socket --> WS["Axum WebSocket"]
    WS --> Service["ChatService"]
    Service --> Message["Anonymous ChatMessage"]
    Message --> Hub["RoomHub"]
    Hub --> WS
    WS --> Socket
    Socket --> UI
```

`ChatService::send_message()` 產生 UUID 字串 message ID，建立 `AnonymousUser` 訊息，不存入資料庫。RoomHub 使用 `HashMap<String, broadcast::Sender<ServerEvent>>`，容量 128；connection 結束後，在最後一個 receiver 離開時清除相同 channel。

| Event | Server / UI 現況 |
| --- | --- |
| `SendMessage { room_id, content }` | 驗證與 connection room 相同，建立訊息並廣播 |
| `MessageCreated` | browser 追加至記憶體訊息清單 |
| `Ping / Pong` | server 回覆 application-level Pong；UI 無定時 heartbeat |
| `Typing` | server no-op |
| `PresenceChanged` | 尚未產生或呈現 |
| `AiDelta / AiCompleted` | 尚未產生或呈現 |
| `Error` | server 可回傳 room mismatch；目前 UI 未顯示此事件 |

目前作者型別為 `AnonymousUser`、`User { user_id: String }`、`Ai { persona_id: String }`、`System`。型別可表示 AI 作者，但聊天室尚未呼叫 ChatModel，也沒有 persona runtime。

### 已知限制

- WebSocket URL 硬編碼為 `ws://localhost:3000/ws/{room_id}`，尚未依同源 / HTTPS 建立 wss endpoint。
- browser 沒有連線狀態 UI、reconnect、send queue、history 或補回漏失訊息。
- broadcast lag 只記錄 skipped events，不重播。
- UI 會 trim 並阻擋空訊息；server/application 尚未實作相同內容驗證。
- 非法 event JSON 會讓 handler 回傳錯誤並結束 connection。
- 所有匿名訊息目前都顯示為「You」，未區分不同參與者。
- 尚無 login、session、authorization、nested/protected routes 或 ServerFn use case。

## Room ID 與 Persistence

目前 room、message、user、persona ID 在 domain/protocol 使用 `String`，WebSocket path 與 RoomHub key 也使用字串。UUID 仍用於生成 room/message ID，但不再是 domain 欄位型別。

| 邊界 | 表示 |
| --- | --- |
| `Room.id`、`category_room_id` | `String` |
| `ChatMessage.id / room_id` | `String` |
| `MessageAuthor.user_id / persona_id` | `String` |
| Client/server event ID | `String` |
| RoomPage、WebSocket path、RoomHub | `String` |
| `RoomRepository::find_by_id` | `&str` |
| PostgreSQL room ID / category ID | `varchar(40)` |

這完成了字串型 ID 基線；尚無共同 ID newtype、格式或長度驗證，PostgreSQL adapter 與 SQL 也仍是 PostgreSQL 專用，不能視為已可直接切換資料庫。

目前 schema：

```sql
CREATE TABLE rooms (
    id varchar(40) PRIMARY KEY,
    category_room_id varchar(40) NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

`RoomRepository: Send + Sync` 定義 async `find_by_id(&str)`。PostgresRoomRepository 執行 SELECT，以 `Option<RoomRow>.map(Room::from)` 轉換成 domain。

**Adapter 尚未接入聊天室 use case。** AppState 持有 PgPool，但 ChatService 仍是無 repository 的 unit struct；request 不檢查 room 是否存在，訊息不儲存，重新整理也不載入歷史。Server 直接呼叫 `PgPool::connect`；persistence 中 max_connections(5) 的 connection helper 目前未被 server 使用。

## AI Core

目前核心為 `ChatRequest { message }`、`ChatResponse { text }`、`AiError` 與：

```rust
#[async_trait]
pub trait ChatModel: Send + Sync {
    async fn chat(&self, request: ChatRequest) -> Result<ChatResponse, AiError>;
}
```

MockChatModel trim 輸入、拒絕空內容，回覆 `mock: {message}`。目前無真實 provider、streaming、conversation history、structured output、tool calling、embeddings、RAG、MCP 或 Agent Runtime。

## Desktop AI Pet 的實際流程

### Managed state 與 UI

Tauri 註冊 `Mutex<PetMachine>` 與 `AiState { model: Arc<dyn ChatModel> }`，model 使用 MockChatModel。Commands 為 `get_pet_state`、`send_pet_command`、`chat_with_pet`。

| 起始 state | Command | 結果 |
| --- | --- | --- |
| Idle | Interact | Interacting |
| Interacting | FinishInteraction | Idle |
| Idle / Interacting | Sleep | Sleeping |
| Sleeping | Wake | Idle |

非法轉移回傳 InvalidTransition，不改變 snapshot；成功轉移增加 revision。send_pet_command 發送 `pet://state-changed`，frontend 同時處理 command response 與 listener，拒絕較舊 revision 覆蓋新狀態。

UI 使用 emoji 表情、4 秒眨眼 timer 與 650ms 動畫 phase；動畫留在 frontend。已有拖曳區、狀態按鈕、原生右鍵選單（互動 / 睡覺 / 醒來）與錯誤顯示。

### Mock chat

```mermaid
flowchart TD
    Bubble["ChatBubble"] --> Bridge["Rust / JS bridge"]
    Bridge --> Command["Tauri chat_with_pet"]
    Command --> Model["ChatModel / MockChatModel"]
    Model --> Command
    Command --> Bridge
    Bridge --> Bubble
```

ChatBubble 已有輸入、送出、空輸入禁用、loading/error、回應顯示與成功後清空輸入。這是單次非串流回應；沒有多輪 conversation history，chat command 也不會驅動 PetMachine。

### 桌面移動基線

Tauri setup spawn `roam_desktop`，讀取 main window 的 outer_position，每次增加 x=2、y=1，再等待 50ms；main window 不存在時退出。

目前只是固定方向移動：

- 尚無 monitor/work-area bounds、碰邊反向、隨機方向或 DPI 策略。
- 不讀 PetState，Sleeping / Interacting 時也會繼續移動。
- 未協調使用者拖曳；位置讀取 / 設定錯誤目前不呈現。

因此不能把它描述成完整 autonomous roaming 或 AI 行為控制。

### 視窗與建置

| 設定 | 值 |
| --- | --- |
| main window | 320 × 360 |
| resizable / decorations / shadow | false |
| transparent / alwaysOnTop | true |
| withGlobalTauri | true |
| devUrl | http://localhost:1420 |
| frontendDist | frontend/dist |
| beforeDevCommand | frontend 目錄執行 `trunk serve --port 1420` |
| beforeBuildCommand | frontend 目錄執行 `trunk build --release` |
| bundle.active | false |

尚未完成 installer、updater 或發佈流程。

## 開發與執行

Rust toolchain 使用 `stable`，包含 rustfmt、clippy 與 wasm target；workspace 使用 edition 2024、resolver 3。需要 PostgreSQL、cargo-leptos、Trunk、Tauri CLI 2 與對應平台 native dependencies。

```bash
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos --locked
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
```

Linux CI 安裝以下套件，桌面 native build 也需要相應環境：

```bash
sudo apt-get install -y \
  build-essential pkg-config curl wget file \
  libssl-dev libxdo-dev librsvg2-dev libwebkit2gtk-4.1-dev
```

建立 PostgreSQL database：

```sql
CREATE DATABASE rust_ai_chat;
```

在 repository root 建立 `.env`：

```dotenv
RUST_LOG=debug
DATABASE_URL=postgres://username:password@localhost:5432/rust_ai_chat
```

目前未初始化 tracing subscriber；設定 RUST_LOG 不代表 observability 已完成。Server 啟動時自動 migration；DB 無法連線時無法啟動 Web Chat。

Web 開發與 release build：

```bash
cargo leptos watch
cargo leptos build --release
```

開啟 `http://127.0.0.1:3000`；以兩個 browser tab 使用同一 room URL 可檢查廣播。

Desktop Pet：

```bash
cd apps/desktop_pet
cargo tauri dev
```

由 Tauri 自動啟動 Trunk。單獨開啟 frontend browser 不具備 `window.__TAURI__`，不能取代 Tauri runtime。

## 測試與 CI 範圍

以下是 `.github/workflows/rust.yml` 的已配置指令，並非本次執行結果。

Check job：

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

Build job：

```bash
cargo build --workspace --exclude desktop-pet-frontend --verbose
cargo leptos build --release
```

另外在 `apps/desktop_pet/frontend` 執行 `trunk build --release`。兩個 job 都安裝 Linux native dependencies；build job 未宣告 needs，因此與 check job 獨立。

現有測試涵蓋 mock model、trait object、ChatService、Pet transitions、protocol roundtrip 與 event decode。CI 無 PostgreSQL service，也未執行 persistence tests；沒有端到端 browser / Tauri UI 測試。

Room repository DB test 需要 DATABASE_URL、已 migration 的 DB，以及 id=`room-rust`、name=`Rust` 的資料；測試本身不建表、不插入 fixture：

```sql
INSERT INTO rooms (id, category_room_id, name)
VALUES ('room-rust', 'category-rust', 'Rust')
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;
```

在專用測試資料庫準備完成後執行：

```bash
cargo test -p persistence test_room_repository_find_by_id
```

## 後續缺口與最終目標

### Web Chat

近期架構缺口是 room create / lookup use case 與 repository runtime injection，其次為 message persistence/history、ID validation、connection lifecycle 與 user/session identity。Typing、presence、AI streaming 目前只有部分 protocol 基線。

最終建立 **Multi-topic + Multi-user + Multi-AI-Persona**：AI 是房間參與者，依 persona、context、memory、speaker selection、turn-taking、reply target、cooldown 與 silence policy 決定是否發言。介面讓 AI 自然融入群聊，身份仍可查詢與辨識。這些 orchestration 能力目前尚未實作。

### Desktop AI Pet

近期缺口是安全的螢幕範圍移動、與睡眠/互動/拖曳協調，以及真實 provider、streaming、history 與 persistent memory。

最終建立 **Learning / Teaching Agent**：主人教 AI 與 AI 教主人形成回饋循環；透過 learner model、knowledge state、主動確認理解、出題、active recall、spaced repetition、difficulty adaptation 與 teaching strategy 推進學習。Activity loop、proactive trigger、RAG、MCP、Agent Runtime 與 OpenTelemetry 都尚未實作。

## 共同實作規則

1. 讀取實際 GitHub repository 並 code review，確認真正缺口。
2. 實作前核對官方 latest stable 與 API 差異，不直接採 alpha / beta / RC。
3. 版本升級連同實作調整；完成適當 native / WASM build、test 後更新文件。
4. Cargo.toml 的 dependency requirement 只寫 major/minor，例如 `leptos = "0.8"`、`tokio = "1.53"`；精確解析版號由 Cargo.lock 記錄。
5. major/minor requirement 使用 Cargo 預設相容版本範圍，不等於鎖死 minor；可重現建置需保留 lockfile 並使用 `--locked`。
6. `[package].version` 保留完整 SemVer，例如 `0.1.0`。
7. 每章完成可驗證的小目標；下一章由程式碼缺口決定。文件中的未來目標不得標示為已完成。
