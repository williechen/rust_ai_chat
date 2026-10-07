# Rust AI Chat

## 大綱

Rust AI Chat 是一個持續演進的 Rust workspace，目前包含兩個產品方向：

- **Web Chat**：以 Rust、Axum、Leptos、WebSocket 與 PostgreSQL 建構多人即時聊天室，最終加入多 AI Persona。
- **Desktop AI Pet**：以 Rust、Tauri、Leptos 與 AI model abstraction 建構可互動、可學習、可教學的桌面 AI 角色。

目前 workspace 主要結構：

| 路徑 | 實際責任 |
| --- | --- |
| `apps/server` | Axum Web server、Leptos SSR、HTTP API、WebSocket、room hub、PostgreSQL 啟動 |
| `apps/desktop_pet` | Tauri desktop application、managed state、commands、桌面移動、FileOrganizer 基線 |
| `apps/desktop_pet/frontend` | Desktop Pet 的 Leptos WASM frontend |
| `crates/chat_domain` | Room、ChatMessage、MessageAuthor 等 chat domain model |
| `crates/chat_application` | Chat use case、RoomRepository port、room create use case |
| `crates/shared` | WebSocket protocol 與 Create Room HTTP contract |
| `crates/ui` | Web Chat 的 Leptos Router、SSR/hydration、room create browser API、browser socket adapter |
| `crates/persistence` | PostgreSQL connection、migration、Room repository read/write adapter |
| `crates/ai_core` | AI model abstraction、request/response、MockChatModel |
| `crates/pet_domain` | Desktop Pet state machine |
| `crates/telemetry` | Observability 預留 crate，目前尚未完成 |
| `migrations` | Database migrations |
| `deploy` | 部署相關設定，目前仍以 placeholder 為主 |
| `tests` | Repository-level test 預留目錄；目前多數測試仍位於各 crate 內 |

Phase 1 正式封版基線：branch `Phase1`、tag `Phase-1(Chat&Pet)`，固定指向 commit `2fc1c0f86e047379c4f5a934d70a866f1b0550a7`（2026-10-07，Asia/Taipei）。Phase 2 從此封版基線之上繼續於 `main` 演進；後續 code review 一律以當下 `main` HEAD 與實際 source 為準。

目前實作基線：

- Web Chat 已具備 Axum + Leptos SSR/hydration、首頁與 `/room/:room_id` 路由、`/health`、同房間 WebSocket broadcast、Ping/Pong，以及 message room ID 與 connection room ID 一致性檢查。
- Room create 流程已接通：首頁呼叫同源 `POST /api/rooms`，server 建立 UUID，application `create_room` 呼叫 repository `create` 寫入 PostgreSQL，成功回傳 room ID 後前端才 navigate。
- Create Room HTTP request / response contract 已移至 `shared`，server 與 browser 共用。
- RoomRepository trait 與 PostgreSQL adapter 已具備 `find_by_id` / `create`；WebSocket upgrade 前會驗證 room 是否存在。
- Persistence tests 已涵蓋 room lookup 與 create-then-find。
- Web Chat 訊息目前仍只在記憶體 broadcast，尚無 message persistence / history。
- `Typing`、`PresenceChanged`、`AiDelta`、`AiCompleted` 仍只是部分 protocol 基線，尚未完整 wiring。
- AI Core 已有 `ChatModel` abstraction 與 `MockChatModel`；真實 AI provider 尚未接入 Web Chat。
- Desktop AI Pet 已有 Tauri managed state、Pet state machine、frontend bridge、mock chat、native context-menu command dispatch，以及 work-area bounds、碰邊反向、位置變更驗證與 stall 停止。
- FileOrganizer 已有 authorized root canonicalization、Unix root EUID 拒絕、regular-file scan、symlink skip、opaque item ID、size candidate grouping、SHA-256 duplicate preview 與對應單元測試。
- FileOrganizer 尚未接入 Tauri command / UI /完整 runtime workflow，也尚未實作 move / consume / Trash。
- Observability、完整 deployment、自動化 E2E 與 release pipeline 仍未完成。

## 最終目標

### Web Chat

建立 **Multi-topic + Multi-user + Multi-AI-Persona** 即時聊天室。

最終能力包含：

- User identity、login、session、authorization
- Message persistence 與 history
- Typing / presence
- Reconnect、send queue、漏失訊息恢復
- 真實 AI provider 與 streaming
- Persona identity
- Conversation context
- Memory
- Speaker selection
- Turn-taking
- Reply target
- Cooldown / silence policy
- Turn budget 與受控 AI ↔ AI 對話
- OpenTelemetry traces / metrics / logs
- Production deployment 與 hardening
- 依實際需求逐步加入 RAG、MCP、Agent Runtime、Domain Events、Queue 與 Transactional Outbox

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
- 授權範圍內的環境觀察、檔案整理與 Trash metadata
- move / consume / delete 的 preview、重新驗證與明確確認
- 可檢視、修改、刪除的 memory / preference / policy
- 可追蹤的主動決策
- Installer、updater 與 release pipeline

