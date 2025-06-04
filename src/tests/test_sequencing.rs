use crate::tests::test_sequencing_crud;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_sequencing_crud().await
}
