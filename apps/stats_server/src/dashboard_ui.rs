use leptos::prelude::*;

#[component]
fn DashboardView(source_label: &'static str, total: i64) -> impl IntoView {
    view! {
        <html lang="zh-Hant">
            <head>
                <meta charset="utf-8"/>
                <title>"Analytics 管理後台"</title>
            </head>
            <body>
                <main>
                    <h1>"Analytics 管理後台"</h1>
                    <nav aria-label="事件來源">
                        <a href="/dashboard?source=web_chat">"Chat 事件"</a>
                        <a href="/dashboard?source=desktop_pet">"Pet 事件"</a>
                    </nav>
                    <a href="/change-password">"修改密碼與 Email"</a>
                    <p>{format!("{source_label} 已提交事件總數 {total}")}</p>
                    <p>"角色與來源 ACL、互動篩選尚待接通。"</p>
                    <form method="post" action="/logout">
                        <button type="submit">"登出"</button>
                    </form>
                </main>
            </body>
        </html>
    }
}

pub fn render_dashboard(source_label: &'static str, total: i64) -> String {
    format!(
        "<!doctype html>{}",
        view! { <DashboardView source_label total /> }.to_html()
    )
}
