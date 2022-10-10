pub mod query {
    use crate::msg::ValueResponse;
    pub fn value() -> ValueResponse {
        ValueResponse { value: 0 }
    }
}
