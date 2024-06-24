struct Impl {}

fn Guest::hello(self : Impl, name : String) -> Unit {
  fantix.examples.stdio.println("Hello, \(name)!")
}

fn init {
  init_guest(Impl::{  })
}

fn main {

}
