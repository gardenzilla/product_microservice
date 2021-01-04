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

use crate::prelude::*;
use chrono::prelude::*;
use gzlib::proto::product::*;
use packman::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Unit {
  Piece,
  Millimeter,
  Gram,
  Milliliter,
}

impl std::fmt::Display for Unit {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      Unit::Piece => write!(f, "db"),
      Unit::Milliliter => write!(f, "ml"),
      Unit::Gram => write!(f, "g"),
      Unit::Millimeter => write!(f, "mm"),
    }
  }
}

impl Into<String> for Unit {
  fn into(self) -> String {
    format!("{}", self)
  }
}

impl Unit {
  pub fn try_from_str(from: &str) -> ServiceResult<Unit> {
    let from = from.trim();
    let res = match from {
      "piece" => Unit::Piece,
      "db" => Unit::Piece,
      "millimeter" => Unit::Millimeter,
      "mm" => Unit::Millimeter,
      "gram" => Unit::Gram,
      "gr" => Unit::Gram,
      "g" => Unit::Gram,
      "milliliter" => Unit::Milliliter,
      "ml" => Unit::Milliliter,
      _ => {
        return Err(ServiceError::bad_request(&format!(
          "Wrong unit format: {}",
          from
        )))
      }
    };
    Ok(res)
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Quantity {
  Simple(u32),
  Complex(u32, u32),
}

impl PartialEq for Quantity {
  fn eq(&self, other: &Self) -> bool {
    match self {
      Quantity::Simple(q) => match other {
        Quantity::Simple(q2) => q == q2,
        Quantity::Complex(_, _) => false,
      },
      Quantity::Complex(m, q) => match other {
        Quantity::Simple(_) => false,
        Quantity::Complex(m2, q2) => m == m2 && q == q2,
      },
    }
  }
}

impl std::fmt::Display for Quantity {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match &self {
      Quantity::Simple(quantity) => write!(f, "{}", quantity),
      Quantity::Complex(multiplier, quantity) => write!(f, "{}x{}", multiplier, quantity),
    }
  }
}

impl Into<String> for Quantity {
  fn into(self) -> String {
    format!("{}", self)
  }
}

impl Quantity {
  pub fn try_from_str(s: &str) -> ServiceResult<Quantity> {
    let s = s.trim();

    let u32parser = |input: &str| -> ServiceResult<u32> {
      match input.parse::<u32>() {
        Ok(res) => Ok(res),
        Err(_) => Err(ServiceError::bad_request(
          "A megadott mennyiség csak pozitív egész számból állhat",
        )),
      }
    };
    match s.contains("x") {
      true => {
        let parts: Vec<&str> = s.split("x").collect();
        if parts.len() == 2 {
          let multiplier = if let Some(_multiplier) = parts.get(0) {
            u32parser(_multiplier)?
          } else {
            return Err(ServiceError::internal_error("This should never happen"));
          };
          let quantity = if let Some(_quantity) = parts.get(1) {
            u32parser(_quantity)?
          } else {
            return Err(ServiceError::internal_error("This should never happen"));
          };
          return Ok(Quantity::Complex(multiplier, quantity));
        } else {
          return Err(ServiceError::bad_request(
            "A komplex mennyiség csak 2 részből állhat. eg.: 3x5",
          ));
        }
      }
      false => return Ok(Quantity::Simple(u32parser(s)?)),
    }
  }
}

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

pub struct Sku {
  sku: u32,
  product_id: u32,
  sub_name: String,
  display_name: String,
  quantity: Quantity, // e.g.: Simple(u32) => 3 ml, or Complex(u32, u32) => 5x3 ml
  can_divide: bool,
  created_by: u32,
  created_at: DateTime<Utc>,
}

impl TryFrom for Product {
  type TryFrom = Product;
}

impl VecPackMember for Product {
  type Out = u32;
  fn get_id(&self) -> Self::Out {
    &self.sku
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_quantity_convert() {
    assert_eq!(Quantity::try_from_str("5").unwrap(), Quantity::Simple(5));
    assert_eq!(Quantity::try_from_str("7").unwrap(), Quantity::Simple(7));
    assert_eq!(Quantity::try_from_str("5e").is_err(), true);
    assert_eq!(Quantity::try_from_str("55").is_err(), false);
    assert_eq!(
      Quantity::try_from_str("1x2").unwrap(),
      Quantity::Complex(1, 2)
    );
    assert_eq!(Quantity::try_from_str("1x3x5").is_err(), true);
    assert_eq!(Quantity::try_from_str("1x").is_err(), true);
    assert_eq!(Quantity::try_from_str("1x3e").is_err(), true);
  }

  #[test]
  fn test_unit_convert() {
    assert_eq!(Unit::try_from_str("mm").unwrap(), Unit::Millimeter);
    assert_eq!(Unit::try_from_str("g").unwrap(), Unit::Gram);
    assert_eq!(Unit::try_from_str("ml").unwrap(), Unit::Milliliter);
    assert_eq!(Unit::try_from_str("piece").unwrap(), Unit::Piece);
    assert_eq!(Unit::try_from_str("db").unwrap(), Unit::Piece);
    assert_eq!(Unit::try_from_str("piecee").is_ok(), false);
    assert_eq!(Unit::try_from_str("kg").is_ok(), false);
    assert_eq!(Unit::try_from_str("grr").is_ok(), false);
    assert_eq!(Unit::try_from_str("g_").is_ok(), false);
    assert_eq!(Unit::try_from_str("m").is_ok(), false);
    assert_eq!(Unit::try_from_str("mm ").is_ok(), true);
    assert_eq!(Unit::try_from_str("g ").is_ok(), true);
    assert_eq!(Unit::try_from_str(" g ").is_ok(), true);
    assert_eq!(Unit::try_from_str(" db ").is_ok(), true);
    assert_eq!(Unit::try_from_str("     piece ").is_ok(), true);
  }
}
