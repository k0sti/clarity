use dioxus::prelude::*;

fn main() {
    dioxus::launch(app);
}

#[component]
fn app() -> Element {
    let mut message = use_signal(|| String::new());
    let mut messages = use_signal(Vec::<(String, String)>::new);

    rsx! {
        style { {include_str!("../assets/style.css")} }
        div {
            class: "container",
            div {
                class: "header",
                h1 { "AI Assistant" }
            }
            div {
                class: "messages",
                for (role, msg) in messages.read().iter() {
                    div {
                        class: "message {role}",
                        p { "{msg}" }
                    }
                }
            }
            div {
                class: "input-area",
                input {
                    r#type: "text",
                    placeholder: "Type your message...",
                    value: "{message}",
                    oninput: move |evt| message.set(evt.value().clone()),
                    onkeypress: move |evt| {
                        if evt.key() == Key::Enter && !message().is_empty() {
                            messages.write().push(("user".to_string(), message().clone()));
                            // TODO: Connect to MCP server to get AI response
                            message.set(String::new());
                        }
                    }
                }
                button {
                    onclick: move |_| {
                        if !message().is_empty() {
                            messages.write().push(("user".to_string(), message().clone()));
                            // TODO: Connect to MCP server to get AI response
                            message.set(String::new());
                        }
                    },
                    "Send"
                }
            }
        }
    }
}
