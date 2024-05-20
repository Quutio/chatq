#![allow(non_snake_case)]

use std::collections::{HashSet};
use chatq_types::data::Snapshot;
use dioxus::prelude::*;
use chatq_types::data::message::Message;
use reqwest::Client;
use serde::{Deserialize, Serialize};

static COLOR_POOL: &[&str] = &[
    "amber-800",
    "yellow-800",
    "lime-800",
    "green-800",
    "emerald-800",
    "teal-800",
    "cyan-800",
    "sky-800",
    "blue-800",
    "indigo-800",
    "violet-800",
    "purple-800",
    "fuchsia-800",
];

async fn fetch_snapshot() -> anyhow::Result<Snapshot> {
    let url = "http://localhost:3030".to_string();

    let url = format!("{}/fetch-snapshot/6fe0bbb0-96b4-4e5d-ac37-f516f3dd220e", url);
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
                MessageBox {}
            }
        }
    })
}

#[derive(PartialEq, Props, Clone)]
struct MessageBarProps {
    author: String,
    content: String,
    context: String,
    timestamp: String,
}

impl From<&Message> for MessageBarProps {
    fn from(value: &Message) -> Self {
        Self {
            author: value.source.player().to_string(),
            content: value.content.to_string(),
            context: value.context.to_string(),
            timestamp: value.timestamp.time().to_string(),
        }
    }
}

#[derive(PartialEq, Props, Clone)]
struct FilterBoxProps {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum HighlightType {
    SelectedUser,
}

#[derive(Serialize)]
struct GraphqlQuery {
    query: String,
    variables: Variables,
}

#[derive(Serialize)]
struct Variables {
    uuids: Vec<String>,
}

#[derive(Deserialize)]
struct User {
    uniqueId: String,
    username: String,
}

#[derive(Deserialize)]
struct UsersResponse {
    users: Vec<User>,
}

#[derive(Deserialize)]
struct GraphqlResponse<T> {
    data: T,
}

async fn uuid_to_username(uuids: Vec<String>) -> anyhow::Result<UsersResponse> {

    let client = Client::new();

    let query = GraphqlQuery {
        query: String::from(r#"
            query GetUsers($uuids: [String!]!) {
                users(uuids: $uuids) {
                    uniqueId
                    username
                }
            }
        "#),
        variables: Variables { uuids },
    };

    let resp: GraphqlResponse<UsersResponse> = client
        .post("https://api.quut.io/graphql")
        .json(&query)
        .send()
        .await?
        .json().await?;

    Ok(resp.data)
}

#[component]
fn MessageBox(cx: Scope) -> Element {
    let messages = use_future(&cx, (), |_| async {
        //let mut map = HashMap::new();
        let messages = fetch_snapshot().await.unwrap();

        //let uuids: Vec<String> = messages.messages.iter().map(|op| op.source.player().to_string()).collect();

        //let users = uuid_to_username(uuids).await.unwrap().users;

        //for user in users {
        //    map.insert(user.uniqueId, user.username);
        //}

        messages
    });

    use_shared_state_provider(cx, || {
        HashSet::<String>::new()
    });

    cx.render(rsx! {
        div {
            class: "bg-gray-50 min-w-fit p-2 text-gray-800 dark:bg-zinc-900 font-mono text-sm p-2 rounded-lg dark:text-zinc-200",

            match messages.value() {
                Some(snap) => {

                    let taken = snap.taken.format("%Y-%m-%d %H:%M:%S").to_string();
                    let target = snap.target.to_string();

                    rsx! {
                        div {
                            class: "pb-4",
                            h1 {
                                "Messages received by [ @{target} ]"
                            },
                            "Snapshot taken at {taken}"
                        },

                        div {
                            class: "grid grid-cols-3 gap-2",
                        }

                        snap.messages.iter().rev().map(|msg| {
                            let props = MessageBarProps::from(msg);
                            rsx!(
                            div {
                                class: "space-2",
                                    MessageBar {
                                        author: props.author,
                                        content: props.content,
                                        context: props.context,
                                        timestamp: props.timestamp,
                                    }
                            }
                        )})
                    }
                },
                None => rsx! {
                    "Loading..."
                }
            }
        }
    })
}

#[component]
fn MessageBar(cx: Scope<MessageBarProps>) -> Element {
    let highlighted_users = use_shared_state::<HashSet<String>>(cx).unwrap();

    let is_highlighted = highlighted_users.read().contains(&cx.props.author);

    let border_class = if is_highlighted {
        "border-l-4 border-lime-800 hover:scale-105"
    } else {
        "hover:border-l-4 border-0 hover:border-l4 dark:border-zinc-700"
    };

    let bg_class = if is_highlighted {
        "dark:bg-zinc-950"
    } else {
        "dark:bg-zinc-800"
    };

    cx.render(rsx! {
        div {
            class: "transition duration-150 ease-in-out bg-gray-50 text-gray-700 font-mono text-sm p-3 rounded-lg dark:text-zinc-200 mb-4 {border_class} {bg_class}",
            div {
                class: "flex justify-between space-x-20 dark:text-white",
                button {
                    title: "highlight user's messages",
                    onclick: move |_| {
                        if is_highlighted {
                            highlighted_users.write().remove(&cx.props.author);
                        } else {
                            highlighted_users.write().insert(cx.props.author.to_string());
                        }
                    },
                    class: "text-sm font-bold dark:md:hover:underline font-mono text-gray-800 dark:text-white",
                    "@{cx.props.author}",
                }
                div {
                    class: "flex flex-row space-x-2 justify-between",
                    button {
                        class: "text-sm dark:md:hover:bg-lime-700 text-gray-500 dark:text-white",
                        r"@[{cx.props.context}]"
                    }
                    button {
                        class: "text-sm text-gray-500 dark:text-white bg-zinc-600",
                        "{cx.props.timestamp}"
                    }
                }
            }
            "{cx.props.content}"
        }
    })
}
