//! Task runner endpoints (`/api/tasks`, `/api/tasks/run`, `/api/tasks/status`).

use serde_json::Value;

use super::Endpoint;

/// `GET /api/tasks` — list runnable tasks grouped by type.
#[derive(Debug, Default, Clone)]
pub struct ListTasks;

impl Endpoint for ListTasks {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/tasks".into()
    }
}

/// `POST /api/tasks/run` — run all tasks.
#[derive(Debug, Default, Clone)]
pub struct RunAllTasks;

impl Endpoint for RunAllTasks {
    type Output = Value;

    fn method(&self) -> &'static str {
        "POST"
    }

    fn path(&self) -> String {
        "/api/tasks/run".into()
    }
}

/// `GET /api/tasks/status` — queue snapshot.
#[derive(Debug, Default, Clone)]
pub struct GetTasksStatus;

impl Endpoint for GetTasksStatus {
    type Output = Value;

    fn method(&self) -> &'static str {
        "GET"
    }

    fn path(&self) -> String {
        "/api/tasks/status".into()
    }
}
