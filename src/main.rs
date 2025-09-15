mod app;
mod build;

fn main() {
    build::web::build();
    println!("Server runs");
}
