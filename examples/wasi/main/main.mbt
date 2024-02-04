struct Impl {}

fn nonsense(self: Impl) -> Int64 {
  wasi.random.random.get_random_u64()
}

fn init {
  init_guest(Impl::{})
}
