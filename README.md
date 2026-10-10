# Rust AI Chat

## 大綱

Rust AI Chat 是一個持續演進的 Rust workspace，目前包含三個產品方向：

- **Web Chat**：以 Rust、Axum、Leptos、WebSocket 與 PostgreSQL 建構多人即時聊天室，最終加入多 AI Persona。
- **Desktop AI Pet**：以 Rust、Tauri、Leptos 與 AI model abstraction 建構可互動、可學習、可教學的桌面 AI 角色。
- **Stats Server / Dashboard 後端基線**：獨立的 Axum HTTP 服務，先建立事件輸入契約與驗證；尚未實作授權、儲存、查詢及 dashboard UI。

目前 workspace 主要結構：

| 路徑 | 實際責任 |
| --- | --- |
| `apps/server` | Axum Web server、Leptos SSR、HTTP API、WebSocket、room hub、PostgreSQL 啟動 |
| `apps/stats_server` | 獨立 Axum stats HTTP server、`/healthz`、`/api/v1/events` 輸入驗證及契約測試 |
| `apps/desktop_pet` | Tauri desktop application、managed state、commands、桌面移動、FileOrganizer 基線 |
| `apps/desktop_pet/frontend` | Desktop Pet 的 Leptos WASM frontend |
| `crates/chat_domain` | Room、ChatMessage、MessageAuthor 等 chat domain model |
| `crates/chat_application` | Chat use case、RoomRepository / MessageRepository ports、room create、先儲存再發布流程 |
| `crates/shared` | WebSocket protocol 與 Create Room HTTP contract |
| `crates/ui` | Web Chat 的 Leptos Router、SSR/hydration、room create browser API、browser socket adapter |
| `crates/persistence` | PostgreSQL connection、migrations、Room / Message repository adapter |
| `crates/ai_core` | AI model abstraction、request/response、MockChatModel |
| `crates/pet_domain` | Desktop Pet state machine、`PetProfile` 與外觀覆寫資料契約 |
| `crates/telemetry` | Observability 預留 crate，目前尚未完成 |
| `migrations` | Database migrations |
| `deploy` | 部署相關設定，目前仍以 placeholder 為主 |
| `tests` | Repository-level test 預留目錄；目前多數測試仍位於各 crate 內 |

Phase 1 正式封版基線：branch `Phase1`、tag `Phase-1(Chat&Pet)`，固定指向 commit `2fc1c0f86e047379c4f5a934d70a866f1b0550a7`（2026-10-07，Asia/Taipei）。Phase 2 從此封版基線之上繼續於 `main` 演進；後續 code review 一律以當下 `main` HEAD 與實際 source 為準。本次文件同步基準為 2026-10-10 Phase 2 Day 5，commit `21d2ac5a8f2f2733d2b7ac01d498a29ea34eb6f3`。

目前實作基線：

