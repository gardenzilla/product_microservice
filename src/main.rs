use prelude::*;
use protos::product::product_server::*;
use protos::product::*;
use std::{path::PathBuf, sync::Mutex};
use storaget::*;
use tonic::{transport::Server, Request, Response, Status};

pub mod convert;
pub mod id;
pub mod password;
pub mod prelude;
pub mod product;

pub struct ProductService {
    products: Mutex<VecPack<product::Product>>,
}

impl ProductService {
    fn new(products: Mutex<VecPack<product::Product>>) -> Self {
        Self { products }
    }
    fn create_new_product(
        &self,
        p: CreateNewRequest,
        created_by: String,
    ) -> ServiceResult<ProductObj> {
        let new_product =
            product::Product::new(p.name, p.quantity.into(), p.unti.into(), created_by)?;
        let user_obj: UserObj = (&new_user).into();
        self.users.lock().unwrap().insert(new_user)?;
        Ok(user_obj)
    }
}

#[tonic::async_trait]
impl Product for ProductService {
    async fn create_new(
        &self,
        request: Request<CreateNewRequest>,
    ) -> Result<Response<CreateNewResponse>, Status> {
        todo!()
    }

    async fn get_all(&self, request: Request<()>) -> Result<Response<GetAllResponse>, Status> {
        todo!()
    }

    async fn get_by_id(
        &self,
        request: Request<GetByIdRequest>,
    ) -> Result<Response<GetByIdResponse>, Status> {
        todo!()
    }

    async fn update_by_id(
        &self,
        request: Request<UpdateByIdRequest>,
    ) -> Result<Response<UpdateByIdResponse>, Status> {
        todo!()
    }

    async fn is_sku(
        &self,
        request: Request<IsSkuRequest>,
    ) -> Result<Response<IsSkuResponse>, Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> prelude::ServiceResult<()> {
    let products: Mutex<VecPack<product::Product>> = Mutex::new(
        VecPack::try_load_or_init(PathBuf::from("data/products"))
            .expect("Error while loading products storage"),
    );

    let product_service = ProductService::new(products);

    let addr = "[::1]:50054".parse().unwrap();

    Server::builder()
        .add_service(ProductServer::new(product_service))
        .serve(addr)
        .await
        .expect("Error while staring server"); // Todo implement ? from<?>

    Ok(())
}
