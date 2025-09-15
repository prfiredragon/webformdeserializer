// In your `oursistem` or shared library crate.
pub trait WebFomData: Sized {
    /// Deserializes a struct from a `Vec<(String, String)>`.
    fn deserialize(data: &Vec<(String, String)>) -> Result<Self, String>;
}

// Re-export the macro from the other crate
pub use webformd_macros::WebformDeserialize;