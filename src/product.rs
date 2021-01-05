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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Product {
  product_id: u32,
  name: String,
  description: String,
  unit: Unit, // e.g.: ml
  skus: Vec<u32>,
  created_by: u32,
  created_at: DateTime<Utc>,
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
  sku: u32,
  // Related product_id
  product_id: u32,
  // Related product name
  parent_name: String,
  // SKU sub name
  sub_name: String,
  // Product name + sub name + packaging
  display_name: String,
  // Quantity + unit as fancy display
  display_packaging: String,
  // Related product unit
  unit: Unit,
  // Sku quantity
  quantity: Quantity,
  // UPLs can divide?
  // Only if Quantity::Simple(_)
  can_divide: bool,
  // Created by UID
  created_by: u32,
  // Created at
  created_at: DateTime<Utc>,
}

impl Sku {
  pub fn new(
    sku: u32,
    product_id: u32,
    parent: &Product,
    sub_name: String,
    quantity: Quantity,
    can_divide: bool,
    created_by: u32,
  ) -> Self {
    Self {
      sku,
      product_id,
      parent_name: parent.name.clone(),
      sub_name,
      display_name: String::default(),
      display_packaging: fancy_display(&quantity, &parent.unit),
      quantity,
      unit: parent.unit.clone(),
      can_divide,
      created_by,
      created_at: Utc::now(),
    }
  }
  /// Update SKU data based on its related parent &Product
  pub fn update_parent(&mut self, parent: &Product) -> &Self {
    self.parent_name = parent.name.clone();
    self.unit = parent.unit.clone();
    self.reset();
    self
  }
  /// Update SKU data
  pub fn update(&mut self, sub_name: String, quantity: Quantity, can_divide: bool) -> &Self {
    self.sub_name = sub_name;
    self.quantity = quantity;
    self.can_divide = can_divide;
    self.reset();
    self
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

impl TryFrom for Sku {
  type TryFrom = Sku;
}

impl VecPackMember for Sku {
  type Out = u32;
  fn get_id(&self) -> &Self::Out {
    &self.sku
  }
}