## 後續缺口

### Web Chat

1. Message persistence 與 history loading。
2. Server/application 層的 ID、空白訊息、內容長度與速率驗證。
3. User identity、login、session、authorization；目前訊息仍為匿名，Nested / protected routes 與 ServerFn 尚未實作。
4. Connection state、reconnect、send queue 與漏失訊息恢復。
5. Browser WebSocket URL 目前仍固定為 `ws://localhost:3000`，尚未依頁面 origin / HTTPS 自動切換 host 與 `wss`。
6. Typing / presence 完整 wiring。
7. 真實 AI provider 與 streaming。
8. `AiPersona`、`RoomAiMember`、reply policy、speaker selection、turn budget、cooldown 與受控 AI ↔ AI orchestration。
9. 慢速 subscriber 的事件漏失恢復；目前 process-local broadcast lag 只記錄 skipped count。
10. Production-ready observability、deployment 與連線資源限制。
11. Domain/application events、Queue、durable delivery 與 Transactional Outbox。
12. 隱私與日誌邊界；tracing subscriber / OpenTelemetry 尚未完整接線。

### Desktop AI Pet

1. 移動與 PetState、使用者拖曳之間的協調。
2. 將現有 FileOrganizer scan / duplicate preview 接入 Tauri command、UI 與 application workflow。
3. FileOrganizer suggestion、受控 move / consume、TrashFeedSource 與 Trash metadata。
4. Filesystem 操作前重新驗證、path traversal / symlink race 防護、preview / confirmation，以及應用啟動層級的 non-root runtime guard。
5. 真實 AI provider、streaming conversation 與 conversation history。
6. Persistent memory、learner model、owner feedback learning 與主動 Learning / Teaching loop。
7. Retriever abstraction、RAG、MCP、Agent Runtime。
8. Domain/application events、Queue / outbox、retry、dead-letter、idempotency 與 graceful shutdown。
9. OpenTelemetry distributed trace / metrics / logs，以及敏感資料的日誌邊界。
10. Installer、updater 與 release pipeline。

## 共同實作規則

1. 每完成一章，先 code review 實際 GitHub repository，再更新 README 的目前現況與後續缺口。
2. 下一章或下一個功能依 code review 後的實際 repository 缺口決定，不只沿用 roadmap 或 Day 編號推測程式結構。
3. **Repository = 實際程式說明；Notion = 深入淺出實作教學。** Phase 2 起只以 Notion「Rust AI Chat + Desktop Pet｜共同主頁」作為唯一 roadmap、共同規則與進度入口；舊 Chat / Desktop Pet 主頁只保留 Phase 1 歷史資料。
4. README 只維護本文件六個區塊：大綱、最終目標、後續缺口、共同實作規則、測試與 CI 範圍、開發與執行。
5. 教學內容只更新 Notion；每章應包含實際修改位置、必要程式碼、操作步驟、測試 / 驗證、預期結果與設計理由，不複製到 README。
6. Repository 的實際程式碼是功能完成與否的主要依據；Notion roadmap 或 Day 編號不能取代實際驗證。
7. 實作前核對官方 latest stable 與 API 差異，不直接採用 alpha / beta / RC。
8. Dependency 升級必須連同程式修改與驗證一起完成。
9. `Cargo.toml` dependency requirement 只寫 major/minor；精確解析版本由 `Cargo.lock` 保留。
10. `[package].version` 保留完整 SemVer。
11. Database / domain ID 優先使用跨資料庫通用表示，避免 domain 不必要地綁定特定 database 型別。
12. Domain / application 不依賴 Axum、Tauri、Leptos、SQLx、OS API、模型 SDK 或特定 broker；以 port / adapter 接入 infrastructure。
13. OpenAI provider 使用 HTTP/SSE abstraction；RAG 使用 Retriever abstraction，不把核心 schema 綁定特定 vector database。
14. TDD / DDD 依規則與複雜度漸進導入，不預先建立空架構。
15. 事件驅動先從 modular monolith / process-local events 開始，只有在獨立部署、failure isolation 或 scaling 需求出現後才評估 microservices。
16. Queue 需要明確涵蓋 bounded capacity、backpressure、ACK/NACK、retry/backoff、visibility timeout、DLQ、idempotency、shutdown 與 telemetry；現有 `tokio::broadcast` 不視為完成 Queue。
17. Desktop Pet 必須以一般登入使用者執行；帳號可以有 sudo 權限，但 process effective UID 不得為 0。
18. Filesystem 操作只接受 authorized root 內經驗證的 opaque ID；破壞性操作前必須重新驗證、preview 並取得明確確認。
19. Telemetry 不記錄檔案內容、完整私人 path 或不必要的敏感資料。
20. Native / wasm32 分別使用正確 target 與 features 驗證；compile / CI 通過不能取代必要 GUI/runtime 驗收。
21. 未來規劃、預留 protocol、placeholder crate 或尚未 wiring 的能力不得標示為已完成。
22. Phase 2 Day 編號重新從 Day 1 開始，不沿用 Phase 1 舊 Day 編號；Chat 起跑 vertical slice 為 message persistence → history loading，Desktop Pet 起跑 vertical slice 為 Leptos UI → bridge → Tauri command/application → `scan_preview()` → `ScanPreview` → UI，且每天仍須先依當下 repository code review 再確認最小實作範圍。

