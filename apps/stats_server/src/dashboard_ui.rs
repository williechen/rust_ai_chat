use leptos::prelude::*;

#[component]
fn DashboardView(source_label: &'static str, total: i64) -> impl IntoView {
    let chat_current = (source_label == "Chat").then_some("page");
    let pet_current = (source_label == "Pet").then_some("page");

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
                        <a href="/dashboard?source=web_chat" aria-current=chat_current>
                            "Chat 事件"
                        </a>
                        <a href="/dashboard?source=desktop_pet" aria-current=pet_current>
                            "Pet 事件"
                        </a>
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

#[component]
fn LoginView() -> impl IntoView {
    view! {
        <html lang="zh-Hant">
            <head>
                <meta charset="utf-8"/>
                <title>"Analytics 帳密登入"</title>
            </head>
            <body>
                <main>
                    <h1>"Analytics 帳密登入"</h1>
                    <form method="post" action="/login">
                        <div>
                            <label for="analytics-username">"帳號"</label>
                            <input id="analytics-username" name="username"
                                autocomplete="username" required/>
                        </div>
                        <div>
                            <label for="analytics-password">"密碼"</label>
                            <input id="analytics-password" name="password"
                                type="password" autocomplete="current-password" required/>
                        </div>
                        <button type="submit">"登入"</button>
                    </form>
                </main>
            </body>
        </html>
    }
}

pub fn render_login() -> String {
    format!("<!doctype html>{}", view! { <LoginView/> }.to_html())
}

#[cfg(test)]
mod day29_tests {
    use super::*;

    #[test]
    fn active_source_is_marked_for_accessibility() {
        let html = render_dashboard("Pet", 3);
        assert!(html.contains("desktop_pet"));
        assert!(html.contains("aria-current=\"page\""));
        assert!(html.contains("Pet 已提交事件總數 3"));
    }
}

#[cfg(test)]
mod day30_tests {
    #[test]
    fn login_page_is_server_rendered_with_accessible_fields() {
        let html = super::render_login();
        assert_eq!(html.matches("<!doctype html>").count(), 1);
        assert!(html.contains("action=\"/login\""));
        assert!(html.contains("name=\"username\""));
        assert!(html.contains("name=\"password\""));
        assert!(html.contains("autocomplete=\"current-password\""));
    }
}
