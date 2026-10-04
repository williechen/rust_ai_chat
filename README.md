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

目前實作基線：

- Web Chat 已具備 Axum + Leptos SSR/hydration、room route、WebSocket 即時 broadcast。
- Chat domain / protocol 的主要識別碼使用跨資料庫較通用的字串表示。
- RoomRepository trait 與 PostgreSQL adapter 已存在，並已注入 server runtime；WebSocket upgrade 前會透過 repository 驗證 room 是否存在。
- Web Chat 尚未完成 message persistence、history、login/session、authorization、reconnect、presence、typing、AI streaming 與 persona runtime。
- AI Core 已有 `ChatModel` abstraction 與 `MockChatModel`，目前真實 AI provider 尚未接入 Web Chat。
- Desktop AI Pet 已有 Tauri managed state、Pet state machine、frontend bridge、mock chat、native context-menu command dispatch，以及具 work-area bounds、碰邊反向、位置變更驗證與 stall 停止機制的桌面移動。
- Desktop AI Pet 尚未完成依 PetState / 使用者拖曳協調移動、真實 AI provider、streaming、memory、agent behavior 與發佈流程。
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
- Streaming response
- 可辨識但自然融入群聊的 AI 身份

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

## 後續缺口

### Web Chat

優先缺口：

1. Room create use case 與 UI / application wiring；目前 room lookup 已用於 WebSocket 連線驗證。
2. Message persistence 與 history loading。
3. ID validation 與 application-level input validation。
4. User identity、login、session、authorization。
5. Connection state、reconnect、send queue、漏失訊息補回。
6. Typing / presence 完整 wiring。
7. 真實 AI provider 與 streaming。
8. AI Persona domain、runtime 與 orchestration。
9. Production-ready observability 與 deployment。

### Desktop AI Pet

優先缺口：

1. 移動與 Pet state、使用者拖曳之間的協調；work-area bounds 與碰邊反向已完成基線。
2. 真實 AI provider。
3. Streaming conversation 與 conversation history。
4. Persistent memory。
5. Learning / Teaching agent loop。
6. RAG、MCP、Agent Runtime。
7. OpenTelemetry AI Observability。
8. Installer、updater 與 release pipeline。

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
13. Notion 才是深入淺出的**實作教學**：每章必須包含實際修改位置、完整或關鍵程式碼、操作步驟、測試/驗證方式、預期結果與必要的設計理由；不能只寫深入淺出的概念說明。

## 測試與 CI 範圍

GitHub Actions：`.github/workflows/rust.yml`

觸發：

- push to `main`
- pull request to `main`

### Check

目前 CI 執行：

```bash
cargo check -p server --features ssr
cargo check -p ui --features ssr
cargo check -p ui --features hydrate --target wasm32-unknown-unknown

cargo check --workspace --exclude desktop-pet-frontend
cargo check -p desktop-pet-frontend --target wasm32-unknown-unknown

cargo test -p desktop-pet --locked
cargo test -p ai-core
cargo test -p chat-application
cargo test -p chat-domain
cargo test -p pet-domain
cargo test -p shared
cargo test -p ui
```

Persistence CI 另外執行：

```bash
sqlx migrate run
cargo test -p persistence --locked
```

### Build

目前 CI 執行：

```bash
cargo build --workspace --exclude desktop-pet-frontend --verbose
cargo leptos build --release

cd apps/desktop_pet/frontend
trunk build --release
```

### 尚未納入 CI 的主要範圍

- Browser E2E
- WebSocket E2E
- Tauri desktop E2E
- Installer / release verification
- Deployment verification
- OpenTelemetry integration verification

Persistence integration test 已納入 GitHub Actions 的 `persistence-test` job：CI 會啟動 PostgreSQL 17 service container、設定臨時 `DATABASE_URL`、執行 migrations，再執行 `cargo test -p persistence --locked`。現有 `test_room_repository_find_by_id` 會自行建立與清理測試資料，不依賴本機既有 seed。

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

預設開啟：

```text
http://127.0.0.1:3000
```

### Desktop AI Pet

```bash
cd apps/desktop_pet
cargo tauri dev
```

Tauri 會依設定啟動 frontend development server；單獨以一般 browser 開啟 frontend 無法取代完整 Tauri runtime。
