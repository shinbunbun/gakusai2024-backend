use std::sync::Arc;

use dotenv::dotenv;
use gakusai2024_proto::api::{
    hello_service_client::HelloServiceClient, hello_service_server::HelloServiceServer,
    CreateHelloRequest,
};
use hyper_util::rt::TokioIo;
use sea_orm::Database;
use tokio::sync::Mutex;
use tonic::transport::{Endpoint, Server, Uri};
use tower::service_fn;
use uuid::Uuid;

use gakusai2024_backend::{
    domain::repository::hello::HelloRepositoryTrait,
    infrastructure,
    interface::{self, handler::hello::HelloHandlerTrait},
    usecase::{self, hello::HelloUsecaseTrait},
};

#[tokio::test]
async fn test_hello() {
    dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let (client, server) = tokio::io::duplex(1024);

    let db = Database::connect(db_url).await.unwrap();
    let hello_persistence =
        infrastructure::db::hello::HelloPersistence::new(Arc::new(Mutex::new(db)));
    let hello_usecase = usecase::hello::HelloUsecase::new(Box::new(hello_persistence));
    let hello_handler = interface::handler::hello::HelloHandler::new(Box::new(hello_usecase));

    tokio::spawn(async move {
        Server::builder()
            .add_service(HelloServiceServer::new(hello_handler))
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

    let mut client = HelloServiceClient::new(channel);

    let name = Uuid::new_v4().to_string();
    let message = Uuid::new_v4().to_string();

    let request = tonic::Request::new(CreateHelloRequest {
        hello: Some(gakusai2024_proto::api::Hello {
            name: name.clone(),
            message: message.clone(),
        }),
    });

    let create_hello_response = client.create_hello(request).await.unwrap();

    println!("RESPONSE={:?}", create_hello_response);

    let read_hello_response = client
        .read_hello(gakusai2024_proto::api::ReadHelloRequest { name: name.clone() })
        .await
        .unwrap();

    println!("RESPONSE={:?}", read_hello_response);

    assert_eq!(
        read_hello_response.get_ref().hello.as_ref().unwrap().name,
        name
    );
    assert_eq!(
        read_hello_response
            .get_ref()
            .hello
            .as_ref()
            .unwrap()
            .message,
        message
    );
}
