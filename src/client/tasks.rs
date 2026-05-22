use serde_json::Value;

use super::RommClient;

impl RommClient {
    /// Triggers a server-side task by name (e.g., `"scan_library"`).
    pub async fn run_task(&self, task_name: &str, kwargs: Option<Value>) -> anyhow::Result<Value> {
        let path = format!("/api/tasks/run/{}", task_name);
        self.request_json("POST", &path, &[], kwargs).await
    }

    /// Polls the status of a running task by its ID.
    pub async fn get_task_status(&self, task_id: &str) -> anyhow::Result<Value> {
        let path = format!("/api/tasks/{}", task_id);
        self.request_json("GET", &path, &[], None).await
    }

    /// Enqueues all runnable tasks on the server.
    pub async fn run_all_tasks(&self) -> anyhow::Result<Value> {
        self.request_json("POST", "/api/tasks/run", &[], None).await
    }

    /// Lists all recent and active tasks.
    pub async fn list_tasks(&self) -> anyhow::Result<Value> {
        self.request_json("GET", "/api/tasks", &[], None).await
    }

    /// Returns the current status of the task queue.
    pub async fn get_tasks_queue_status(&self) -> anyhow::Result<Value> {
        self.request_json("GET", "/api/tasks/status", &[], None)
            .await
    }
}
