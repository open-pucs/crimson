mod local_store;

use crate::types::{DocStatus, TaskID};

pub async fn ingest_file_to_queue(status: DocStatus) {
    todo!("Implement Ingest to store location.")
}

pub async fn update_task_data(status: DocStatus) {
    todo!()
}

pub async fn get_file_task_from_queue() -> DocStatus {
    todo!("Implement popping a task.")
}

pub async fn get_task_data_from_id(id: TaskID) -> DocStatus {
    todo!()
}
