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
