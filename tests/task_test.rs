use std::sync::Arc;

use dotenv::dotenv;
use gakusai2024_proto::api::{
    task_service_client::TaskServiceClient, task_service_server::TaskServiceServer, CreateTaskRequest, GetListTasksRequest, GetTaskRequest, TaskRequest
};
use hyper_util::rt::TokioIo;
use sea_orm::Database;
use tokio::sync::Mutex;
use tonic::transport::{Endpoint, Server, Uri};
use tower::service_fn;

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

    let db = Database::connect(db_url).await.unwrap();
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

    let title = "test_title".to_string();
    let user_id = "harukun".to_string();

    let request = tonic::Request::new(CreateTaskRequest {
        task_request: Some(TaskRequest {
            title: title.clone(),
            description: Some("test_description".to_string()),
            due_date: Some(prost_types::Timestamp::default()),
            priority: 1,
            weight: 1,
            user_id: user_id.clone(),
        }),
    });

    let create_task_response = client.create_task(request).await.unwrap();

    println!("RESPONSE={:?}", create_task_response);

    let task_id = create_task_response.get_ref().task_id.clone();

    let get_task_response = client.get_task(GetTaskRequest { task_id }).await.unwrap();

    println!("RESPONSE={:?}", get_task_response);

    let requests = vec![
        tonic::Request::new(CreateTaskRequest {
            task_request: Some(TaskRequest {
                title: "test_title2".to_string(),
                description: Some("test_description".to_string()),
                due_date: Some(prost_types::Timestamp::default()),
                priority: 1,
                weight: 1,
                user_id: user_id.clone(),
            }),
        }),
        tonic::Request::new(CreateTaskRequest {
            task_request: Some(TaskRequest {
                title: "test_title3".to_string(),
                description: Some("test_description".to_string()),
                due_date: Some(prost_types::Timestamp::default()),
                priority: 1,
                weight: 1,
                user_id: "bunbun".to_string(),
            }),
        }),
    ];

    for r in requests {
        let res = client.create_task(r).await.unwrap();
        println!("RESPONSE={:?}", res);
    }

    let get_list_tasks_response = client.get_list_tasks(GetListTasksRequest{ user_id: user_id.clone() }).await.unwrap();

    assert_eq!(
        get_task_response.get_ref().task.as_ref().unwrap().title,
        title
    );
    assert_eq!(
        get_task_response.get_ref().task.as_ref().unwrap().user_id,
        user_id
    );
    assert_eq!(
        get_list_tasks_response.get_ref().tasks[0].title,
        "test_title".to_string()
    );
}