- Web Chat 已具備 Axum + Leptos SSR/hydration、首頁與 `/room/:room_id` 路由、`/health`、同房間 WebSocket broadcast、Ping/Pong，以及 message room ID 與 connection room ID 一致性檢查。
- Room create 流程已接通：首頁呼叫同源 `POST /api/rooms`，server 建立 UUID，application `create_room` 呼叫 repository `create` 寫入 PostgreSQL，成功回傳 room ID 後前端才 navigate。
- Create Room HTTP request / response contract 已移至 `shared`，server 與 browser 共用。
- RoomRepository trait 與 PostgreSQL adapter 已具備 `find_by_id` / `create`；WebSocket upgrade 前會驗證 room 是否存在。
- Persistence tests 已涵蓋 room lookup 與 create-then-find。
- Web Chat Phase 2 已加入 `MessageRepository`、PostgreSQL `messages` migration 與 `persist_then_publish` application 流程。WebSocket 傳訊會先寫入 repository，成功才向房間 broadcast；失敗則回傳 `message_persistence_failed`。已新增此順序與失敗不發布的 mock repository 測試；完整 PostgreSQL / WebSocket runtime 驗收狀態仍需以 CI 及實測確認，history loading 尚未實作。
- Phase 2 Day 5 新增 `RoomHub::subscriber_count` 與 server 測試：房間間事件隔離、同房間多訂閱者、最後訂閱者 cleanup、訂閱數生命週期。這些是 process-local unit tests，不等於 WebSocket E2E。
- `Typing`、`PresenceChanged`、`AiDelta`、`AiCompleted` 仍只是部分 protocol 基線，尚未完整 wiring。
- AI Core 已有 `ChatModel` abstraction 與 `MockChatModel`；真實 AI provider 尚未接入 Web Chat。
- Desktop AI Pet 已有 Tauri managed state、Pet state machine、frontend bridge、mock chat、native context-menu command dispatch，以及 work-area bounds、碰邊反向、位置變更驗證與 stall 停止。
- FileOrganizer 已有 authorized root canonicalization、Unix root EUID 拒絕、regular-file scan、symlink skip、opaque item ID、size candidate grouping、SHA-256 duplicate preview 與對應單元測試。
- FileOrganizer 的 `scan_file_preview()` 唯讀 Tauri command 已透過 JS / WASM bridge 接到 frontend，Leptos `App` 啟動時呼叫掃描；目前成功結果尚未呈現於 UI，錯誤僅寫入既有 error signal。Desktop Pet main 已加入 `dotenv::dotenv().ok()`，允許從 `.env` 讀取 `PET_AUTHORIZED_ROOT`。尚未完成完整 GUI/runtime 驗收，也未實作 move / consume / Trash。
- Desktop Pet Phase 2 已加入 `PetProfile` 五個必要欄位（名字、種類、喜好、個性、興趣）與可選外觀（顏色、風格），支援 serde JSON、非空白驗證與 Day 5 `decode_profile` / `encode_profile` codec（先驗證再序列化），並新增相關 domain tests；尚未接入 UI、持久化或 AI 行為。
- Stats Server 已建立獨立 Axum app：`GET /healthz` 回傳 200；`POST /api/v1/events` 只接受 `web_chat` / `desktop_pet` 來源、非空白且不超過 128 UTF-8 bytes 的 `event_type`，限制 JSON body 4096 bytes 並拒絕未知欄位。有效輸入刻意回傳 `501 Not Implemented`，因尚無 auth / DB；目前僅綁定 `127.0.0.1:3100`。已有輸入及 HTTP contract tests。
- Stats Server Phase 2 Day 5 加入 `EventIdentity(source_app, event_id)` 與 `InMemoryDeduplicator` 的 `HashSet` 去重及對應測試；目前仍是獨立純記憶體元件，**尚未接入 HTTP ingest**，有效事件仍回傳 501，無 durable deduplication。
- CI 已從單一 `rust.yml` 拆分為 `ai_chat.yml`、`desktop_pet.yml`、`dashboard.yml`、`persistence.yml`、`test.yml`、`node.yml`；各 job 的執行結果需另查 GitHub Actions。
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

### Stats Server / Dashboard

目標是以具備身分驗證、可靠持久化與查詢能力的獨立服務支援 Web Chat / Desktop Pet 事件統計與視覺化。目前只有 HTTP contract 基線；後續需定義可追蹤的事件結構、儲存與查詢、授權及 dashboard UI。

## 後續缺口

### Web Chat

