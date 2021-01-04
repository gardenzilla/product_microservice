pub struct Product {
  id: u32,
  name: String,
  description: String,
  related_skus: Vec<u32>,
  unit: (),
  display_unit: (),
}

pub struct Sku {
  id: u32,
  product_id: u32,
  name_product: String,
  name_sub: String,
  name_display: String,
  price_retail_net: i32,
  price_vat: Vat,
  price_retail_gross: i32,
  quantity: (), // Quantity
  unit: (),     // Unit
  display_unit: (),
  divisible: bool,
}

pub enum Vat {
  TAM,
  AAM,
  FAD,
  _5,
  _27,
}

pub enum Unit {
  Mm,
  Gram,
  Ml,
  Piece,
}

mod HexId {
  use core::convert::TryFrom;
  use std::num::ParseIntError;

  trait HexId {
    fn to_hex_string(&self) -> String;
    fn from_hex_str(str: &str) -> Result<Self, ParseIntError>
    where
      Self: Sized;
  }

  impl HexId for i32 {
    fn to_hex_string(&self) -> String {
      format!("{:x}", self)
    }
    fn from_hex_str(str: &str) -> Result<Self, ParseIntError> {
      Self::from_str_radix(str, 16)
    }
  }

  #[cfg(test)]
  mod tests {
    use super::*;
    #[test]
    fn demo_to_hex_string() {
      assert_eq!("c", 12.to_hex_string());
      assert_eq!("9", 9.to_hex_string());
      assert_eq!("a", 10.to_hex_string());
    }
    #[test]
    fn demo_from_hex_string() {
      assert_eq!(12, i32::from_hex_str("c").unwrap());
      assert_eq!(10, i32::from_hex_str("a").unwrap());
      assert_eq!(1459, i32::from_hex_str("5b3").unwrap());
    }
  }
}
