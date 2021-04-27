Categories
---

struct Node {
  id: u32,
  parent: u32,
  name: String,
}

pub trait NodeExt {
  fn new(..) -> Self;
  fn rename(..) -> Self;
  fn has_child() -> bool;
}

A
  AA
    AAA
    AAB
    AAC
  AB
  AC
  AD