1. Message persistence 已具備先儲存再 broadcast 的 application / WebSocket 接線與 mock 測試，仍需確認 PostgreSQL integration、WebSocket runtime / 失敗路徑驗收；history loading 尚未實作。
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
2. FileOrganizer 唯讀 Tauri command 已接 frontend bridge 且在 App 啟動時呼叫，但目前未顯示掃描結果；仍需完整 runtime 驗證、可操作的預覽 UI 與 application workflow。
3. FileOrganizer suggestion、受控 move / consume、TrashFeedSource 與 Trash metadata。
4. Filesystem 操作前重新驗證、path traversal / symlink race 防護、preview / confirmation，以及應用啟動層級的 non-root runtime guard。
5. `PetProfile` 已有資料契約、JSON encode/decode codec 與測試，仍需 UI 編輯、持久化儲存及 profile 驅動的角色行為；真實 AI provider、streaming conversation 與 conversation history 尚未實作。
6. Persistent memory、learner model、owner feedback learning 與主動 Learning / Teaching loop。
7. Retriever abstraction、RAG、MCP、Agent Runtime。
8. Domain/application events、Queue / outbox、retry、dead-letter、idempotency 與 graceful shutdown。
9. OpenTelemetry distributed trace / metrics / logs，以及敏感資料的日誌邊界。
10. Installer、updater 與 release pipeline。

### Stats Server / Dashboard

1. 為事件 API 增加明確的驗證與授權邊界；未實作前維持 loopback-only，不對外暴露。
2. 已有記憶體 `EventIdentity` / `InMemoryDeduplicator` 基線；仍需將去重接上 ingest、定義 durable idempotency、可靠持久化、查詢與保留政策，避免將 HTTP `501` 視為成功 ingest。
3. 新增真正的 dashboard UI、統計聚合及整合測試；目前獨立 server 不代表已完成 Dashboard。

## 共同實作規則

1. 每完成一章，先 code review 實際 GitHub repository，再更新 README 的目前現況與後續缺口。
2. 下一章或下一個功能依 code review 後的實際 repository 缺口決定，不只沿用 roadmap 或 Day 編號推測程式結構。
3. **三處資訊責任固定**：GitHub repository 是功能完成與否的 source of truth；Notion「Rust AI Chat + Desktop Pet｜共同主頁」是 Phase 2 roadmap、共同規則、每日格式與進度的唯一完整規範；每日 08:00 排程只依共同主頁規則執行。
4. README 只維護本文件六個區塊：大綱、最終目標、後續缺口、共同實作規則、測試與 CI 範圍、開發與執行；不複製每日 Day 教學與詳細格式規則。
5. 每日依序處理 **聊天室（Web Chat）→ 桌面寵物（Desktop Pet）**，兩條各自一個最小、一天可完成且可驗證的 Day，分別維護編號與 Notion 教學；每個 Day 只顯示六區塊：學習大綱、實作內容、套件版本、重構內容、驗證項目、啟動程式方式。格式與去重規則以共同主頁為準。
6. Repository 的實際 source、tests、build 與 runtime 驗收是功能完成與否的主要依據；roadmap、排程或 Day 編號不能取代實際驗證。
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
22. Phase 2 Chat 與 Desktop Pet 分別從 Day 1 編號，不沿用 Phase 1；Chat 起跑方向為 message persistence → history loading，Pet 起跑方向為 FileOrganizer 唯讀 Tauri command → frontend bridge → Leptos UI。每日以當下 repository code review 決定各條的下一個最小可驗證範圍，未驗證的實作不得標示完成。
23. **語系與 i18n**：目前自有使用者可見文案以繁體中文（zh-TW）為預設；自己撰寫的標題、說明、提示、操作文字與自訂 `error` / `message` 使用繁體中文。第三方 crate、framework、database driver、OS/runtime 或外部 API 的原始 `error` / `message` 保留原文；需要面向使用者時，可在 UI/presentation 邊界補充繁體中文說明，但不改寫原始錯誤。程式名稱、方法／函式名稱、API、變數、型別、trait、crate/package、檔案路徑、CLI 指令、設定鍵與 protocol 欄位名稱保留原文。所有教學程式註解使用繁體中文。i18n/localization 為正式需求；自有文案應集中在 UI/presentation 或可替換訊息層，實際需要多語系時再導入相關 infrastructure，不預先建立空架構。

