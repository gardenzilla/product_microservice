fn main() {
  println!(
    "Size of arc: {}",
    std::mem::size_of::<std::sync::Arc<std::sync::Mutex<u32>>>()
  );
  println!(
    "Size of mutex: {}",
    std::mem::size_of::<std::sync::Mutex<u32>>()
  );
  println!("Size of u32: {}", std::mem::size_of::<u32>());
}
