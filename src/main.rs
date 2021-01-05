use gzlib::proto::product::product_server::*;
use gzlib::proto::product::*;
use packman::*;
use prelude::*;
use std::path::PathBuf;
use tokio::sync::{oneshot, Mutex};
use tonic::{transport::Server, Request, Response, Status};

mod convert;
mod id;
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
}

#[tokio::main]
async fn main() -> prelude::ServiceResult<()> {
  // let product_db: VecPack<product::Product> =
  //   VecPack::try_load_or_init(PathBuf::from("data/products"))
  //     .expect("Error while loading product storage");

  // let sku_db: VecPack<product::Sku> =
  //   VecPack::try_load_or_init(PathBuf::from("data/skus")).expect("Error while loading sku storage");

  // let product_service = ProductService::init(product_db, sku_db);

  // let addr = "[::1]:50054".parse().unwrap();

  // // Create shutdown channel
  // let (tx, rx) = oneshot::channel();

  // // Spawn the server into a runtime
  // tokio::task::spawn(async move {
  //   Server::builder()
  //     .add_service(ProductServer::new(product_service))
  //     .serve_with_shutdown(addr, async { rx.await.unwrap() })
  //     .await
  // });

  // tokio::signal::ctrl_c().await.unwrap();

  // println!("SIGINT");

  // // Send shutdown signal after SIGINT received
  // let _ = tx.send(());

  Ok(())
}
