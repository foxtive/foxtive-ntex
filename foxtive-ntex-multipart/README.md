# Foxtive-Ntex Multipart

Multipart form data parser for [ntex](https://ntex.rs). Handles file uploads with validation, parses primitive types, optional fields, custom types, and UUIDs.

## Install

```bash
cargo add foxtive-ntex-multipart
# or with UUID support
cargo add foxtive-ntex-multipart --features uuid
```

```toml
foxtive-ntex-multipart = "1.0"

# with UUID support
foxtive-ntex-multipart = { version = "1.0", features = ["uuid"] }
```

## Usage

### Parsing primitive types

```rust
let user_id: i32 = multipart.post("user_id")?;
let username: String = multipart.post("username")?;
let is_active: bool = multipart.post("is_active")?;
```

### Optional fields and defaults

```rust
let age: Option<i32> = multipart.post("age")?;
let priority = multipart.post_or("priority", false);
let timeout = multipart.post_or("timeout", 30);
```

### UUIDs (with `uuid` feature)

```rust
let user_id: Uuid = multipart.post("user_id")?;
let session: Option<Uuid> = multipart.post("session_id")?;
```

### Custom types

Any type that implements `FromStr`:

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

impl_post_parseable_for_custom_type!(UserId);

let user_id: UserId = multipart.post("user_id")?;
```

### Supported types

Anything that implements `FromStr` works out of the box — integers, floats, `bool`, `String`, `IpAddr`, `PathBuf`, `NonZero*` types, and `Uuid` (with the feature flag). Custom types just need the macro.

## License

MIT
