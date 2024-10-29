struct Impl {}

/// `greet` writes a greeting string to the host's output.
fn Guest::greet(self : Impl, name : String) -> Unit {
  let greeting = "Hello, \(name)!"
  fantix.examples.stdio.println(greeting)
}

fn init {
  init_guest(Impl::{  })
}

fn main {

}
