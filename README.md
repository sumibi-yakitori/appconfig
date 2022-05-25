[![Crates.io](https://img.shields.io/crates/v/appconfig.svg)](https://crates.io/crates/appconfig)
[![Documentation](https://docs.rs/appconfig/badge.svg)](https://docs.rs/appconfig)
[![License](https://img.shields.io/crates/l/appconfig.svg)](LICENSE)
[![Workflow Status](https://github.com/sumibi-yakitori/appconfig/workflows/Rust/badge.svg)](https://github.com/sumibi-yakitori/appconfig/actions?query=workflow%3A%22Rust%22)

# appconfig

A simple configuration file manager for desktop applications.

The configuration file is read from and written to the following locations.

|Platform | Value                                    | Example                                  |
| ------- | ---------------------------------------- | ---------------------------------------- |
| Linux   | `$XDG_DATA_HOME` or `$HOME`/.local/share | /home/alice/.local/share                 |
| macOS   | `$HOME`/Library/Application Support      | /Users/Alice/Library/Application Support |
| Windows | `{FOLDERID_LocalAppData}`                | C:\Users\Alice\AppData\Local             |

## Usage

```sh
cargo add appconfig serde
```

```rust
use std::{cell::RefCell, rc::Rc};
use appconfig::AppConfigManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct MyAppConfig {
  window_pos: (u32, u32),
}

impl Default for MyAppConfig {
  fn default() -> Self {
    Self {
      window_pos: (320, 280),
    }
  }
}

fn main() {
  let config = Rc::from(RefCell::from(MyAppConfig::default()));
  let manager = AppConfigManager::new(
    config.clone(),
    std::env!("CARGO_CRATE_NAME"), // CRATE_BIN_NAME etc..,
    "sumibi-yakitori",
  );

  manager.save().unwrap();
  manager.load().unwrap();
  assert_eq!(*config.borrow(), MyAppConfig::default());
}
```
