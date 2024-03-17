#![allow(non_snake_case)]

use chatq_types::data::Snapshot;
use dioxus::prelude::*;
use chatq_types::data::message::Message;
use log::log;

async fn fetch_snapshot() -> anyhow::Result<Snapshot> {
    let url = "http://localhost:3030".to_string();

    let url = format!("{}/fetch-snapshot/1", url);
    let res = reqwest::get(&url).await?.json::<Snapshot>().await?;

    log::info!("{:?}", res);

    Ok(res)
}

fn main() {

    wasm_logger::init(wasm_logger::Config::default());
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
    context: String,
}

impl From<&Message> for MessageBarProps {
    fn from(value: &Message) -> Self {
        Self {
            author: value.source.player().to_string(),
            content: value.content.to_string(),
            context: value.context.to_string(),
        }
    }
}

#[derive(PartialEq, Props, Clone)]
struct MessageBoxProps {
    target: String,
}

#[derive(PartialEq, Props, Clone)]
struct FilterBoxProps {}

#[component]
fn MessageBox(cx: Scope<MessageBoxProps>) -> Element {
    let messages = use_future(&cx, (), |_| async {
        fetch_snapshot().await.unwrap().messages
    });
    cx.render(rsx! {
        div {
            class: "bg-gray-50 min-w-fit p-2 text-gray-800 dark:bg-zinc-900 font-mono text-sm p-2 rounded-lg dark:text-zinc-200",

            div {
                "Messages received by user [ @{cx.props.target} ]"
            },

            div {
                class: "grid grid-cols-3 gap-2",
            }

            match messages.value() {
                Some(mesgs) => rsx! {

                    mesgs.iter().map(|msg| {
                        let props = MessageBarProps::from(msg);
                        rsx!(
                        div {
                            class: "space-5",
                                MessageBar {
                                    author: props.author,
                                    content: props.content,
                                    context: props.context
                                }
                        }
                    )})
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
            class: "bg-gray-50 text-gray-700 dark:bg-zinc-800 font-mono text-sm p-3 rounded-lg dark:text-zinc-200 mb-4",
            div {
                class: "flex justify-between space-x-20 dark:text-white",
                button {
                    class: "text-sm font-bold dark:md:hover:underline font-mono text-gray-800 dark:text-white",
                    "@{cx.props.author}"
                }
                div {
                    class: "flex flex-row space-x-2 justify-between",
                    button {
                        class: "text-sm text-gray-500 dark:text-white bg-zinc-600",
                        "20:20"
                    }
                    button {
                        class: "text-sm dark:md:hover:bg-lime-700 text-gray-500 dark:text-white",
                        "@DTM"
                    }
                }
            }
            "{cx.props.content}"
        }
    })
}
