# Foxtive Ntex Multipart
This is a simple multipart parser built for [Ntex.rs](https://github.com/ntex-rs/ntex)

A powerful and flexible multipart form data parser with support for:
- **File uploads** with validation (size, type, count)
- **Custom types** that implement `FromStr`
- **Optional UUID support** via feature flags
- **Type-safe parsing** for all primitive types
- **Option<T> support** for optional fields
- **Comprehensive error handling**

---

## Installation

### Basic Installation
```bash
cargo add foxtive-ntex-multipart
```

### With UUID Support
```bash
cargo add foxtive-ntex-multipart --features uuid
```

### Cargo.toml Configuration
```toml
[dependencies]
# Basic installation
foxtive-ntex-multipart = "0.3"

# With UUID support
foxtive-ntex-multipart = { version = "0.3", features = ["uuid"] }
```

## Features

### Optional Features
- **`uuid`** - Enables support for parsing `uuid::Uuid` from multipart data

## Usage

### Basic Type Parsing
```rust
use foxtive_ntex_multipart::Multipart;

// Parse various types from multipart data
let user_id: i32 = multipart.post("user_id")?;
let username: String = multipart.post("username")?;
let is_active: bool = multipart.post("is_active")?;
let price: f64 = multipart.post("price")?;
```

### Optional Fields
```rust
// Optional fields return None if missing or empty
let optional_age: Option<i32> = multipart.post("age")?;
let optional_email: Option<String> = multipart.post("email")?;
```

### Default Values
```rust
// Provide default values for missing fields
let priority = multipart.post_or("priority", false);
let timeout = multipart.post_or("timeout", 30);
```

### UUID Support (with `uuid` feature)
```rust
use uuid::Uuid;

// Parse UUIDs from multipart data
let user_id: Uuid = multipart.post("user_id")?;
let optional_session: Option<Uuid> = multipart.post("session_id")?;
let default_id = multipart.post_or("missing_id", Uuid::new_v4());
```

### Custom Types
```rust
use foxtive_ntex_multipart::impl_post_parseable_for_custom_type;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
struct UserId(u64);

impl FromStr for UserId {
    type Err = std::num::ParseIntError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UserId(s.parse()?))
    }
}

// Enable multipart parsing for your custom type
impl_post_parseable_for_custom_type!(UserId);

// Now you can use it in multipart parsing
let user_id: UserId = multipart.post("user_id")?;
let optional_id: Option<UserId> = multipart.post("optional_id")?;
```

### Supported Types

The library automatically supports all types that implement `FromStr`:

**Primitive Types:**
- Integers: `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`
- Floats: `f32`, `f64`
- Other: `bool`, `char`, `String`

**Standard Library Types:**
- Network: `IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `SocketAddrV4`, `SocketAddrV6`
- Path: `PathBuf`
- NonZero: All `NonZero*` integer types

**Optional Types:**
- `uuid::Uuid` (with `uuid` feature)

**Custom Types:**
- Any type implementing `FromStr` via the `impl_post_parseable_for_custom_type!` macro


## 🙌 Contributing
Contributions, bug reports, and feature requests are welcome! Feel free to open issues or PRs.

## License
This project is licensed under the MIT License.


