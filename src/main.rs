use gzlib::proto::product::product_server::*;
use gzlib::proto::product::*;
use packman::*;
use prelude::*;
use quantity::{Quantity, Unit};
use std::{env, path::PathBuf};
use tokio::sync::{oneshot, Mutex};
use tonic::{transport::Server, Request, Response, Status};

mod convert;
mod prelude;
mod product;
mod quantity;

struct ProductService {
  products: Mutex<VecPack<product::Product>>,
  skus: Mutex<VecPack<product::Sku>>,
}

impl ProductService {
  /// Init new product service with the required DBs
  fn init(product_db: VecPack<product::Product>, sku_db: VecPack<product::Sku>) -> Self {
    Self {
      products: Mutex::new(product_db),
      skus: Mutex::new(sku_db),
    }
  }
  /// Get next product id to use
  async fn next_product_id(&self) -> u32 {
    let mut latest_id: u32 = 0;
    self.products.lock().await.iter().for_each(|product| {
      let id: u32 = *product.unpack().get_id();
      if id > latest_id {
        latest_id = id;
      }
    });
    latest_id + 1
  }
  /// Get next SKU id to use
  async fn next_sku(&self) -> u32 {
    let mut latest_id: u32 = 0;
    self.skus.lock().await.iter().for_each(|sku| {
      let id: u32 = *sku.unpack().get_id();
      if id > latest_id {
        latest_id = id;
      }
    });
    latest_id + 1
  }
  // Create new product
  async fn create_product(&self, r: NewProduct) -> ServiceResult<ProductObj> {
    // Get the next product id
    let next_product_id = self.next_product_id().await;
    // Create new product object
    let new_product = product::Product::new(
      next_product_id,
      r.name,
      r.description,
      Unit::try_from_str(&r.unit)?,
      r.created_by,
    );
    // Store new product in storage
    self.products.lock().await.insert(new_product.clone())?;
    // Return new product as ProductObj
    Ok(new_product.into())
  }
  // Get all product
  async fn get_product_all(&self) -> ServiceResult<Vec<u32>> {
    // Create a product id vector from all the products available
    let res = self
      .products
      .lock()
      .await
      .iter()
      .map(|p| *p.unpack().get_id())
      .collect::<Vec<u32>>();
    // Return ID vector
    Ok(res)
  }
  // Get product by ID
  async fn get_product(&self, r: GetProductRequest) -> ServiceResult<ProductObj> {
    // Try to find PID
    let res = self
      .products
      .lock()
      .await
      .find_id(&r.product_id)?
      .unpack()
      .clone();
    // Return product as ProductObj
    Ok(res.into())
  }
  // Get product in bulk
  async fn get_product_bulk(&self, r: GetProductBulkRequest) -> ServiceResult<Vec<ProductObj>> {
    // Filters the required product IDs
    let res = self
      .products
      .lock()
      .await
      .iter()
      .filter(|p| r.product_ids.contains(p.unpack().get_id()))
      .map(|p| p.unpack().clone().into())
      .collect::<Vec<ProductObj>>();
    // Return result as Vec<ProductObj>
    Ok(res)
  }
  // Tries to update product object
  async fn update_product(&self, r: ProductObj) -> ServiceResult<ProductObj> {
    // Define product_id to update
    let product_id = r.product_id;
    // Find and update product
    let res = self
      .products
      .lock()
      .await
      .find_id_mut(&r.product_id)?
      .as_mut()
      .unpack()
      .update(r.name, r.description, Unit::try_from_str(&r.unit)?)
      .clone();
    // Update all related SKUs with product updates
    self
      .skus
      .lock()
      .await
      .as_vec_mut()
      .iter_mut()
      .filter(|s| s.unpack().product_id == product_id)
      .for_each(|s| {
        s.as_mut().unpack().update_parent(&res);
      });
    // Return result as ProductObj
    Ok(res.into())
  }
  // Find products by query
  async fn find_product(&self, r: FindProductRequest) -> ServiceResult<Vec<u32>> {
    // Filter products if their names contain the required query str
    let res = self
      .products
      .lock()
      .await
      .iter()
      .filter(|p| p.unpack().name.to_lowercase().contains(&r.query))
      .map(|p| *p.unpack().get_id())
      .collect::<Vec<u32>>();
    // Return result product id vector
    Ok(res)
  }
  // Create new sku
  async fn create_sku(&self, r: NewSku) -> ServiceResult<SkuObj> {
    // Query the next SKU ID
    let next_sku_id = self.next_sku().await;
    // Find product object as parent
    let parent = self
      .products
      .lock()
      .await
      .find_id(&r.product_id)
      .map_err(|_| {
        ServiceError::bad_request("A SKU nem hozható létre, a megadott termék ID nem létezik!")
      })?
      .unpack()
      .clone();
    // Create new SKU object
    let new_sku = product::Sku::new(
      next_sku_id,
      r.product_id,
      &parent,
      r.sub_name,
      Quantity::try_from_str(&r.quantity)?,
      r.created_by,
    );
    // Insert new SKU into storage
    self.skus.lock().await.insert(new_sku.clone())?;
    // Add SKU to its parent product
    let _ = self
      .products
      .lock()
      .await
      .find_id_mut(&r.product_id)?
      .unpack()
      .add_sku(new_sku.sku);
    // Return new_sku as SkuObj
    Ok(new_sku.into())
  }
  // Get all SKU
  async fn get_sku_all(&self) -> ServiceResult<Vec<u32>> {
    // Collect all the IDs
    let res = self
      .skus
      .lock()
      .await
      .iter()
      .map(|s| *s.unpack().get_id())
      .collect::<Vec<u32>>();
    // Return IDs as vector
    Ok(res)
  }
  // Get SKU by ID
  async fn get_sku(&self, r: GetSkuRequest) -> ServiceResult<SkuObj> {
    // Find SKU
    let res = self.skus.lock().await.find_id(&r.sku_id)?.unpack().clone();
    // Return SKU as SkuObj
    Ok(res.into())
  }
  // Get SKUs in bulk
  async fn get_sku_bulk(&self, r: GetSkuBulkRequest) -> ServiceResult<Vec<SkuObj>> {
    // Find the requested SKUs
    let res = self
      .skus
      .lock()
      .await
      .iter()
      .filter(|s| r.sku_id.contains(s.unpack().get_id()))
      .map(|s| s.unpack().clone().into())
      .collect::<Vec<SkuObj>>();
    // Return SKUs as SkuObj vector
    Ok(res)
  }
  // Try to update SKU
  async fn update_sku(&self, r: SkuObj) -> ServiceResult<SkuObj> {
    // Find and update SKU
    let res = self
      .skus
      .lock()
      .await
      .find_id_mut(&r.sku)?
      .as_mut()
      .unpack()
      .update(r.subname, Quantity::try_from_str(&r.quantity)?)
      .clone();
    // Return SKU as SkuObj
    Ok(res.into())
  }
  // Try to update SKU divide
  async fn update_sku_divide(&self, r: UpdateSkuDivideRequest) -> ServiceResult<SkuObj> {
    // Find SKU and tries to update its divide
    let res = self
      .skus
      .lock()
      .await
      .find_id_mut(&r.sku)?
      .as_mut()
      .unpack()
      .set_divide(r.can_divide)
      .map_err(|e| ServiceError::bad_request(&e))?
      .clone();
    // Returns Sku as SkuObj
    Ok(res.into())
  }
  // Find SKUs
  async fn find_sku(&self, r: FindSkuRequest) -> ServiceResult<Vec<u32>> {
    // Filter SKUs if their display names contain the required query str
    let res = self
      .skus
      .lock()
      .await
      .iter()
      .filter(|s| s.unpack().display_name.to_lowercase().contains(&r.query))
      .map(|s| *s.unpack().get_id())
      .collect::<Vec<u32>>();
    // Return result SKU ids as vector
    Ok(res)
  }
}

