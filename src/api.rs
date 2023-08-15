use crate::todos::Todo;
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;
use thiserror::Error;

const URL: &str = "https://simple-api.metsysfhtagn.repl.co/api/todos";

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Unable to send request")]
    SendRequestError(#[from] reqwest::Error),
    #[error("Request failed: {0}")]
    BadRequest(&'static str),
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

pub fn get_todos(tx: Sender<Vec<Todo>>) {
    tokio::spawn(async move {
        let body: String = reqwest::get(URL)
            .await
            .expect("Failed to fetch data from server")
            .text()
            .await
            .expect("Failed to parse data to text");

        let result: ResponseTodos = serde_json::from_str(&body).unwrap_or(ResponseTodos::default());
        dbg!(&result);
        let _ = tx.send(result.todos);
    });
}

pub fn create_todo(todo: Todo, tx: Sender<Result<Todo, ApiError>>) {
    tokio::spawn(async move {
        let response = post_todo(todo).await;
        let _ = tx.send(response);
    });
}

async fn post_todo(todo: Todo) -> Result<Todo, ApiError> {
    let client = reqwest::Client::new();
    let request = client
        .request(Method::POST, URL)
        .json(&todo)
        .build()
        .map_err(|e| ApiError::SendRequestError(e))?;

    let response: ResponsePost = client
        .execute(request)
        .await?
        .json()
        .await
        .map_err(|e| ApiError::SendRequestError(e))?;

    dbg!(&response);
    match response.status.as_str() {
        "success" => return Ok(response.data.todo),
        _ => return Err(ApiError::BadRequest("Unknown error")),
    }
}
