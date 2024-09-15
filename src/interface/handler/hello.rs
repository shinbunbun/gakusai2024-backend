use gakusai2024_proto::hello::{hello_service_server::HelloService, HelloRequest, HelloResponse};
use tonic::{Request, Response, Status};

use crate::{domain::repository::hello::HelloRepository, usecase::hello::HelloUsecaseTrait};

pub trait HelloHandlerTrait<HU, HR>
where
    HU: HelloUsecaseTrait<HR>,
    HR: HelloRepository + 'static,
{
    fn new(usecase: Box<HU>) -> Self
    where
        Self: Sized;
}

pub struct HelloHandler<HU, HR>
where
    HU: HelloUsecaseTrait<HR>,
    HR: HelloRepository + 'static,
{
    usecase: Box<HU>,
    _phantom: std::marker::PhantomData<HR>,
}

impl<HU, HR> HelloHandlerTrait<HU, HR> for HelloHandler<HU, HR>
where
    HU: HelloUsecaseTrait<HR>,
    HR: HelloRepository,
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
    HR: HelloRepository + Sync + Send + 'static,
{
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        log::info!("Got a request: {:?}", request);

        let name = request.into_inner().name;

        _ = self
            .usecase
            .insert(crate::domain::hello::Hello {
                name: name.clone(),
                message: "Hello, ".to_string() + &name,
            })
            .await?;

        let hello = self.usecase.find(name).await?;

        let reply = HelloResponse {
            message: hello.message,
        };

        Ok(Response::new(reply))
    }
}
