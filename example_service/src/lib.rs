#[tarpc::service]
pub trait HelloWorld {
    async fn hello(name: String) -> String;
}
