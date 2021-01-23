use gzlib::proto::product::{ProductObj, SkuObj};

pub enum ServiceError {
  InternalError(String),
  NotFound(String),
  AlreadyExists(String),
  BadRequest(String),
}

impl ServiceError {
  pub fn internal_error(msg: &str) -> Self {
    ServiceError::InternalError(msg.to_string())
  }
  pub fn not_found(msg: &str) -> Self {
    ServiceError::NotFound(msg.to_string())
  }
  pub fn already_exist(msg: &str) -> Self {
    ServiceError::AlreadyExists(msg.to_string())
  }
  pub fn bad_request(msg: &str) -> Self {
    ServiceError::BadRequest(msg.to_string())
  }
}

impl std::fmt::Display for ServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ServiceError::InternalError(msg) => write!(f, "{}", msg),
      ServiceError::NotFound(msg) => write!(f, "{}", msg),
      ServiceError::AlreadyExists(msg) => write!(f, "{}", msg),
      ServiceError::BadRequest(msg) => write!(f, "{}", msg),
    }
  }
}

impl std::fmt::Debug for ServiceError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("")
      .field(&"ServiceError".to_string())
      .field(self)
      .finish()
  }
}

impl From<ServiceError> for ::tonic::Status {
  fn from(error: ServiceError) -> Self {
    match error {
      ServiceError::InternalError(msg) => ::tonic::Status::internal(msg),
      ServiceError::NotFound(msg) => ::tonic::Status::not_found(msg),
      ServiceError::AlreadyExists(msg) => ::tonic::Status::already_exists(msg),
      ServiceError::BadRequest(msg) => ::tonic::Status::invalid_argument(msg),
    }
  }
}

impl From<::packman::PackError> for ServiceError {
  fn from(error: ::packman::PackError) -> Self {
    match error {
      ::packman::PackError::ObjectNotFound => ServiceError::not_found(&error.to_string()),
      _ => ServiceError::internal_error(&error.to_string()),
    }
  }
}

pub type ServiceResult<T> = Result<T, ServiceError>;

impl From<std::env::VarError> for ServiceError {
  fn from(error: std::env::VarError) -> Self {
    ServiceError::internal_error(&format!("ENV KEY NOT FOUND. {}", error))
  }
}

impl From<crate::product::Product> for ProductObj {
  fn from(p: crate::product::Product) -> Self {
    Self {
      product_id: p.product_id,
      name: p.name,
      description: p.description,
      unit: p.unit.to_string(),
      skus: p.skus.clone(),
      discontinued: p.discontinued,
      perishable: p.perishable,
      created_by: p.created_by,
      created_at: p.created_at.to_rfc3339(),
    }
  }
}

impl From<crate::product::Sku> for SkuObj {
  fn from(s: crate::product::Sku) -> Self {
    let divisible_amount = s.get_divisible_amount();
    Self {
      sku: s.sku,
      product_id: s.product_id,
      subname: s.sub_name,
      display_name: s.display_name,
      display_packaging: s.display_packaging,
      quantity: s.quantity.to_string(),
      unit: s.unit.to_string(),
      can_divide: s.can_divide,
      divisible_amount: divisible_amount,
      discontinued: s.discontinued,
      perishable: s.perishable,
      created_by: s.created_by,
      created_at: s.created_at.to_rfc3339(),
    }
  }
}
