use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Claims {
    id: i32,
}
