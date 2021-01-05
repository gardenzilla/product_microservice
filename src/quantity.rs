use std::fmt::Display;

use crate::prelude::*;
use chrono::prelude::*;
use gzlib::proto::product::*;
use packman::*;
use serde::{Deserialize, Serialize};
use thousands::Separable;

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
  pub fn to_display_unit(&self, quantity_display: &QuantityDisplay) -> String {
    match quantity_display {
      QuantityDisplay::Transformed(_) => {
        match self {
          // Piece remains piece
          Unit::Piece => self.to_string(),
          // MM to Meter
          Unit::Millimeter => "m".to_string(),
          // Gram to Kg
          Unit::Gram => "kg".to_string(),
          // Ml to Liter
          Unit::Milliliter => "l".to_string(),
        }
      }
      QuantityDisplay::Original(_) => format!("{}", &self),
    }
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
      Quantity::Simple(quantity) => write!(f, "{}", quantity.separate_with_spaces()),
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

enum QuantityDisplay<'a> {
  Transformed(Quantity),
  Original(&'a Quantity),
}

impl<'a> Display for QuantityDisplay<'a> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      QuantityDisplay::Transformed(q) => write!(f, "{}", q),
      QuantityDisplay::Original(q) => write!(f, "{}", q),
    }
  }
}

/// Convert a quantity and a unit to a nice looking
/// easier to look format
pub fn fancy_display(quantity: &Quantity, unit: &Unit) -> String {
  // Helper to decide wether transform quantity or not
  let can_transform = |u: u32| (u >= 1000) && (u % 1000 == 0);
  // Transform quantity
  let transformed = |q: &Quantity| match q {
    Quantity::Simple(_q) => match can_transform(*_q) {
      true => QuantityDisplay::Transformed(Quantity::Simple(_q / 1000)),
      false => QuantityDisplay::Original(quantity),
    },
    Quantity::Complex(_m, _q) => match can_transform(*_q) {
      true => QuantityDisplay::Transformed(Quantity::Complex(*_m, _q / 1000)),
      false => QuantityDisplay::Original(quantity),
    },
  };
  // Convert quantity to QuantityDisplay
  let quantity_transformed = transformed(quantity);
  // Create display string
  match unit {
    // When we have a Piece, we do not transform anything
    Unit::Piece => format!("{} {}", quantity, unit),
    _ => format!(
      "{} {}",
      &quantity_transformed,
      unit.to_display_unit(&quantity_transformed)
    ),
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

  #[test]
  fn test_fancy_display() {
    // Test piece transform
    assert_eq!(fancy_display(&Quantity::Simple(1), &Unit::Piece), "1 db");
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 1), &Unit::Piece),
      "3x1 db"
    );
    assert_eq!(fancy_display(&Quantity::Simple(10), &Unit::Piece), "10 db");
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 10), &Unit::Piece),
      "3x10 db"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(500), &Unit::Piece),
      "500 db"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(1000), &Unit::Piece),
      "1 000 db"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(11000), &Unit::Piece),
      "11 000 db"
    );
    // Test gram transform
    assert_eq!(fancy_display(&Quantity::Simple(500), &Unit::Gram), "500 g");
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 500), &Unit::Gram),
      "3x500 g"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(1100), &Unit::Gram),
      "1 100 g"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 1100), &Unit::Gram),
      "3x1100 g"
    );
    assert_eq!(fancy_display(&Quantity::Simple(1000), &Unit::Gram), "1 kg");
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 1000), &Unit::Gram),
      "3x1 kg"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(16000), &Unit::Gram),
      "16 kg"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 16000), &Unit::Gram),
      "3x16 kg"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(16500), &Unit::Gram),
      "16 500 g"
    );
    // Test mm transform
    assert_eq!(
      fancy_display(&Quantity::Simple(500), &Unit::Millimeter),
      "500 mm"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(150, 500), &Unit::Millimeter),
      "150x500 mm"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(1), &Unit::Millimeter),
      "1 mm"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(1000), &Unit::Millimeter),
      "1 m"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 1000), &Unit::Millimeter),
      "3x1 m"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(1500), &Unit::Millimeter),
      "1 500 mm"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 1500), &Unit::Millimeter),
      "3x1500 mm"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(3000), &Unit::Millimeter),
      "3 m"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 3000), &Unit::Millimeter),
      "3x3 m"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(15000), &Unit::Millimeter),
      "15 m"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(15001), &Unit::Millimeter),
      "15 001 mm"
    );
    // Test ml transform
    assert_eq!(
      fancy_display(&Quantity::Simple(1), &Unit::Milliliter),
      "1 ml"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(100), &Unit::Milliliter),
      "100 ml"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 100), &Unit::Milliliter),
      "3x100 ml"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(1000), &Unit::Milliliter),
      "1 l"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 1000), &Unit::Milliliter),
      "3x1 l"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(2000), &Unit::Milliliter),
      "2 l"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(1400), &Unit::Milliliter),
      "1 400 ml"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(9, 1400), &Unit::Milliliter),
      "9x1400 ml"
    );
    assert_eq!(
      fancy_display(&Quantity::Simple(13000), &Unit::Milliliter),
      "13 l"
    );
    assert_eq!(
      fancy_display(&Quantity::Complex(3, 13000), &Unit::Milliliter),
      "3x13 l"
    );
  }
}
