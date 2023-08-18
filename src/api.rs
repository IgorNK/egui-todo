use crate::todos::Todo;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use thiserror::Error;

const URL: &str = "https://simple-api.metsysfhtagn.repl.co/api/todos";

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Unable to send request")]
    SendRequestError(#[from] reqwest::Error),
    #[cfg(target_arch = "wasm32")]
    #[error("Unable to send web request")]
    WebRequestError(#[from] reqwasm::Error),
    #[error("Request failed: {0}")]
    BadRequest(&'static str),
}

pub enum ResponseData {
    GetResponse(Result<Vec<Todo>, ApiError>),
    PostResponse(Result<Todo, ApiError>),
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct ResponseTodos {
    status: String,
    results: u32,
    todos: Vec<Todo>,
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct ResponsePost {
    status: String,
    data: TodoData,
}

#[derive(Deserialize, Serialize, Default, Debug)]
struct TodoData {
    todo: Todo,
}

// Native

#[cfg(not(target_arch = "wasm32"))]
pub fn get_todos(tx: Sender<ResponseData>) {
    tokio::spawn(async move {
        let body: String = reqwest::get(URL)
            .await
            .expect("Failed to fetch data from server")
            .text()
            .await
            .expect("Failed to parse data to text");

        let result: ResponseTodos = serde_json::from_str(&body).unwrap_or(ResponseTodos::default());
        dbg!(&result);
        let _ = tx.send(ResponseData::GetResponse(Ok(result.todos)));
    });
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_todo(todo: Todo, tx: Sender<ResponseData>) {
    tokio::spawn(async move {
        let response = post_todo(todo).await;
        let _ = tx.send(ResponseData::PostResponse(response));
    });
}

#[cfg(not(target_arch = "wasm32"))]
async fn post_todo(todo: Todo) -> Result<Todo, ApiError> {
    let client = reqwest::Client::new();
    let request = client
        .request(Method::POST, URL)
        .json(&todo)
        .build()
        .map_err(ApiError::SendRequestError)?;

    let response: ResponsePost = client
        .execute(request)
        .await?
        .json()
        .await
        .map_err(ApiError::SendRequestError)?;

    dbg!(&response);
    match response.status.as_str() {
        "success" => Ok(response.data.todo),
        _ => Err(ApiError::BadRequest("Unknown error")),
    }
}

// WebAssembly

#[cfg(target_arch = "wasm32")]
pub fn get_todos_web(tx: Sender<ResponseData>) {
    wasm_bindgen_futures::spawn_local(async move {
        fetch_todos_web(tx).await;
    });
}

#[cfg(target_arch = "wasm32")]
pub async fn fetch_todos_web(tx: Sender<ResponseData>) {
    log::warn!("We're in async function! Yay!");  
    let req = reqwasm::http::Request::get(URL)
      .mode(reqwasm::http::RequestMode::Cors);
    let res = req.send().await.expect("Failed to send a request");
    let response: ResponseTodos = res.json().await.expect("Failed to parse json");
    dbg!(&response);
    let _ = tx.send(ResponseData::GetResponse(Ok(response.todos)));
}


#[cfg(target_arch = "wasm32")]
pub fn create_todo_web(todo: Todo, tx: Sender<ResponseData>) {
    wasm_bindgen_futures::spawn_local(async move {
        let response = post_todo_web(todo).await;
        let _ = tx.send(ResponseData::PostResponse(response));
    });
}

#[cfg(target_arch = "wasm32")]
async fn post_todo_web(todo: Todo) -> Result<Todo, ApiError> {
    let body = serde_json::to_string(&todo).unwrap_or(String::new());
    let request = reqwasm::http::Request::post(URL)
      .body(body)
      .mode(reqwasm::http::RequestMode::Cors);
    let response: ResponsePost = request
        .send()
        .await?
        .json()
        .await
        .map_err(ApiError::WebRequestError)?;
    dbg!(&response);
    match response.status.as_str() {
        "success" => Ok(response.data.todo),
        _ => Err(ApiError::BadRequest("Unknown error")),
    }
}
