# Rust AI Chat

Rust AI Chat 是一個持續演進的 Rust workspace，包含兩個產品方向：

- **Web Chat**：以 Rust、Axum、Leptos、WebSocket 與 PostgreSQL 建構多人即時聊天室，最終加入多 AI Persona。
- **Desktop AI Pet**：以 Rust、Tauri、Leptos 與 AI model abstraction 建構可互動、可學習、可教學的桌面 AI 角色。

> **文件分工**
>
> - **GitHub Repository / README**：只記錄實際程式大綱、最終目標、後續缺口、共同實作規則、測試與 CI 範圍、開發與執行方式。
> - **Notion**：記錄每日章節、設計理由與**深入淺出的實作教學**；必須包含實作步驟、程式碼修改、驗證方式與實作細節，不只是概念或原理解說。
>
> README 以實際 repository 為準，不以 Day 編號或教學規劃判定功能是否完成。

專案方向與共同規則對照：[Notion 聊天室主頁](https://app.notion.com/p/3e3e51d4509381df860bf52cc2a4a827)、[Notion 桌面小動物主頁](https://app.notion.com/p/3e3e51d4509381639005e7c9c63f6917)。主頁與章節中的規劃不代表程式已完成。

## 大綱

目前 workspace 主要結構：

| 路徑 | 實際責任 |
| --- | --- |
| `apps/server` | Axum Web server、Leptos SSR、WebSocket、room hub、PostgreSQL 啟動 |
| `apps/desktop_pet` | Tauri desktop application、managed state、commands、桌面移動 |
| `apps/desktop_pet/frontend` | Desktop Pet 的 Leptos WASM frontend |
| `crates/chat_domain` | Room、ChatMessage、MessageAuthor 等 chat domain model |
| `crates/chat_application` | Chat use case、RoomRepository port |
| `crates/shared` | WebSocket client/server protocol |
| `crates/ui` | Web Chat 的 Leptos Router、SSR/hydration 與 browser socket adapter |
| `crates/persistence` | PostgreSQL connection、migration、Room repository adapter |
| `crates/ai_core` | AI model abstraction、request/response、MockChatModel |
| `crates/pet_domain` | Desktop Pet state machine |
| `crates/telemetry` | Observability 預留 crate，目前尚未完成 |
| `migrations` | Database migrations |
| `deploy` | 部署相關設定，目前仍以 placeholder 為主 |
| `tests` | Repository-level test 預留目錄；目前多數測試仍位於各 crate 內 |

本輪核對基線：`main` commit [`2735b135`](https://github.com/williechen/rust_ai_chat/commit/2735b135b581b5b1d74a256a8adcf11b7a15aece)（2026-10-05，Asia/Taipei）。以下「已完成」指程式已接線的基線能力；完整使用流程仍受後續缺口限制。

目前實作基線：

- Web Chat 已具備 Axum + Leptos SSR/hydration、首頁與 `/room/:room_id` 路由、`/health`、同房間 WebSocket broadcast、應用層 Ping/Pong，以及訊息 room ID 與連線 room ID 的一致性檢查。
- 首頁 `Create Room` 目前只產生 UUID 並跳轉，未呼叫建立房間 use case，也未寫入資料庫；房間建立流程尚未完成。
- Chat domain / protocol 的主要識別碼使用跨資料庫較通用的字串表示。
- RoomRepository trait 與 PostgreSQL adapter 已存在，並已注入 server runtime；WebSocket upgrade 前會透過 repository 驗證 room 是否存在。
- Web Chat 尚未完成 message persistence、history、login/session、authorization、reconnect、presence、typing、AI streaming 與 persona runtime。
- `Typing`、`PresenceChanged`、`AiDelta`、`AiCompleted` 只是預留 protocol；server 的 typing 分支尚未處理，Web UI 目前只消費 `MessageCreated`。
- AI Core 已有 `ChatModel` abstraction 與 `MockChatModel`，目前真實 AI provider 尚未接入 Web Chat。
- Desktop AI Pet 已有 Tauri managed state、Pet state machine、frontend bridge、mock chat、native context-menu command dispatch，以及具 work-area bounds、碰邊反向、位置變更驗證與 stall 停止機制的桌面移動。
- Desktop AI Pet 尚未完成依 PetState / 使用者拖曳協調移動、真實 AI provider、streaming、memory、agent behavior 與發佈流程。
- Filesystem / FileOrganizer / Trash 尚無實作；Desktop Pet 啟動流程亦尚無 effective UID 為 0 時拒絕執行的檢查。
- Observability、完整部署、自動化 E2E 測試仍未完成。

## 最終目標

### Web Chat

建立 **Multi-topic + Multi-user + Multi-AI-Persona** 即時聊天室。

最終 AI Persona 應是房間中的正式參與者，具備：

- Persona identity
- Conversation context
- Memory
- Speaker selection
- Turn-taking
- Reply target
- Cooldown / silence policy
- Turn budget 與受控 AI ↔ AI 對話
- Streaming response
- 可辨識但自然融入群聊的 AI 身份，不冒充真實人物

最終由 provider abstraction 接入 OpenAI HTTP/SSE，並完成 portable persistence、OpenTelemetry traces / metrics / logs、VirtualBox Linux VM、nginx、systemd、TLS 與 production hardening。後續依需求加入 Retriever abstraction 下的 RAG、MCP、Agent Runtime、Domain Events、自製 Queue 與 Transactional Outbox。

### Desktop AI Pet

建立具備 **Learning / Teaching Agent** 能力的桌面 AI 角色。

最終能力包含：

- 真實 AI provider
- Streaming conversation
- Persistent memory
- 主動互動
- Learner / knowledge state
- 主動確認理解
- 出題與 Active Recall
- Spaced Repetition
- Difficulty Adaptation
- Teaching Strategy
- RAG
- MCP
- Agent Runtime
- OpenTelemetry AI Observability
- 授權範圍內的環境觀察、檔案整理與 Trash metadata；move / consume / delete 以預覽、重新驗證與明確確認為前提
- 可檢視、修改、刪除的 memory / preference / policy，以及可追蹤的主動決策

## 後續缺口

### Web Chat

優先缺口：

1. 完成 Room create use case、repository write port 與 UI 接線；目前首頁產生的房間 ID 未入庫，WebSocket lookup 因而回傳 404。
2. Message persistence 與 history loading。
3. Server/application 層的 ID、空白訊息、內容長度與速率驗證；目前空白內容只由 UI 過濾，直接發送 WebSocket event 仍可繞過。
4. User identity、login、session、authorization；目前所有訊息均為 `AnonymousUser`，UI 一律顯示 `You`，無法辨識其他發言者。Nested / protected routes 與 ServerFn 亦尚未實作。
5. Connection state、reconnect、send queue、漏失訊息補回；browser socket URL 目前固定為 `ws://localhost:3000`，需改為依頁面 origin 與 HTTPS 決定 host / `wss`。斷線、連線未就緒與送出失敗亦需提供 UI 回饋。
6. Typing / presence 完整 wiring。
7. 真實 AI provider 與 streaming。
8. `AiPersona`、`RoomAiMember`、reply policy、speaker selection、turn budget、cooldown 與受控 AI ↔ AI orchestration。
9. 慢速 subscriber 的事件漏失恢復；目前 broadcast capacity 為 128，lagged 只記錄跳過數量，尚未補回。
10. Production-ready observability、deployment 與連線資源限制。
11. Domain/application events、自製 Queue、durable delivery 與 Transactional Outbox；目前僅有 process-local broadcast。
12. 隱私與日誌整理；browser socket 目前會把完整收到的 JSON 寫入 console，尚無 tracing subscriber 或 OpenTelemetry 接線。

### Desktop AI Pet

優先缺口：

1. 移動與 Pet state、使用者拖曳之間的協調；work-area bounds 與碰邊反向已完成基線。
2. FileOrganizer scan / suggestion / hash duplicate preview、TrashFeedSource，以及受控 move / consume；目前沒有對應 source、port 或 runtime 接線。
3. Filesystem 安全邊界與 non-root runtime guard：authorized root、opaque ID、path traversal / symlink escape 防護、操作前重新驗證、preview 與 confirmation。
4. 真實 AI provider、streaming conversation 與 conversation history。
5. Persistent memory、learner model、owner feedback learning 與主動 Learning / Teaching loop。
6. Retriever abstraction、RAG、MCP、Agent Runtime。
7. Domain/application events、自製 Queue / outbox、retry、dead-letter、idempotency 與 graceful shutdown。
8. OpenTelemetry distributed trace / metrics / logs，以及敏感資料的日誌邊界。
9. Installer、updater 與 release pipeline。

## 共同實作規則

1. 每完成一章，先讀取並 code review 實際 GitHub repository，再更新 README 的目前現況、已完成項目與後續缺口。
2. 下一章或下一個功能必須依 code review 後的實際 repository 缺口決定，不能只沿用既有 roadmap 推測程式結構。
3. Repository 的實際程式碼是功能完成與否的主要依據；Notion roadmap 或 Day 編號不能取代實際驗證。
4. 實作前核對官方 **latest stable** 與 API 差異，不直接採用 alpha、beta、RC。
5. Dependency 升級必須連同程式修改與驗證一起完成。
6. `Cargo.toml` dependency requirement 只寫 major/minor，例如 `leptos = "0.8"`、`tokio = "1.53"`。
7. 精確解析版本由 `Cargo.lock` 保留；可重現建置與 CI 優先使用 `--locked`。
8. `[package].version` 保留完整 SemVer，例如 `0.1.0`。
9. Database / domain 欄位優先使用跨資料庫通用表示，避免不必要地把 domain 綁定特定 database 型別。
10. 每一章只完成可驗證的小目標；完成後再依實際 repository 缺口決定下一章。
11. 未來規劃、預留 protocol、placeholder crate 或尚未 wiring 的能力不得標示為已完成。
12. GitHub README 只維護 repository 的實際程式說明，不承擔章節式教學內容。
13. Domain / application 不依賴 Axum、Tauri、Leptos、SQLx、OS API、模型 SDK 或特定 broker；以 port / adapter 接入 infrastructure。Tauri 負責 desktop composition root，Leptos 透過 command / event / bridge 投影狀態。
14. OpenAI provider 直接使用 HTTP/SSE，不使用 SDK；RAG 使用 Retriever abstraction，不使用 pgvector，也不把核心 schema 綁定特定 vector database。
15. ID 可使用 UUID 生成後轉為字串，但 domain 與資料庫 ID 欄位不採 PostgreSQL UUID 型別；目前 `rooms.id` / `category_room_id` 為 `varchar(40)`。其他 DB adapter 尚未實作，SQL 方言與 migrations 的可攜性仍需另行驗證。
16. TDD / DDD 在規則與複雜度需要時漸進導入，不預先建立空架構。必要重構先鎖住行為，再驗證 caller migration 與 regression。
17. 事件驅動先從 process-local events 與 modular monolith 開始；有獨立部署、failure isolation 或 scaling 需求後才評估 microservices。
18. Queue 優先自製：FIFO、bounded capacity、backpressure、ACK/NACK、retry/backoff、visibility timeout、DLQ、idempotency、shutdown 與 telemetry；需要 durability 時加入 persistence-backed queue / Transactional Outbox，再依實際瓶頸比較外部 broker。現有 `tokio::broadcast` 不代表已完成此 Queue。
19. Desktop Pet 必須以一般登入使用者執行；帳號可有 sudo 權限，但 process effective UID 不得為 0，禁止以 `sudo` 啟動。OS 設定可由具 sudo 權限的使用者操作；runtime guard 目前仍是缺口。
20. Filesystem 操作只接受 authorized root 內經驗證的 opaque ID；防止 path traversal 與 symlink escape。先 preview，破壞性操作需明確確認與重新驗證；telemetry 不記錄檔案內容、完整私人 path 或不必要敏感資料。
21. native / wasm32 分別使用正確 target 與 features 驗證；compile / CI 通過不取代必要 GUI/runtime 驗收。
22. Notion 才是深入淺出的**實作教學**：每章包含實際修改位置、必要程式碼、操作步驟、測試、預期結果與設計理由。舊 Day 的落差修正放入下一個合理章節；最新 Day 子頁排在最上方，章節細節不複製到 README。

## 測試與 CI 範圍

GitHub Actions：`.github/workflows/rust.yml`

觸發：

- push to `main`
- pull request to `main`

### Check

`rust.yml` 以 `check-chat`、`check-pet`、`unit-test` 與 `unit-of-work-test` jobs 驗證程式碼。Check jobs 執行：

```bash
cargo check -p server --features ssr
cargo check -p ui --features ssr
cargo check -p ui --features hydrate --target wasm32-unknown-unknown

cargo check --workspace --exclude desktop-pet-frontend
cargo check -p desktop-pet-frontend --target wasm32-unknown-unknown
```

`unit-test` job 執行：

```bash
cargo test -p desktop-pet
cargo test -p ai-core
cargo test -p chat-application
cargo test -p chat-domain
cargo test -p pet-domain
cargo test -p shared
cargo test -p ui
```

`unit-of-work-test` job 啟動 `postgres:latest` service，設定臨時 `DATABASE_URL`，執行 migrations 與 persistence tests：

```bash
sqlx migrate run
cargo test -p persistence
```

### Build

`build-chat` 與 `build-pet` jobs 執行：

```bash
cargo build --workspace --exclude desktop-pet-frontend --verbose
cargo leptos build --release

cd apps/desktop_pet/frontend
trunk build --release
```

目前已核對 [Rust workflow Run #98](https://github.com/williechen/rust_ai_chat/actions/runs/37185855876)：對應上述 commit，六個 jobs 全部成功。本輪未在本機重跑測試。

`unit-of-work-test` 是 job 名稱，目前實際測試的是 Room repository 查詢與 migrations，尚無 Unit of Work transaction abstraction。`cargo test -p chat-domain` 目前沒有自訂測試案例；不能以命令通過推定 domain 行為已全面覆蓋。

### 尚未納入 CI 的主要範圍

- `cargo fmt --check`、Clippy（VS Code task 存在，但 GitHub workflow 未執行）
- Desktop frontend 的 `project_pose` 單元測試（目前只 check / build WASM）
- Server / room hub 行為測試與不存在房間、跨房間、lagged 等回歸測試
- Browser E2E
- WebSocket E2E
- Tauri desktop E2E
- Installer / release verification
- Deployment verification
- OpenTelemetry integration verification

Persistence integration test 已納入 GitHub Actions 的 `unit-of-work-test` job：CI 會啟動 PostgreSQL service container、設定臨時 `DATABASE_URL`、執行 migrations，再執行 `cargo test -p persistence`。Repository test 使用 `#[sqlx::test(migrations = "../../migrations")]`；SQLx 會為測試建立隔離 database 並套用 migrations，測試本身只建立必要 fixture，不依賴本機既有 seed，也不需要手動 cleanup。

## 開發與執行

Rust toolchain 使用 stable，workspace 使用 Rust edition 2024、resolver 3。

### 基本工具

```bash
rustup target add wasm32-unknown-unknown

cargo install cargo-leptos --locked
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
```

Linux native build 需要：

```bash
sudo apt-get install -y \
  build-essential pkg-config curl wget file \
  libssl-dev libxdo-dev librsvg2-dev libwebkit2gtk-4.1-dev
```

### PostgreSQL

先安裝並啟動 PostgreSQL，以下操作使用自己的資料庫帳號。

建立 database：

```sql
CREATE DATABASE rust_ai_chat;
```

在 repository root 建立 `.env`：

```dotenv
RUST_LOG=debug
DATABASE_URL=postgres://username:password@localhost:5432/rust_ai_chat
```

Server 啟動時會執行 migration；目前 Web Chat 啟動依賴 PostgreSQL 可連線。

### Web Chat

開發：

```bash
cargo leptos watch
```

Release build：

```bash
cargo leptos build --release
```

Release 執行（在 repository root，保留 `target/site` 並確保 `.env` 可讀）：

```bash
LEPTOS_OUTPUT_NAME=rust_ai_chat LEPTOS_SITE_ROOT=target/site LEPTOS_SITE_PKG_DIR=pkg ./target/release/server
```

Server 目前在程式內固定綁定 `127.0.0.1:3000`，並非可透過 listen-address 環境變數調整。

預設開啟：

```text
http://127.0.0.1:3000
```

目前須先在上述 `.env` 指向的資料庫建立測試房間，再直接開啟房間網址；首頁按鈕尚不能完成建立流程：

```sql
INSERT INTO rooms (id, category_room_id, name, created_at, updated_at)
VALUES ('room-rust', 'category-programming', 'Rust', CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (id) DO NOTHING;
```

開啟 `http://localhost:3000/room/room-rust`；可用兩個分頁測試同房間 broadcast。訊息目前只存在記憶體，重新載入不會取得歷史。browser socket 固定連到 localhost，因此此操作方式僅適用本機開發。

### Desktop AI Pet

```bash
cd apps/desktop_pet
cargo tauri dev
```

Tauri 會依設定啟動 frontend development server；單獨以一般 browser 開啟 frontend 無法取代完整 Tauri runtime。

目前 `tauri.conf.json` 的 `bundle.active` 為 `false`，CI native build 與 Trunk build 不代表已產生安裝包。桌面移動與選單需在實際桌面環境驗證；CI 未驗證視窗管理器行為。