## 測試與 CI 範圍

GitHub Actions：`.github/workflows/rust.yml`

觸發：

- push to `main`
- pull request to `main`

目前 jobs：

- `check-chat`
- `check-pet`
- `unit-test`
- `unit-of-work-test`
- `build-chat`
- `build-pet`

主要 check：

```bash
cargo check -p server --features ssr
cargo check -p ui --features ssr
cargo check -p ui --features hydrate --target wasm32-unknown-unknown

cargo check --workspace --exclude desktop-pet-frontend
cargo check -p desktop-pet-frontend --target wasm32-unknown-unknown
```

Unit tests：

```bash
cargo test -p desktop-pet
cargo test -p ai-core
cargo test -p chat-application
cargo test -p chat-domain
cargo test -p pet-domain
cargo test -p shared
cargo test -p ui
```

Persistence integration test：

```bash
sqlx migrate run
cargo test -p persistence
```

`unit-of-work-test` 會啟動 `postgres:latest` service、設定臨時 `DATABASE_URL`、執行 migrations，再執行 persistence tests。目前涵蓋 Room repository lookup 與 create-then-find；job 名稱不代表已存在 Unit of Work transaction abstraction。

Build：

```bash
cargo leptos build --release

cargo build --workspace --exclude desktop-pet-frontend --verbose

cd apps/desktop_pet/frontend
trunk build --release
```

目前尚未納入 CI 的主要範圍：

- `cargo fmt --check`
- Clippy
- Desktop frontend 的 WASM 單元測試
- Server / RoomHub 行為回歸測試
- Browser E2E
- WebSocket E2E
- Tauri desktop E2E
- Installer / release verification
- Deployment verification
- OpenTelemetry integration verification

## 開發與執行

Rust toolchain 使用 stable，workspace 使用 Rust edition 2024、resolver 3。

基本工具：

```bash
rustup target add wasm32-unknown-unknown

cargo install cargo-leptos --locked
cargo install trunk --locked
cargo install tauri-cli --version "^2" --locked
```

Linux native build dependencies：

```bash
sudo apt-get install -y \
  build-essential pkg-config curl wget file \
  libssl-dev libxdo-dev librsvg2-dev libwebkit2gtk-4.1-dev
```

PostgreSQL：

```sql
CREATE DATABASE rust_ai_chat;
```

Repository root `.env`：

```dotenv
RUST_LOG=debug
DATABASE_URL=postgres://username:password@localhost:5432/rust_ai_chat
```

Server 啟動時會執行 migration；Web Chat 目前依賴 PostgreSQL 可連線。

Web Chat 開發：

```bash
cargo leptos watch
```

Release build：

```bash
cargo leptos build --release
```

Release 執行：

```bash
LEPTOS_OUTPUT_NAME=rust_ai_chat \
LEPTOS_SITE_ROOT=target/site \
LEPTOS_SITE_PKG_DIR=pkg \
./target/release/server
```

Server 目前固定綁定：

```text
http://127.0.0.1:3000
```

首頁 `Create Room` 已接上 `POST /api/rooms`；建立成功後才 navigate 到新 room。訊息目前仍只存在記憶體，重新載入不會取得歷史；browser WebSocket URL 目前固定使用 localhost，因此現況主要適用本機開發。

Desktop AI Pet：

```bash
cd apps/desktop_pet
cargo tauri dev
```

Tauri 會啟動 frontend development server；單獨以一般 browser 開啟 frontend 無法取代完整 Tauri runtime。

目前 `tauri.conf.json` 的 `bundle.active` 為 `false`；CI native build 與 Trunk build 不代表已產生安裝包。桌面移動、native menu 與 filesystem workflow 仍需要實際桌面環境驗收。
