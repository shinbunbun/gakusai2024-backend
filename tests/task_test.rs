use std::sync::Arc;

use dotenv::dotenv;
use entity::user;
use gakusai2024_proto::api::{
    task_service_client::TaskServiceClient, task_service_server::TaskServiceServer,
    CreateTaskRequest, GetListTasksRequest, GetTaskRequest, TaskRequest, TaskUpdate,
    UpdateTaskRequest,
};
use hyper_util::rt::TokioIo;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, Database, EntityTrait, QueryFilter, Set,
    Statement,
};
use tokio::sync::Mutex;
use tonic::transport::{Endpoint, Server, Uri};
use tower::service_fn;
use uuid::Uuid;

use gakusai2024_backend::{
    domain::repository::task::TaskRepositoryTrait,
    infrastructure,
    interface::{self, handler::task::TaskHandlerTrait},
    usecase::{self, task::TaskUsecaseTrait},
};

//#[ignore]
#[tokio::test]
async fn test_task() {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let (client, server) = tokio::io::duplex(1024);

    let db_for_cleanup = Database::connect(db_url.clone()).await.unwrap();
    let db = Database::connect(db_url).await.unwrap();

    // テスト用のユーザーIDを生成
    let test_user_id = format!("test_user_{}", Uuid::new_v4());

    // テスト前にデータベースをクリーンアップ
    let cleanup_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"DELETE FROM tasks WHERE user_id = $1"#,
        vec![test_user_id.clone().into()],
    );
    db_for_cleanup.execute(cleanup_stmt).await.unwrap();

    let cleanup_user_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"DELETE FROM users WHERE user_id = $1"#,
        vec![test_user_id.clone().into()],
    );
    db_for_cleanup.execute(cleanup_user_stmt).await.unwrap();

    // テスト用のユーザーを作成
    let user = user::ActiveModel {
        id: Set(test_user_id.clone()),
        username: Set("Test User".to_string()),
        email: Set("test@example.com".to_string()),
        ..Default::default()
    };
    user.insert(&db).await.unwrap();

    let task_persistence = infrastructure::db::task::TaskPersistence::new(Arc::new(Mutex::new(db)));
    let task_usecase = usecase::task::TaskUsecase::new(Box::new(task_persistence));
    let task_handler = interface::handler::task::TaskHandler::new(Box::new(task_usecase));

    tokio::spawn(async move {
        Server::builder()
            .add_service(TaskServiceServer::new(task_handler))
            .serve_with_incoming(tokio_stream::once(Ok::<_, std::io::Error>(server)))
            .await
    });

    // Move client to an option so we can _move_ the inner value
    // on the first attempt to connect. All other attempts will fail.
    let mut client = Some(client);
    let channel = Endpoint::try_from("http://[::]:50051")
        .unwrap()
        .connect_with_connector(service_fn(move |_: Uri| {
            let client = client.take();

            async move {
                if let Some(client) = client {
                    Ok(TokioIo::new(client))
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Client already taken",
                    ))
                }
            }
        }))
        .await
        .unwrap();

    let mut client = TaskServiceClient::new(channel);

    // CreateTaskとGetTaskのためのTaskRequest
    let task_request = TaskRequest {
        title: "test_title".to_string(),
        description: Some("test_description".to_string()),
        due_date: Some(prost_types::Timestamp::default()),
        priority: 1,
        weight: 1,
        user_id: test_user_id.clone(),
    };

    let request = tonic::Request::new(CreateTaskRequest {
        task_request: Some(task_request.clone()),
    });

    let create_task_response = client.create_task(request).await.unwrap();

    println!("RESPONSE={:?}", create_task_response);

    let task_id = create_task_response.get_ref().task_id.clone();

    let get_task_response = client.get_task(GetTaskRequest { task_id }).await.unwrap();

    println!("RESPONSE={:?}", get_task_response);

    // GetListTasksのためのTaskRequest
    let one_of_tasks = TaskRequest {
        title: "test_title2".to_string(),
        description: Some("test_description".to_string()),
        due_date: Some(prost_types::Timestamp::default()),
        priority: 1,
        weight: 1,
        user_id: test_user_id.clone(),
    };

    let requests = vec![
        tonic::Request::new(CreateTaskRequest {
            task_request: Some(one_of_tasks.clone()),
        }),
        tonic::Request::new(CreateTaskRequest {
            task_request: Some(TaskRequest {
                title: "test_title3".to_string(),
                description: Some("test_description".to_string()),
                due_date: Some(prost_types::Timestamp::default()),
                priority: 1,
                weight: 1,
                user_id: test_user_id.clone(),
            }),
        }),
        tonic::Request::new(CreateTaskRequest {
            task_request: Some(TaskRequest {
                title: "test_title4".to_string(),
                description: Some("test_description".to_string()),
                due_date: Some(prost_types::Timestamp::default()),
                priority: 1,
                weight: 1,
                user_id: test_user_id.clone(),
            }),
        }),
    ];

    for r in requests {
        let res = client.create_task(r).await.unwrap();
        println!("RESPONSE={:?}", res);
    }

    let get_list_tasks_response = client
        .get_list_tasks(GetListTasksRequest {
            user_id: test_user_id.clone(),
        })
        .await
        .unwrap();

    // get_taskのassert
    assert_eq!(
        get_task_response.get_ref().task.as_ref().unwrap().title,
        task_request.title
    );
    assert_eq!(
        get_task_response
            .get_ref()
            .task
            .as_ref()
            .unwrap()
            .description,
        task_request.description
    );
    assert_eq!(
        get_task_response.get_ref().task.as_ref().unwrap().due_date,
        task_request.due_date
    );
    assert_eq!(
        get_task_response.get_ref().task.as_ref().unwrap().priority,
        task_request.priority
    );
    assert_eq!(
        get_task_response.get_ref().task.as_ref().unwrap().weight,
        task_request.weight
    );
    assert_eq!(
        get_task_response.get_ref().task.as_ref().unwrap().user_id,
        task_request.user_id
    );

    // get_list_tasksのassert
    let tasks = get_list_tasks_response.get_ref().tasks.clone();
    assert_eq!(tasks.len(), 4); // 合計4つのタスクが作成されているはず

    // 特定のタスクが含まれていることを確認
    let found_task = tasks
        .iter()
        .find(|task| task.title == one_of_tasks.title)
        .expect("Task not found");
    assert_eq!(found_task.description, one_of_tasks.description);
    assert_eq!(found_task.due_date, one_of_tasks.due_date);
    assert_eq!(found_task.priority, one_of_tasks.priority);
    assert_eq!(found_task.weight, one_of_tasks.weight);
    assert_eq!(found_task.user_id, one_of_tasks.user_id);

    // UpdateTaskのテスト
    let update_task_request = TaskUpdate {
        title: Some("updated_title".to_string()),
        description: Some("updated_description".to_string()),
        due_date: Some(prost_types::Timestamp::default()),
        priority: Some(2),
        weight: Some(2),
        user_id: Some(test_user_id.clone()),
    };

    let update_request = tonic::Request::new(UpdateTaskRequest {
        task_id: create_task_response.get_ref().task_id.clone(),
        task_update: Some(update_task_request.clone()),
    });

    let update_task_response = client.update_task(update_request).await.unwrap();
    println!("UPDATE RESPONSE={:?}", update_task_response);

    // 更新後のタスクを取得して検証
    let updated_task_response = client
        .get_task(GetTaskRequest {
            task_id: create_task_response.get_ref().task_id.clone(),
        })
        .await
        .unwrap();

    // update_taskのassert
    assert_eq!(
        updated_task_response.get_ref().task.as_ref().unwrap().title,
        update_task_request.title.unwrap()
    );
    assert_eq!(
        updated_task_response
            .get_ref()
            .task
            .as_ref()
            .unwrap()
            .description,
        update_task_request.description
    );
    assert_eq!(
        updated_task_response
            .get_ref()
            .task
            .as_ref()
            .unwrap()
            .due_date,
        update_task_request.due_date
    );
    assert_eq!(
        updated_task_response
            .get_ref()
            .task
            .as_ref()
            .unwrap()
            .priority,
        update_task_request.priority.unwrap()
    );
    assert_eq!(
        updated_task_response
            .get_ref()
            .task
            .as_ref()
            .unwrap()
            .weight,
        update_task_request.weight.unwrap()
    );
    assert_eq!(
        updated_task_response
            .get_ref()
            .task
            .as_ref()
            .unwrap()
            .user_id,
        update_task_request.user_id.unwrap()
    );

    // テスト後にデータベースをクリーンアップ
    let cleanup_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"DELETE FROM tasks WHERE user_id = $1"#,
        vec![test_user_id.clone().into()],
    );
    db_for_cleanup.execute(cleanup_stmt).await.unwrap();

    let cleanup_user_stmt = Statement::from_sql_and_values(
        sea_orm::DatabaseBackend::Postgres,
        r#"DELETE FROM users WHERE user_id = $1"#,
        vec![test_user_id.clone().into()],
    );
    db_for_cleanup.execute(cleanup_user_stmt).await.unwrap();
}
