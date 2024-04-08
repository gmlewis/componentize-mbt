struct Impl {}

fn Guest::hello(self: Impl, name: String) -> Unit {
  fantix.examples.stdio.println("你好，\(name)！")
}

fn init {
  init_guest(Impl::{})
}
