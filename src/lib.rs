//! A simple configuration file manager for desktop applications.
//!
//! The configuration file is read from and written to the following locations.
//!
//! |Platform | Value                                    | Example                                  |
//! | ------- | ---------------------------------------- | ---------------------------------------- |
//! | Linux   | `$XDG_DATA_HOME` or `$HOME`/.local/share | /home/alice/.local/share                 |
//! | macOS   | `$HOME`/Library/Application Support      | /Users/Alice/Library/Application Support |
//! | Windows | `{FOLDERID_LocalAppData}`                | C:\Users\Alice\AppData\Local             |
//!
//! # Usage
//!
//! ```sh
//! cargo add appconfig
//! ```
//!
//! ```rust
//! use std::{cell::RefCell, rc::Rc};
//! use appconfig::AppConfigManager;
//! use appconfig::serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, Serialize, Deserialize, PartialEq)]
//! struct MyAppConfig {
//!   window_pos: (u32, u32),
//! }
//!
//! impl Default for MyAppConfig {
//!   fn default() -> Self {
//!     Self {
//!       window_pos: (320, 280),
//!     }
//!   }
//! }
//!
//! fn main() {
//!   let config = Rc::from(RefCell::from(MyAppConfig::default()));
//!   let manager = AppConfigManager::new(
//!     config.clone(),
//!     std::env!("CARGO_CRATE_NAME"), // CRATE_BIN_NAME etc..,
//!     "sumibi-yakitori",
//!   );
//!
//!   manager.save().unwrap();
//!   manager.load().unwrap();
//!   assert_eq!(*config.borrow(), MyAppConfig::default());
//! }
//! ```

pub use serde;
use serde::{de::DeserializeOwned, Serialize};
use std::{cell::RefCell, error::Error, ops::Deref, path::PathBuf, rc::Rc};

/// A manager that manages a single configuration file.
///
/// By default, the configuration file will be saved automatically when the manager is dropped.
/// The name of the folder where the configuration file will be saved will be the FQDN consisting of the specified organization name and application name.
///
/// e.g.
/// `com.{organization_name}.{app_name}/app_config.toml`
pub struct AppConfigManager<T>
where
  T: Sized + Serialize + DeserializeOwned,
{
  data: Rc<RefCell<T>>,
  organization_name: String,
  app_name: String,
  skip_parsing_error_when_loading: bool,
  auto_saving: bool,
}

impl<T> AppConfigManager<T>
where
  T: Sized + Serialize + DeserializeOwned,
{
  pub fn new(
    data: Rc<RefCell<T>>,
    app_name: impl Into<String>,
    organization_name: impl Into<String>,
  ) -> Self {
    Self {
      data,
      organization_name: organization_name.into(),
      app_name: app_name.into(),
      auto_saving: true,
      skip_parsing_error_when_loading: true,
    }
  }

  pub fn set_skip_parsing_error_when_loading(&mut self, value: bool) -> &mut Self {
    self.skip_parsing_error_when_loading = value;
    self
  }

  pub fn with_skip_parsing_error_when_loading(mut self, value: bool) -> Self {
    self.set_skip_parsing_error_when_loading(value);
    self
  }

  pub fn set_auto_saving(&mut self, value: bool) -> &mut Self {
    self.auto_saving = value;
    self
  }

  pub fn with_auto_saving(mut self, value: bool) -> Self {
    self.set_auto_saving(value);
    self
  }

  pub fn set_organization_name(&mut self, value: impl Into<String>) -> &mut Self {
    self.organization_name = value.into();
    self
  }

  pub fn with_organization_name(mut self, value: impl Into<String>) -> Self {
    self.set_organization_name(value);
    self
  }

  pub fn set_app_name(&mut self, value: impl Into<String>) -> &mut Self {
    self.app_name = value.into();
    self
  }

  pub fn with_app_name(mut self, value: impl Into<String>) -> Self {
    self.set_app_name(value);
    self
  }

  pub fn load(&self) -> Result<(), Box<dyn Error>> {
    let path = self.get_user_config_path()?;
    let s = std::fs::read_to_string(&path)?;
    if self.skip_parsing_error_when_loading {
      if let Ok(value) = toml::from_str(&s) {
        *self.data.as_ref().borrow_mut() = value;
      }
    } else {
      *self.data.as_ref().borrow_mut() = toml::from_str(&s)?;
    }
    Ok(())
  }

  pub fn save(&self) -> Result<(), Box<dyn Error>> {
    let path = self.get_user_config_path()?;
    let toml = toml::to_string(&*self.data.as_ref().borrow())?;
    std::fs::write(&path, &toml.as_bytes())?;
    Ok(())
  }

  pub fn data(&self) -> &RefCell<T> {
    &self.data
  }

  fn get_user_config_path(&self) -> Result<PathBuf, Box<dyn Error>> {
    use std::io;
    let mut path = dirs_next::config_dir()
      // TODO:
      .ok_or(io::Error::new(io::ErrorKind::NotFound, "Config path"))?
      .join(&format!("com.{}.{}", self.organization_name, self.app_name));

    if !path.exists() {
      std::fs::create_dir_all(&path)?;
    }
    path = path.join("app_config.toml");
    Ok(path)
  }
}

impl<T> Deref for AppConfigManager<T>
where
  T: Sized + Serialize + DeserializeOwned,
{
  type Target = RefCell<T>;

  fn deref(&self) -> &Self::Target {
    self.data()
  }
}

impl<T> Drop for AppConfigManager<T>
where
  T: Sized + Serialize + DeserializeOwned,
{
  fn drop(&mut self) {
    if self.auto_saving {
      self.save().ok();
    }
  }
}

#[cfg(test)]
mod tests {
  use std::{cell::RefCell, rc::Rc};

  use crate::AppConfigManager;
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

  #[test]
  fn it_works() {
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
}
