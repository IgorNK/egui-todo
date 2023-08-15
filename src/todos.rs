use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Todo {
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    pub completed: Option<bool>,
    pub createdAt: Option<DateTime<Utc>>,
    pub updatedAt: Option<DateTime<Utc>>,
}

impl Todo {
    pub fn new(title: &str, content: &str) -> Self {
        Self {
            id: None,
            title: title.to_string(),
            content: content.to_string(),
            completed: None,
            createdAt: None,
            updatedAt: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TodoList {
    pub todos: Vec<Todo>,
}
