use prelude::*;
use protos::product::product_server::*;
use protos::product::*;
use std::path::PathBuf;
use storaget::*;
use tokio::sync::{oneshot, Mutex};
use tonic::{transport::Server, Request, Response, Status};

pub mod convert;
pub mod id;
pub mod prelude;
pub mod product;

pub struct ProductService {
  products: Mutex<VecPack<product::Product>>,
}

impl ProductService {
  fn new(products: Mutex<VecPack<product::Product>>) -> Self {
    Self { products }
  }
  async fn create_new_product(&self, p: CreateNewRequest) -> ServiceResult<ProductObj> {
    let new_product = product::Product::new(
      p.name,
      product::Quantity::try_from_str(&p.quantity)?,
      product::Unit::try_from_str(&p.unit)?,
      p.created_by,
    )?;
    self.products.lock().await.insert(new_product.clone())?;
    Ok(new_product.into())
  }
}

#[tonic::async_trait]
impl Product for ProductService {
  async fn create_new(
    &self,
    request: Request<CreateNewRequest>,
  ) -> Result<Response<CreateNewResponse>, Status> {
    let res = self.create_new_product(request.into_inner()).await?;
    Ok(Response::new(CreateNewResponse { product: Some(res) }))
  }

  async fn get_all(&self, _request: Request<()>) -> Result<Response<GetAllResponse>, Status> {
    let products: Vec<ProductObj> = self
      .products
      .lock()
      .await
      .into_iter()
      .map(|p: &mut Pack<product::Product>| p.unpack().into())
      .collect::<Vec<ProductObj>>();
    Ok(Response::new(GetAllResponse { products: products }))
  }

  async fn get_by_id(
    &self,
    request: Request<GetByIdRequest>,
  ) -> Result<Response<GetByIdResponse>, Status> {
    let product: ProductObj = self
      .products
      .lock()
      .await
      .find_id(&request.into_inner().sku)
      .map_err(|_| Status::not_found("Product not found"))?
      .unpack()
      .into();
    let response = GetByIdResponse {
      product: Some(product),
    };
    return Ok(Response::new(response));
  }

  async fn update_by_id(
    &self,
    request: Request<UpdateByIdRequest>,
  ) -> Result<Response<UpdateByIdResponse>, Status> {
    let _product: ProductUpdateObj = match request.into_inner().product {
      Some(u) => u,
      None => return Err(Status::internal("Request has an empty user object")),
    };
    let mut lock = self.products.lock().await;
    let product = match lock.find_id_mut(&_product.sku) {
      Ok(u) => u,
      Err(err) => return Err(Status::not_found(format!("{}", err))),
    };

    {
      let mut product_mut = product.as_mut();
      let mut _product_mut = product_mut.unpack();
      _product_mut.set_name(_product.name.to_string());
      _product_mut.set_quantity(product::Quantity::try_from_str(&_product.quantity)?);
      _product_mut.set_unit(product::Unit::try_from_str(&_product.unit)?);
    }

    let response = UpdateByIdResponse {
      product: Some(product.unpack().into()),
    };
    return Ok(Response::new(response));
  }

  async fn is_sku(
    &self,
    request: Request<IsSkuRequest>,
  ) -> Result<Response<IsSkuResponse>, Status> {
    let res = match self
      .products
      .lock()
      .await
      .find_id(&request.into_inner().sku)
    {
      Ok(_) => true,
      Err(_) => false,
    };
    let response = IsSkuResponse { sku_exist: res };
    return Ok(Response::new(response));
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

  // Create shutdown channel
  let (tx, rx) = oneshot::channel();

  // Spawn the server into a runtime
  tokio::task::spawn(async move {
    Server::builder()
      .add_service(ProductServer::new(product_service))
      .serve_with_shutdown(addr, async { rx.await.unwrap() })
      .await
  });

  tokio::signal::ctrl_c().await.unwrap();

  println!("SIGINT");

  // Send shutdown signal after SIGINT received
  let _ = tx.send(());

  Ok(())
}
