# 実装ガイド

本リポジトリはレイヤードアーキテクチャで構成されています

## ディレクトリ構造

- entity
  - SeaORMのEntityが実装されています
- migration
  - SeaORMのmigrationが実装されています
- src
  - domain
    - ドメインモデルを置く
    - EntityやそのRepositoryのインターフェイスなど
    - Entityはentityディレクトリにあるため、ここではそのtype aliasを実装する
  - infrastructure
    - DBまわりの操作などを記述する
    - Repositoryの実態はここに実装する
  - interface
    - APIのHandlerなどを実装する
    - gRPCのHandlerのTraitはProtobufのリポジトリで自動生成されているため、それを実装する
  - usecase
    - アプリケーションロジックを記述する
    - ここからdomainのRepositoryを参照する

## 実装の流れ

### RPCを追加する

まず[Protobufのリポジトリ](https://github.com/shinbunbun/gakusai2024-proto)にgRPCのAPIを実装します。

以下にサンプルを示します。

```protobuf
syntax = "proto3";

package api;

service HelloService {
    rpc CreateHello (CreateHelloRequest) returns (CreateHelloResponse);
    rpc ReadHello (ReadHelloRequest) returns (ReadHelloResponse);
}

message Hello {
    string name = 1;
    string message = 2;
}

message CreateHelloRequest {
    Hello hello = 1;
}

message CreateHelloResponse {}

message ReadHelloRequest {
    string name = 1;
}

message ReadHelloResponse {
    Hello hello = 1;
}
```

- `HelloService`で`Service`を記述します
  - 今回は`CreateHello`と`ReadHello`の2つのRPCがあります
- Hello型、`CreateHello`と`ReadHello`それぞれのリクエストとレスポンスの型を記述します

次に、`build.rs`に以下の行を追記します

```rust
.compile(&["proto/api/helloworld.proto"], &["proto/"])
```

次に、`make build`をすると`src/proto`にbuild結果が出力されます

その後、GitHubにプッシュし、コミットハッシュを控えます

backendのリポジトリに移動して、`Cargo.toml`の以下の行のrevを新しいコミットハッシュに書き換えます

```toml
gakusai2024-proto = { git = "ssh://git@github.com/shinbunbun/gakusai2024-proto.git", rev = "d896b58" }
```

`make build`をすると、新しいrpcのコードが読み込まれます

### コードの実装をする

まずはentityディレクトリにあるプロジェクトにDBのスキーマを追加します

srcディレクトリに以下のようなコードを追加します

```rust
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "hello")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub name: String,
    pub message: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
```

`lib.rs`に`pub mod hello;`を追加します

`migration`ディレクトリで`sea-orm-cli migrate generate "create_hello_table"`を実行し、migrationを作成します。

次に、migration関連のファイルを編集します。

```rust
use entity::hello::{Column, Entity};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Entity)
                    .if_not_exists()
                    .col(ColumnDef::new(Column::Name).string().not_null())
                    .col(ColumnDef::new(Column::Message).string().not_null())
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Entity).to_owned())
            .await
    }
}
```

`make db-migrate`を実行してmigrationをします（この前に`docker compose up`をしてDBを起動しておく必要があります）。

次にdomain層を実装します。

Modelの実態は`entity`にあるため、ここではそのType aliasのみを実装します。

```rust
// src/domain/hello.rs

use entity::hello::Model;

pub type Hello = Model;

```

次にリポジトリの定義をします。

```rust
// src/domain/repository/hello.rs

use std::{future::Future, sync::Arc};

use mockall::automock;
use sea_orm::DatabaseConnection;

use crate::{domain::hello::Hello, error::CustomError};

#[automock]
pub trait HelloRepositoryTrait {
    fn new(conn: Arc<tokio::sync::Mutex<DatabaseConnection>>) -> Self
    where
        Self: Sized;
    fn insert(&self, hello: Hello) -> impl Future<Output = Result<String, CustomError>> + Send;
    fn find(&self, name: String) -> impl Future<Output = Result<Hello, CustomError>> + Send;
}

```

次にリポジトリの実装をします。

```rust
// src/infrastructure/db/hello.rs

use std::{ops::Deref, sync::Arc};

use entity::hello::{self, ActiveModel};
use sea_orm::{DatabaseConnection, EntityTrait, IntoSimpleExpr, QueryFilter, Set};
use tokio::sync::Mutex;

use crate::{
    domain::{hello::Hello, repository::hello::HelloRepositoryTrait},
    error::CustomError,
};

use entity::hello::Entity as HelloEntity;

use super::Repository;

pub struct HelloPersistence {
    repository: Repository,
}

impl HelloRepositoryTrait for HelloPersistence {
    fn new(conn: Arc<Mutex<DatabaseConnection>>) -> Self
    where
        Self: Sized,
    {
        Self {
            repository: Repository::new(conn),
        }
    }
    async fn insert(&self, hello: Hello) -> Result<String, CustomError> {
        let db_unlock = self.repository.get_db();
        let db_lock = db_unlock.lock().await;
        let db = db_lock.deref();
        let hello_am = ActiveModel {
            name: Set(hello.name),
            message: Set(hello.message),
        };
        let insert_result = HelloEntity::insert(hello_am).exec(db).await?;
        Ok(insert_result.last_insert_id.to_string())
    }

    async fn find(&self, name: String) -> Result<Hello, CustomError> {
        let db_unlock = self.repository.get_db();
        let db_lock = db_unlock.lock().await;
        let db = db_lock.deref();
        let result = HelloEntity::find()
            .filter(hello::Column::Name.into_simple_expr().eq(&name))
            .one(db)
            .await?;
        match result {
            Some(hello) => Ok(Hello {
                name: hello.name,
                message: hello.message,
            }),
            None => Err(CustomError::DbNotFound(format!("key: {}", &name))),
        }
    }
}

```

次にusecase層でアプリケーションロジックの実装をします。

```rust
// src/usecase/hello.rs

use std::future::Future;

use mockall::automock;

use crate::{
    domain::{hello::Hello, repository::hello::HelloRepositoryTrait},
    error::CustomError,
};

#[automock]
pub trait HelloUsecaseTrait<HR: HelloRepositoryTrait + 'static> {
    fn new(repository: Box<HR>) -> Self
    where
        Self: Sized;
    fn insert(&self, hello: Hello) -> impl Future<Output = Result<String, CustomError>> + Send;
    fn find(&self, name: String) -> impl Future<Output = Result<Hello, CustomError>> + Send;
}

pub struct HelloUsecase<HR: HelloRepositoryTrait> {
    repository: Box<HR>,
}

impl<HR: HelloRepositoryTrait + 'static> HelloUsecaseTrait<HR> for HelloUsecase<HR> {
    fn new(repository: Box<HR>) -> Self {
        Self { repository }
    }

    fn insert(&self, hello: Hello) -> impl Future<Output = Result<String, CustomError>> + Send {
        self.repository.insert(hello)
    }

    fn find(&self, name: String) -> impl Future<Output = Result<Hello, CustomError>> + Send {
        self.repository.find(name)
    }
}

#[cfg(test)]
mod tests {

    use mockall::predicate::eq;

    use super::*;
    use crate::domain::{hello::Hello, repository::hello::MockHelloRepositoryTrait};

    #[tokio::test]
    async fn test_insert() {
        let mut mock = MockHelloRepositoryTrait::default();
        mock.expect_insert()
            .returning(|_| Box::pin(async { Ok("test_name".to_string()) }));

        let usecase = HelloUsecase::new(Box::new(mock));
        let hello = Hello {
            name: "test_name".to_string(),
            message: "test_message".to_string(),
        };
        let result = usecase.insert(hello).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_name".to_string());
    }

    #[tokio::test]
    async fn test_find() {
        let mut mock = MockHelloRepositoryTrait::default();
        let except_hello = Hello {
            name: "test_name".to_string(),
            message: "test_message".to_string(),
        };
        mock.expect_find()
            .with(eq("test_name".to_string()))
            .returning(move |_| {
                Box::pin({
                    let value = except_hello.clone();
                    async move { Ok(value) }
                })
            });

        let usecase = HelloUsecase::new(Box::new(mock));
        let result = usecase.find("test_name".to_string()).await;
        assert!(result.is_ok());
    }
}

```

usecase層の関数は必ず単体テストを書いてください。

次にHandlerを実装します。

```rust
// src/interface/handler/hello.rs

use gakusai2024_proto::api::{
    hello_service_server::HelloService, CreateHelloRequest, CreateHelloResponse, Hello,
    ReadHelloRequest, ReadHelloResponse,
};
use tonic::{Request, Response, Status};

use crate::{domain::repository::hello::HelloRepositoryTrait, usecase::hello::HelloUsecaseTrait};

pub trait HelloHandlerTrait<HU, HR>
where
    HU: HelloUsecaseTrait<HR>,
    HR: HelloRepositoryTrait + 'static,
{
    fn new(usecase: Box<HU>) -> Self
    where
        Self: Sized;
}

pub struct HelloHandler<HU, HR>
where
    HU: HelloUsecaseTrait<HR>,
    HR: HelloRepositoryTrait + 'static,
{
    usecase: Box<HU>,
    _phantom: std::marker::PhantomData<HR>,
}

impl<HU, HR> HelloHandlerTrait<HU, HR> for HelloHandler<HU, HR>
where
    HU: HelloUsecaseTrait<HR>,
    HR: HelloRepositoryTrait,
{
    fn new(usecase: Box<HU>) -> Self {
        Self {
            usecase,
            _phantom: std::marker::PhantomData,
        }
    }
}

#[tonic::async_trait]
impl<HU, HR> HelloService for HelloHandler<HU, HR>
where
    HU: HelloUsecaseTrait<HR> + 'static + Sync + Send,
    HR: HelloRepositoryTrait + Sync + Send + 'static,
{
    async fn create_hello(
        &self,
        request: Request<CreateHelloRequest>,
    ) -> Result<Response<CreateHelloResponse>, Status> {
        log::info!("Got a request: {:?}", request);

        let hello = request
            .into_inner()
            .hello
            .ok_or_else(|| Status::invalid_argument("Hello is required"))?;

        _ = self
            .usecase
            .insert(crate::domain::hello::Hello {
                name: hello.name,
                message: hello.message,
            })
            .await?;

        Ok(Response::new(CreateHelloResponse {}))
    }

    async fn read_hello(
        &self,
        request: Request<ReadHelloRequest>,
    ) -> Result<Response<ReadHelloResponse>, Status> {
        log::info!("Got a request: {:?}", request);

        let name = request.into_inner().name;

        let hello = self.usecase.find(name).await?;

        Ok(Response::new(ReadHelloResponse {
            hello: Some(Hello {
                name: hello.name,
                message: hello.message,
            }),
        }))
    }
}

```

自動生成されたgRPCのtraitをprotobufのリポジトリから読み込み、実装する形になります。

最後にDIをし、Serverにサービスを登録します。

```rust
// src/main.rs

use std::{env, sync::Arc};

use dotenv::dotenv;
use gakusai2024_backend::domain::repository::hello::HelloRepositoryTrait;
use gakusai2024_proto::api::hello_service_server::HelloServiceServer;
use sea_orm::Database;
use tokio::sync::Mutex;
use tonic::transport::Server;

use gakusai2024_backend::infrastructure;
use gakusai2024_backend::interface;
use gakusai2024_backend::interface::handler::hello::HelloHandlerTrait;
use gakusai2024_backend::usecase;
use gakusai2024_backend::usecase::hello::HelloUsecaseTrait;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    dotenv().ok();
    let addr = env::var("SERVER_ADDR")
        .expect("SERVER_ADDR must be set")
        .parse()?;
    let db_addr = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let conn = Database::connect(db_addr).await?;

    // Dependency Injection
    let hello_persistence =
        infrastructure::db::hello::HelloPersistence::new(Arc::new(Mutex::new(conn)));
    let hello_usecase = usecase::hello::HelloUsecase::new(Box::new(hello_persistence));
    let hello_handler = interface::handler::hello::HelloHandler::new(Box::new(hello_usecase));

    log::info!("GreeterServer listening on {}", addr);

    Server::builder()
        .add_service(HelloServiceServer::new(hello_handler))
        .serve(addr)
        .await?;

    Ok(())
}

```

また、実装が終わったら必ずE2Eテストを書いてください。

```rust
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

#[ignore]
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

```

以上で実装は終了です。
