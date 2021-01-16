// Copyright (C) 2020 Peter Mezei
//
// This file is part of Gardenzilla.
//
// Gardenzilla is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 2 of the License, or
// (at your option) any later version.
//
// Gardenzilla is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Gardenzilla.  If not, see <http://www.gnu.org/licenses/>.

use crate::quantity::*;
use chrono::prelude::*;
use packman::*;
use serde::{Deserialize, Serialize};
use sku_version_update::SkuOld;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Product {
  /// Product ID
  pub product_id: u32,
  /// Product name
  pub name: String,
  /// Product description
  pub description: String,
  /// Product unit
  pub unit: Unit, // e.g.: ml
  /// Related SKUs
  pub skus: Vec<u32>,
  /// Created by UID
  pub created_by: u32,
  /// Created at
  pub created_at: DateTime<Utc>,
}

impl Product {
  /// Create new product object
  pub fn new(
    product_id: u32,
    name: String,
    description: String,
    unit: Unit,
    created_by: u32,
  ) -> Self {
    Self {
      product_id,
      name,
      description,
      unit,
      skus: Vec::new(),
      created_by,
      created_at: Utc::now(),
    }
  }
  /// Update product data
  pub fn update(&mut self, name: String, description: String, unit: Unit) -> &Self {
    self.name = name;
    self.description = description;
    self.unit = unit;
    self
  }
  // Add related SKU
  pub fn add_sku(&mut self, sku: u32) -> &Self {
    self.skus.push(sku);
    self
  }
}

impl Default for Product {
  fn default() -> Self {
    Self {
      product_id: 0,
      name: String::default(),
      description: String::default(),
      unit: Unit::Milliliter,
      skus: Vec::new(),
      created_by: 0,
      created_at: Utc::now(),
    }
  }
}

impl TryFrom for Product {
  type TryFrom = Product;
}

impl VecPackMember for Product {
  type Out = u32;
  fn get_id(&self) -> &Self::Out {
    &self.product_id
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sku {
  // SKU ID
  pub sku: u32,
  // Related product_id
  pub product_id: u32,
  // Related product name
  pub parent_name: String,
  // SKU sub name
  pub sub_name: String,
  // Product name + sub name + packaging
  pub display_name: String,
  // Quantity + unit as fancy display
  pub display_packaging: String,
  // Related product unit
  pub unit: Unit,
  // Sku quantity
  pub quantity: Quantity,
  // UPLs can divide?
  // Only if Quantity::Simple(_)
  pub can_divide: bool,
  // Created by UID
  pub created_by: u32,
  // Created at
  pub created_at: DateTime<Utc>,
}

impl Sku {
  pub fn new(
    sku: u32,
    product_id: u32,
    parent: &Product,
    sub_name: String,
    quantity: Quantity,
    created_by: u32,
  ) -> Self {
    let mut res = Self {
      sku,
      product_id,
      parent_name: parent.name.clone(),
      sub_name,
      display_name: String::default(),
      display_packaging: String::default(),
      quantity,
      unit: parent.unit.clone(),
      can_divide: false,
      created_by,
      created_at: Utc::now(),
    };
    res.reset();
    res
  }
  /// Update SKU data based on its related parent &Product
  pub fn update_parent(&mut self, parent: &Product) -> &Self {
    self.parent_name = parent.name.clone();
    self.unit = parent.unit.clone();
    self.reset();
    self
  }
  /// Update SKU data
  pub fn update(&mut self, sub_name: String, quantity: Quantity) -> &Self {
    self.sub_name = sub_name;
    self.quantity = quantity;
    self.reset();
    self
  }
  /// Try to set divide
  pub fn set_divide(&mut self, can_divide: bool) -> Result<&Self, String> {
    // If can_divide false
    // Then we set it without conditions
    if !can_divide {
      self.can_divide = false;
      return Ok(self);
    }
    // If can_divide true,
    // we check if quantity is Simple, then set it to true
    // otherwise return error
    match self.quantity {
      Quantity::Simple(_) => {
        self.can_divide = true;
        Ok(self)
      }
      _ => Err("Csak egyszerű mennyiség lehet osztható!".to_string()),
    }
  }
  /// Central reset function
  /// This calls all the needed reset sub methods
  /// Call order important!
  pub fn reset(&mut self) {
    self.reset_display_packaging();
    self.reset_display_name();
  }
  /// Reset display_name by a parent &Product data
  /// and self data
  pub fn reset_display_name(&mut self) {
    self.display_name = format!(
      "{} {}, {}",
      self.parent_name, self.sub_name, self.display_packaging
    );
  }
  /// Reset display_packaging
  /// based on the stored quantity and unit
  pub fn reset_display_packaging(&mut self) {
    self.display_packaging = fancy_display(&self.quantity, &self.unit);
  }
}

impl Default for Sku {
  fn default() -> Self {
    Self {
      sku: 0,
      product_id: 0,
      parent_name: String::default(),
      sub_name: String::default(),
      display_name: String::default(),
      display_packaging: String::default(),
      quantity: Quantity::Simple(0),
      unit: Unit::Milliliter,
      can_divide: false,
      created_by: 0,
      created_at: Utc::now(),
    }
  }
}

mod sku_version_update {
  use crate::*;
  use chrono::prelude::*;
  use serde::{Deserialize, Serialize};
  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub enum QuantityOld {
    Simple(u32),
    Complex(u32, u32),
  }
  impl Default for QuantityOld {
    fn default() -> Self {
      Self::Simple(0)
    }
  }
  #[derive(Serialize, Deserialize, Clone, Debug)]
  pub struct SkuOld {
    // SKU ID
    pub sku: u32,
    // Related product_id
    pub product_id: u32,
    // Related product name
    pub parent_name: String,
    // SKU sub name
    pub sub_name: String,
    // Product name + sub name + packaging
    pub display_name: String,
    // Quantity + unit as fancy display
    pub display_packaging: String,
    // Related product unit
    pub unit: Unit,
    // Sku quantity
    pub quantity: QuantityOld,
    // UPLs can divide?
    // Only if Quantity::Simple(_)
    pub can_divide: bool,
    // Created by UID
    pub created_by: u32,
    // Created at
    pub created_at: DateTime<Utc>,
  }
  impl Default for SkuOld {
    fn default() -> Self {
      Self {
        sku: 0,
        product_id: 0,
        parent_name: String::default(),
        sub_name: String::default(),
        display_name: String::default(),
        display_packaging: String::default(),
        unit: Unit::Milliliter,
        quantity: QuantityOld::default(),
        can_divide: false,
        created_by: 0,
        created_at: Utc::now(),
      }
    }
  }
}

impl TryFrom for Sku {
  type TryFrom = Sku;
}

impl From<SkuOld> for Sku {
  fn from(so: SkuOld) -> Self {
    Self {
      sku: so.sku,
      product_id: so.product_id,
      parent_name: so.parent_name,
      sub_name: so.sub_name,
      display_name: so.display_name,
      display_packaging: so.display_packaging,
      unit: so.unit,
      quantity: match so.quantity {
        sku_version_update::QuantityOld::Simple(q) => Quantity::Simple(q),
        sku_version_update::QuantityOld::Complex(m, q) => Quantity::Complex(m, q),
      },
      can_divide: so.can_divide,
      created_by: so.created_by,
      created_at: so.created_at,
    }
  }
}

impl VecPackMember for Sku {
  type Out = u32;
  fn get_id(&self) -> &Self::Out {
    &self.sku
  }
}
