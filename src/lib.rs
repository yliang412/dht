#[tarpc::service]
pub trait HashNode {
    async fn get(key: String) -> Option<String>;
    async fn insert(key: String, value: String) -> Option<String>;
    async fn remove(key: String) -> Option<String>;
}
