#![allow(non_snake_case)]

use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
struct Message {
    timestamp: String, // You could also use a proper date-time type
    author: String,
    content: String,
}

struct AppState {
    messages: Vec<Message>,
}

fn main() {
    dioxus_web::launch(App)
}

fn App(cx: Scope) -> Element {
    cx.render(rsx! {
        div {
            class: "relative min-h-screen min-w-full justify-center overflow-hidden bg-gray-50 dark:bg-zinc-950 py-10 sm:py-10",
            img {
                class: "absolute top-0 left-1/2 transform -translate-x-1/2 w-20 h-20 shadow-lg",
                src: "https://i.imgur.com/J2iCsaZ.png"
            },
            div {
                class: "flex flex-col p-1 space-y-1 max-w-2xl mx-auto rounded-md bg-white dark:bg-zinc-900 rounded-md shadow-xl p-5 dark:border-lime-500",
                div {
                    class: "text-lg font-semibold text-gray-900 dark:text-white mb-4",
                    "// CHATq :: MESSAGE LOG"
                }
                MessageBox {
                    target: "mauri".to_string(),
                }
            }
        }
    })
}

#[derive(PartialEq, Props, Clone)]
struct MessageBarProps {
    author: String,
    content: String,
}

#[derive(PartialEq, Props, Clone)]
struct MessageBoxProps {
    target: String,
}

async fn fetch_messages() -> Vec<MessageBarProps> {
    let mut res = Vec::new();

    for i in (0..25) {
        res.push(MessageBarProps {
            author: "user1".to_string(),
            content: "Hi there!".to_string(),
        });
        res.push(MessageBarProps {
            author: "user2".to_string(),
            content: "Hello there!".to_string(),
        })
    }

    res
}

#[component]
fn MessageBox(cx: Scope<MessageBoxProps>) -> Element {
    let messages = use_future(&cx, (), |_| {
        async {
            fetch_messages().await
        }
    });
    cx.render(rsx! {
        div {
            class: "bg-gray-50 min-w-fit text-gray-800 dark:bg-zinc-900 font-mono text-sm p-2 rounded-lg dark:text-zinc-200",

            div {
                "Messages received by [{cx.props.target}]"
            }

            match messages.value() {
                Some(mesgs) => rsx! {
                    mesgs.iter().map(|msg| rsx!(
                        div {
                            class: "space-5",
                            MessageBar {
                                author: msg.author.clone(),
                                content: msg.content.clone()
                            }
                        }
                    ))
                },
                None => rsx! {
                    "Loading"
                }
            }
        }
    })
}

#[component]
fn MessageBar(cx: Scope<MessageBarProps>) -> Element {
    cx.render(rsx! {
        div {
            class: "bg-gray-50 text-gray-700 dark:bg-zinc-800 dark:md:hover:bg-fuchsia-600 font-mono text-sm p-3 rounded-lg dark:text-zinc-200 mb-4",
            div {
                class: "flex justify-between space-x-20 dark:text-white",
                span {
                    class: "text-sm font-bold font-mono text-gray-800 dark:text-white",
                    "@{cx.props.author}"
                }
                span {
                    class: "text-sm text-gray-500 dark:text-white",
                    "20:20 @DTM"
                }
            }
            "{cx.props.content}"
        }
    })
}


