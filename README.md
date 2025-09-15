# `webformd` - Web Form Deserializer

[!Crates.io](https://crates.io/crates/webformd)
[!docs.rs](https://docs.rs/webformd)

`webformd` is a Rust crate that provides a derive macro to deserialize web form data (`application/x-www-form-urlencoded`) into your structs.

It is designed to easily handle form data, especially in common web development scenarios like those found when using frameworks such as Actix Web. The crate can deserialize fields into `String`, `Vec<String>`, `Option<String>`, and `Option<Vec<String>>` types, making it very flexible for text inputs, checkboxes, multiple-selects, and optional fields.

## Features

*   **Easy to Use**: Simply add `#[derive(WebformDeserialize)]` to your struct.
*   **Flexible Type Handling**: Supports deserialization for:
    *   `String` (for required text fields).
    *   `Option<String>` (for optional text fields).
    *   `Vec<String>` (for checkboxes or multiple-selects with at least one option selected).
    *   `Option<Vec<String>>` (for checkboxes or multiple-selects that may have no options selected).
*   **Validation**: Returns an error if a required field (`String`) is missing.
*   **Integration**: Works seamlessly with Actix Web's form extractor (`actix_web::web::Form`).

## Installation

Add `webformd` to your `Cargo.toml` file:

```toml
[dependencies]
webformd = "0.1.0" # Replace with the latest version
```

## Usage

Define your struct and derive `WebformDeserialize` from `webformd`. Then, you can use the `deserialize` method provided by the `WebFomData` trait.

### Example

Imagine you have an HTML form like this:

```html
<form action="/submit" method="post">
  <!-- Required field -->
  <label for="name">Name:</label>
  <input type="text" id="name" name="name" required>

  <!-- Optional field -->
  <label for="email">Email (optional):</label>
  <input type="email" id="email" name="email">

  <!-- Checkboxes (multiple values) -->
  <p>Interests:</p>
  <input type="checkbox" name="interests" value="rust"> Rust
  <input type="checkbox" name="interests" value="webdev"> Web Dev
  <input type="checkbox" name="interests" value="gamedev"> Game Dev

  <button type="submit">Submit</button>
</form>
```

You can deserialize the data from this form into a Rust struct as follows:

```rust
use webformd::{WebFomData, WebformDeserialize};

#[derive(Debug, WebformDeserialize)]
struct MyForm {
    name: String,
    email: Option<String>,
    interests: Option<Vec<String>>, // Use Option<Vec<String>> because no interests might be selected
}

fn main() {
    // Simulated web form data (like what you'd get in an Actix Web handler)
    let form_data = vec![
        ("name".to_string(), "Alice".to_string()),
        ("interests".to_string(), "rust".to_string()),
        ("interests".to_string(), "webdev".to_string()),
    ];

    let my_form = MyForm::deserialize(&form_data).unwrap();

    println!("{:?}", my_form);
    // Output: MyForm { name: "Alice", email: None, interests: Some(["rust", "webdev"]) }
}
```

### With Actix Web

Integration with `actix_web::web::Form` is straightforward. The `Vec<(String, String)>` format is compatible with what `Form` extracts.

```rust
// In your Actix Web handler:
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use webformd::{WebFomData, WebformDeserialize};

#[derive(Debug, WebformDeserialize)]
struct MyForm {
    name: String,
    email: Option<String>,
    interests: Option<Vec<String>>,
}

async fn handle_form(form: web::Form<Vec<(String, String)>>) -> impl Responder {
    match MyForm::deserialize(&form.into_inner()) {
        Ok(data) => {
            println!("Deserialized form data: {:?}", data);
            HttpResponse::Ok().body(format!("Hello, {}!", data.name))
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}
```

## License

This project is licensed under the [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) license.
