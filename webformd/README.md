[![Rust](https://github.com/prfiredragon/webformdeserializer/actions/workflows/rust.yml/badge.svg)](https://github.com/prfiredragon/webformdeserializer/actions/workflows/rust.yml)

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0--1.0-blue.svg)](https://creativecommons.org/publicdomain/zero/1.0/)
# `webformd`: Web Form Deserializer

`webformd` is a Rust crate that provides a derive macro to deserialize web form data (`application/x-www-form-urlencoded`) into your structs.

It is designed to easily handle form data, especially in common web development scenarios like those found when using frameworks such as Actix Web. The crate can deserialize fields into various primitive types, `String`, `Vec<T>`, and `Option<T>`, making it very flexible for all kinds of form inputs. It also supports parsing into any type that implements the `FromStr` trait.


## Features

*   **Easy to Use**: Simply add `#[derive(WebformDeserialize)]` to your struct.
*   **Flexible Type Handling**: Supports deserialization for a wide range of types, including:
    *   `String` and numeric types (`i32`, `f64`, etc.) for single value fields.
    *   `Option<T>` for optional fields.
    *   `Vec<T>` for fields that can have multiple values (like checkboxes).
    *   `Option<Vec<T>>` for optional, multiple-value fields.
*   **Validation**: Returns an error if a required field (e.g., `String`, `i32`) is missing or can't be parsed.
*   **Integration**: Works seamlessly with Actix Web's form extractor (`actix_web::web::Form`).

## Installation

Add `webformd` to your `Cargo.toml` file:

```toml
[dependencies]
webformd = "0.1.0"
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

  <!-- Required numeric field -->
  <label for="age">Age:</label>
  <input type="number" id="age" name="age" required>

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

#[derive(Debug, PartialEq, WebformDeserialize)]
struct MyForm {
    name: String, // Required by default
    age: u32,
    email: Option<String>,
    interests: Option<Vec<String>>, // Use Option<Vec<String>> because no interests might be selected
}

fn main() {
    // Simulated web form data (like what you'd get in an Actix Web handler)
    let form_data = vec![
        ("name".to_string(), "Alice".to_string()), // email is missing, so it will be None
        ("age".to_string(), "30".to_string()),
        ("interests".to_string(), "rust".to_string()),
        ("interests".to_string(), "webdev".to_string()),
    ];

    let my_form = MyForm::deserialize(&form_data).unwrap();
    
    assert_eq!(my_form, MyForm {
        name: "Alice".to_string(),
        age: 30,
        email: None,
        interests: Some(vec!["rust".to_string(), "webdev".to_string()]),
    });

    println!("Deserialized successfully: {:?}", my_form);
}
```

### Advanced Usage : `#[webformd(from_str)]`

For fields that are not `String` but implement the `FromStr` trait (like `u32`, `f64`, etc.), the `WebformDeserialize` macro will automatically try to parse them.

In the first example, the `age: u32` field is deserialized from a `String` ("30") thanks to this internal logic. If the conversion fails (for example, if "thirty" is received instead of "30"), `deserialize` will return an error.

You can also use it for `Vec<T>` where `T` implements `FromStr` by explicitly adding the `#[webformd(from_str)]` attribute.

```rust
use webformd::{WebFomData, WebformDeserialize};
use std::str::FromStr;

#[derive(Debug, PartialEq, WebformDeserialize)]
struct ProductSelection {
    // Each "product_ids" value will be parsed into a u32
    #[webformd(from_str)]
    product_ids: Vec<u32>,
    
    // This also works with custom enums
    #[webformd(from_str)]
    categories: Option<Vec<ProductCategory>>,
}

#[derive(Debug, PartialEq)]
enum ProductCategory {
    Electronics,
    Books,
    Clothing,
}

impl FromStr for ProductCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "electronics" => Ok(ProductCategory::Electronics),
            "books" => Ok(ProductCategory::Books),
            "clothing" => Ok(ProductCategory::Clothing),
            _ => Err(format!("'{}' is not a valid category", s)),
        }
    }
}

fn main() {
    let form_data = vec![
        ("product_ids".to_string(), "101".to_string()),
        ("product_ids".to_string(), "202".to_string()),
        ("categories".to_string(), "electronics".to_string()),
        ("categories".to_string(), "books".to_string()),
    ];

    let selection = ProductSelection::deserialize(&form_data).unwrap();
    assert_eq!(selection, ProductSelection { 
        product_ids: vec![101, 202],
        categories: Some(vec![ProductCategory::Electronics, ProductCategory::Books]),
    });
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
    age: u32,
    email: Option<String>,
    interests: Option<Vec<String>>,
}

async fn handle_form(form: web::Form<Vec<(String, String)>>) -> HttpResponse {
    match MyForm::deserialize(&form.into_inner()) {
        Ok(data) => {
            println!("Deserialized form data: {:?}", data);
            HttpResponse::Ok().body(format!("Hello, {}! You are {} years old.", data.name, data.age))
        }
        Err(e) => HttpResponse::BadRequest().body(e),
    }
}
```

## License

This project is licensed under the [CC0 1.0 Universal](https://creativecommons.org/publicdomain/zero/1.0/) license.
