use serde::{Serialize, Deserialize};

enum Types {
    Bool, Number, Chunk, Array, Table
}

#[derive(Serialize, Deserialize)]
struct InterOpLocation {
    beg: u32,
    size: u32
}

#[derive(Serialize, Deserialize)]
struct InterOpValue {
    info: String,
    loc: InterOpLocation,
}
