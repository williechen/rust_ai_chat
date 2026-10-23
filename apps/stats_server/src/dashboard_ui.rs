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

#[component]
fn ChangePasswordView(restricted: bool, csrf: String) -> impl IntoView {
    let heading = if restricted {
        "首次登入須更換密碼與Email"
    } else {
        "修改密碼與Email"
    };
    view! {
        <html lang="zh-Hant">
            <head><meta charset="utf-8"/><title>"修改密碼"</title></head>
            <body><main>
                <h1>{heading}</h1>
                <form method="post" action="/change-password">
                    <input type="hidden" name="csrf" value=csrf/>
                    <p>
                        <label for="change-current">"目前密碼"</label>
                        <input id="change-current" type="password" name="current_password"
                            autocomplete="current-password" required maxlength="128"/>
                    </p>
                    <p>
                        <label for="change-new">"新密碼"</label>
                        <input id="change-new" type="password" name="new_password"
                            autocomplete="new-password" required minlength="8" maxlength="128"/>
                    </p>
                    <p>
                        <label for="change-confirm">"確認新密碼"</label>
                        <input id="change-confirm" type="password" name="confirm_password"
                            autocomplete="new-password" required minlength="8" maxlength="128"/>
                    </p>
                    <p>
                        <label for="change-email">"唯一Email"</label>
                        <input id="change-email" type="email" name="email"
                            autocomplete="email" required maxlength="254"/>
                    </p>
                    <button type="submit">"儲存並重新登入"</button>
                </form>
                <p>"成功後所有裝置都須重新登入。Email不驗證；寄信重設尚未接通。"</p>
                <form method="post" action="/logout">
                    <button type="submit">"登出"</button>
                </form>
            </main></body>
        </html>
    }
}

pub fn render_change_password(restricted: bool, csrf: String) -> String {
    format!(
        "<!doctype html>{}",
        view! {
            <ChangePasswordView restricted csrf/>
        }
        .to_html()
    )
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

#[cfg(test)]
mod day31_tests {
    #[test]
    fn change_form_keeps_wire_names_and_escapes_attribute_values() {
        let html = super::render_change_password(true, "a\" onfocus=\"bad".into());
        assert_eq!(html.matches("<!doctype html>").count(), 1);
        for name in [
            "current_password",
            "new_password",
            "confirm_password",
            "email",
            "csrf",
        ] {
            assert!(html.contains(&format!("name=\"{name}\"")));
        }
        assert!(html.contains("&quot;"));
        assert!(!html.contains("value=\"a\" onfocus="));
        assert!(html.contains("首次登入須更換密碼與Email"));
        let ordinary = super::render_change_password(false, "test-csrf".into());
        assert!(ordinary.contains("修改密碼與Email"));
        assert!(!ordinary.contains("首次登入須更換密碼與Email"));
        assert!(!ordinary.contains("lesson-only-password"));
    }
}
