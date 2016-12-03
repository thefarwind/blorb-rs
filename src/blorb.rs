use std::collections::HashMap;

// Metadata Structs
////////////////////////////////////////////////////////////////////////

/// Metadata struct for `FORM` chunks. Used for identifying a form
/// without loading the full form into memory.
///
/// Forms are a type of a chunk which contain additional chunks. These
/// are called out separately to chunks due to the large number of form
/// types.
///
/// **NOTE**: The `len` includes the 4 bytes in `id`. The remaining
/// length of the chunk after the `id` is `len - 4`.
#[derive(Debug)]
pub struct FormData {
    /// the length of the form, not counting the 8 byte chunk header
    pub len: u32,
    /// the 4 byte ascii id. The length includes these bytes.
    pub id: [u8; 0x4],
}


/// Container for chunk metadata. Used for identifying a chunk without
/// loading the full chunk into memory.
#[derive(Debug)]
pub struct ChunkData {
    /// The 4 byte ascii id of the chunk
    pub id: [u8; 0x4],
    /// The length of the form, not counting the 8 byte chunk header
    pub len: u32,
}


impl From<FormData> for ChunkData {
    fn from(form_data: FormData) -> ChunkData {
        ChunkData{len: form_data.len, id: *b"FORM"}
    }
}


// Chunk Structs
////////////////////////////////////////////////////////////////////////


/// The usage information for an `IndexEntry`.
#[derive(Debug)]
pub enum Usage {
    /// Identifier: `b"Pict"`.
    /// Indicates the resource is an image.
    Pict,

    /// Identifier: `b"Snd "`.
    /// Indicates the resource is a sound.
    Snd,

    /// Indentifier: `b"Data"`.
    /// Indicates the resource is some data.
    Data,

    /// Identifier: `b"Exec"`.
    /// Indicates the resource is an executable.
    Exec,
}


/// Contains the usage information for an entry, the resource number of
/// the entry, and where in the blob the entry starts.
#[derive(Debug)]
pub struct IndexEntry {
    /// The type of the resource
    pub usage: Usage,
    /// The index of the resource
    pub num: u32,
    /// The starting address of the resource
    pub start: u32,
}


/// Container for list of resource index entries.
#[derive(Debug)]
pub struct ResourceIndex {
    /// a map of index value of a resource to the index entry of the
    /// resource.
    pub entries: HashMap<usize, IndexEntry>,
}


/// Representation for loaded blorb chunks
pub enum Chunk {

    /// Chunk returned when the loaded chunk type is unable to be
    /// identified. Per Specification, the machine must ignore unknown
    /// chunks, and this type will be used to do so when necessary.
    Unknown{meta: ChunkData, data: Vec<u8>},

    /// Identifier: `b"RIdx"`.
    /// Contains a resource index for the IF.
    /// This chunk is mandatory and must be the first chunk in the blorb.
    ResourceIndex{index: ResourceIndex},

    /// Identifier: `b"IFmd"`.
    /// Contains xml metadata content for the IF.
    Metadata{info: String},

    /// Identifier: `b"Fspc"`.
    /// Contains a reference to a frontispiece image.
    /// This chunk is optional.
    Frontispiece{num: u32},

    /// Identifier: `b"ZCOD"`.
    /// Contains Z-code executable.
    /// This is an executable resource chunk.
    ZCode{code: Vec<u8>},

    /// Identifier: `b"GLUL"`.
    /// Contains Glulx executable.
    /// This is an executable resource chunk.
    Glulx{code: Vec<u8>},

    /// Identifier: `b"PNG "`.
    /// Contains a PNG image.
    /// This is a picture resource chunk.
    Png{data: Vec<u8>},

    /// Identifier: `b"JPEG"`.
    /// Contains a JPEG image.
    /// This is a picture resource chunk.
    Jpeg{data: Vec<u8>},
}