#[tonic::async_trait]
impl gzlib::proto::product::product_server::Product for ProductService {
  async fn create_product(
    &self,
    request: Request<NewProduct>,
  ) -> Result<Response<ProductObj>, Status> {
    let res = self.create_product(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn get_product_all(&self, _: Request<()>) -> Result<Response<ProductIds>, Status> {
    let res = self.get_product_all().await?;
    Ok(Response::new(ProductIds { product_ids: res }))
  }

  async fn get_product(
    &self,
    request: Request<GetProductRequest>,
  ) -> Result<Response<ProductObj>, Status> {
    let res = self.get_product(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  type GetProductBulkStream = tokio::sync::mpsc::Receiver<Result<ProductObj, Status>>;

  async fn get_product_bulk(
    &self,
    request: Request<GetProductBulkRequest>,
  ) -> Result<Response<Self::GetProductBulkStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.get_product_bulk(request.into_inner()).await?;

    // Send the result items through the channel
    for sobject in res {
      tx.send(Ok(sobject))
        .await
        .map_err(|_| Status::internal("Error while sending sources over channel"))?;
    }

    // Send back the receiver
    Ok(Response::new(rx))
  }

  async fn update_product(
    &self,
    request: Request<ProductObj>,
  ) -> Result<Response<ProductObj>, Status> {
    let res = self.update_product(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn find_product(
    &self,
    request: Request<FindProductRequest>,
  ) -> Result<Response<SkuIds>, Status> {
    let res = self.find_product(request.into_inner()).await?;
    Ok(Response::new(SkuIds { sku_ids: res }))
  }

  async fn create_sku(&self, request: Request<NewSku>) -> Result<Response<SkuObj>, Status> {
    let res = self.create_sku(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn get_sku_all(&self, _: Request<()>) -> Result<Response<SkuIds>, Status> {
    let res = self.get_sku_all().await?;
    Ok(Response::new(SkuIds { sku_ids: res }))
  }

  async fn get_sku(&self, request: Request<GetSkuRequest>) -> Result<Response<SkuObj>, Status> {
    let res = self.get_sku(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  type GetSkuBulkStream = tokio::sync::mpsc::Receiver<Result<SkuObj, Status>>;

  async fn get_sku_bulk(
    &self,
    request: Request<GetSkuBulkRequest>,
  ) -> Result<Response<Self::GetSkuBulkStream>, Status> {
    // Create channel for stream response
    let (mut tx, rx) = tokio::sync::mpsc::channel(100);

    // Get resources as Vec<SourceObject>
    let res = self.get_sku_bulk(request.into_inner()).await?;

    // Send the result items through the channel
    for sobject in res {
      tx.send(Ok(sobject))
        .await
        .map_err(|_| Status::internal("Error while sending sources over channel"))?;
    }

    // Send back the receiver
    Ok(Response::new(rx))
  }

  async fn update_sku(&self, request: Request<SkuObj>) -> Result<Response<SkuObj>, Status> {
    let res = self.update_sku(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn update_sku_divide(
    &self,
    request: Request<UpdateSkuDivideRequest>,
  ) -> Result<Response<SkuObj>, Status> {
    let res = self.update_sku_divide(request.into_inner()).await?;
    Ok(Response::new(res))
  }

  async fn find_sku(&self, request: Request<FindSkuRequest>) -> Result<Response<SkuIds>, Status> {
    let res = self.find_sku(request.into_inner()).await?;
    Ok(Response::new(SkuIds { sku_ids: res }))
  }
}

#[tokio::main]
async fn main() -> prelude::ServiceResult<()> {
  let product_db: VecPack<product::Product> =
    VecPack::try_load_or_init(PathBuf::from("data/products"))
      .expect("Error while loading product storage");

  let sku_db: VecPack<product::Sku> =
    VecPack::try_load_or_init(PathBuf::from("data/skus")).expect("Error while loading sku storage");

  let product_service = ProductService::init(product_db, sku_db);

  let addr = env::var("SERVICE_ADDR_PRODUCT")
    .unwrap_or("[::1]:50054".into())
    .parse()
    .unwrap();

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
