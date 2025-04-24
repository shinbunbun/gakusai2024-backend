use gakusai2024_proto::api::{
    task_service_server::TaskService, CreateTaskRequest, CreateTaskResponse, GetListTasksRequest,
    GetListTasksResponse, GetTaskRequest, GetTaskResponse, Task as ProtoTask, UpdateTaskRequest,
    UpdateTaskResponse,
};
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::{
    domain::{
        repository::task::TaskRepositoryTrait,
        task::{Task, TaskExt},
    },
    usecase::task::TaskUsecaseTrait,
};

pub trait TaskHandlerTrait<TU, TR>
where
    TU: TaskUsecaseTrait<TR>,
    TR: TaskRepositoryTrait + 'static,
{
    fn new(usecase: Box<TU>) -> Self
    where
        Self: Sized;
}

pub struct TaskHandler<TU, TR>
where
    TU: TaskUsecaseTrait<TR>,
    TR: TaskRepositoryTrait + 'static,
{
    usecase: Box<TU>,
    _phantom: std::marker::PhantomData<TR>,
}

impl<TU, TR> TaskHandlerTrait<TU, TR> for TaskHandler<TU, TR>
where
    TU: TaskUsecaseTrait<TR>,
    TR: TaskRepositoryTrait,
{
    fn new(usecase: Box<TU>) -> Self {
        Self {
            usecase,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[tonic::async_trait]
impl<TU, TR> TaskService for TaskHandler<TU, TR>
where
    TU: TaskUsecaseTrait<TR> + 'static + Sync + Send,
    TR: TaskRepositoryTrait + Sync + Send + 'static,
{
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<CreateTaskResponse>, Status> {
        log::info!("Got a request: {:?}", request);

        let task = request
            .into_inner()
            .task_request
            .ok_or_else(|| Status::invalid_argument("Task is required"))?;

        let uuid = Uuid::new_v4();
        _ = self
            .usecase
            .insert(crate::domain::task::Task {
                id: uuid, // 仮
                title: task.title,
                description: task.description.unwrap_or("none".to_string()),
                due_date: time::OffsetDateTime::from_unix_timestamp(task.due_date.unwrap().seconds)
                    .map_err(|_| Status::invalid_argument("Invalid timestamp"))?, //Copilotくん
                priority: task.priority,
                weight: task.weight,
                created_at: time::OffsetDateTime::now_utc(),
                updated_at: time::OffsetDateTime::now_utc(),
                user_id: task.user_id,
            })
            .await?;

        Ok(Response::new(CreateTaskResponse {
            task_id: uuid.to_string(),
        }))
    }

    async fn get_task(
        &self,
        request: Request<GetTaskRequest>,
    ) -> Result<Response<GetTaskResponse>, Status> {
        log::info!("Got a request: {:?}", request);

        let id = Uuid::parse_str(request.into_inner().task_id.as_str()).unwrap();

        let task = self.usecase.find(id).await?;

        Ok(Response::new(GetTaskResponse {
            task: Some(ProtoTask {
                id: task.id.to_string(),
                title: task.title,
                description: Some(task.description),
                due_date: Some(prost_types::Timestamp {
                    seconds: task.due_date.unix_timestamp(),
                    nanos: task.due_date.nanosecond() as i32,
                }),
                priority: task.priority,
                weight: task.weight,
                created_at: Some(prost_types::Timestamp {
                    seconds: task.created_at.unix_timestamp(),
                    nanos: task.created_at.nanosecond() as i32,
                }),
                updated_at: Some(prost_types::Timestamp {
                    seconds: task.updated_at.unix_timestamp(),
                    nanos: task.updated_at.nanosecond() as i32,
                }),
                user_id: task.user_id,
            }),
        }))
    }

    async fn get_list_tasks(
        &self,
        request: Request<GetListTasksRequest>,
    ) -> Result<Response<GetListTasksResponse>, Status> {
        log::info!("Got a request: {:?}", request);

        let user_id = request.into_inner().user_id;

        let tasks = self.usecase.find_from_user_id(user_id).await?;

        Ok(Response::new(GetListTasksResponse {
            tasks: tasks
                .iter()
                .map(|t| ProtoTask {
                    id: t.id.to_string(),
                    title: t.title.clone(),
                    description: Some(t.description.clone()),
                    due_date: Some(prost_types::Timestamp {
                        seconds: t.due_date.unix_timestamp(),
                        nanos: t.due_date.nanosecond() as i32,
                    }),
                    priority: t.priority,
                    weight: t.weight,
                    created_at: Some(prost_types::Timestamp {
                        seconds: t.created_at.unix_timestamp(),
                        nanos: t.created_at.nanosecond() as i32,
                    }),
                    updated_at: Some(prost_types::Timestamp {
                        seconds: t.updated_at.unix_timestamp(),
                        nanos: t.updated_at.nanosecond() as i32,
                    }),
                    user_id: t.user_id.clone(),
                })
                .collect(),
        }))
    }

    async fn update_task(
        &self,
        request: Request<UpdateTaskRequest>,
    ) -> Result<Response<UpdateTaskResponse>, Status> {
        log::info!("Got a request: {:?}", request);

        let inner_request = request.into_inner();

        let task_request = inner_request
            .task_update
            .ok_or_else(|| Status::invalid_argument("Task is required"))?;

        // タスクIDをパース
        let uuid = Uuid::parse_str(inner_request.task_id.as_str())
            .map_err(|_| Status::invalid_argument("Invalid task ID"))?;

        // 既存のタスクを取得
        let existing_task = self.usecase.find(uuid).await?;

        // ProtoTaskRequest -> ドメイン Task 変換
        let due_date = if let Some(ts) = task_request.due_date {
            Some(
                time::OffsetDateTime::from_unix_timestamp(ts.seconds)
                    .map_err(|_| Status::invalid_argument("Invalid timestamp"))?,
            )
        } else {
            None
        };
        let updated_task = existing_task.update(
            task_request.title,
            task_request.description,
            task_request.user_id,
            due_date,
            task_request.priority,
            task_request.weight,
        );

        // 更新処理
        self.usecase.update(updated_task).await?;

        Ok(Response::new(UpdateTaskResponse {
            task_id: uuid.to_string(),
        }))
    }
}