## 測試與 CI 範圍

GitHub Actions 已拆分為六個 workflow，均針對 `main` 的 push / pull request：

| Workflow | 主要 jobs |
| --- | --- |
| `.github/workflows/ai_chat.yml` | `check-chat`、`build-chat` |
| `.github/workflows/desktop_pet.yml` | `check-pet`、`build-pet` |
| `.github/workflows/dashboard.yml` | `check-dashboard`、`build-dashboard` |
| `.github/workflows/persistence.yml` | `test-persistence`（PostgreSQL service + migrations） |
| `.github/workflows/test.yml` | 分 crate 的 Rust test jobs |
| `.github/workflows/node.yml` | `test-node`（Node.js 24） |

這是 pipeline 分工調整，不代表已完成 E2E / release verification。

主要 check：

```bash
cargo check -p server --features ssr
cargo check -p ui --features ssr
cargo check -p ui --features hydrate --target wasm32-unknown-unknown

cargo check --workspace --exclude desktop-pet-frontend
cargo check -p desktop-pet-frontend --target wasm32-unknown-unknown
cargo check -p stats-server
```

Unit tests：

```bash
cargo test -p desktop-pet
cargo test -p server
cargo test -p stats-server
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

`test-persistence` 會啟動 `postgres:latest` service、設定臨時 `DATABASE_URL`、執行 migrations，再執行 persistence tests。目前涵蓋 Room repository lookup 與 create-then-find；job 名稱不代表已存在 Unit of Work transaction abstraction。

Build：

```bash
cargo leptos build --release

cargo build --workspace --exclude desktop-pet-frontend --verbose

cd apps/desktop_pet/frontend
trunk build --release
```

另有 `test-node` 執行 `node --test`（Node.js 24）；`stats-server` HTTP contract tests 涵蓋 200 / 400 / 413 / 422 / 501 回應。新增的工作與測試項目不等於所有 CI job 均已通過，實際結果以 GitHub Actions run 為準。

目前尚未納入 CI 的主要範圍：

- `cargo fmt --check`
- Clippy
- Desktop frontend 的 WASM 單元測試
- WebSocket runtime / RoomHub 整合回歸測試（Day 5 已有 RoomHub 單元測試）
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

首頁 `Create Room` 已接上 `POST /api/rooms`；建立成功後才 navigate 到新 room。訊息已接上 PostgreSQL 持久化寫入流程，但尚未提供 history loading，因此重新載入不會取得歷史；browser WebSocket URL 目前固定使用 localhost，因此現況主要適用本機開發。

Desktop AI Pet：

```bash
cd apps/desktop_pet
cargo tauri dev
```

Tauri 會啟動 frontend development server；單獨以一般 browser 開啟 frontend 無法取代完整 Tauri runtime。可在 repository root 的 `.env` 設定 `PET_AUTHORIZED_ROOT=/path/to/authorized/directory`；Desktop Pet 啟動時載入 `.env`，啟動後會執行一次唯讀 scan preview，但目前僅處理錯誤，不會在 UI 展示成功的掃描結果。

目前 `tauri.conf.json` 的 `bundle.active` 為 `false`；CI native build 與 Trunk build 不代表已產生安裝包。桌面移動、native menu 與 filesystem workflow 仍需要實際桌面環境驗收。

Stats Server（獨立開發基線）：

```bash
cargo run -p stats-server
curl -i http://127.0.0.1:3100/healthz
curl -i -X POST http://127.0.0.1:3100/api/v1/events \
  -H 'Content-Type: application/json' \
  -d '{"event_id":"00000000-0000-4000-8000-000000000001","source_app":"web_chat","event_type":"message_sent"}'
```

`/healthz` 應回傳 200；合法事件目前應回傳 501（尚未處理、也沒有寫入資料庫），不是成功收件。禁止未經授權將此服務綁定公開網路介面。
