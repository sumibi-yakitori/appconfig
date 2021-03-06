use serde::{de::DeserializeOwned, Serialize};
use std::{error::Error, marker::PhantomData, path::PathBuf};

// pub trait AppConfig: Sized + Serialize + DeserializeOwned + Default {}

pub struct AppConfigManager<'a, T: Sized + Serialize + DeserializeOwned + Default> {
  organization_name: &'a str,
  app_name: &'a str,
  auto_recovery: bool,
  // options: AppConfigManagerOptions<'a>,
  _marker: PhantomData<fn() -> T>,
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct AppConfigManagerOptions<'a> { }

impl<'a, T: Sized + Serialize + DeserializeOwned + Default> AppConfigManager<'a, T> {
  pub fn new(organization_name: &'a str) -> Self {
    Self {
      organization_name,
      app_name: std::env!("CARGO_CRATE_NAME"),
      auto_recovery: true,
      _marker: Default::default(),
    }
  }

  pub fn set_auto_recovery(&mut self, value: bool) -> &mut Self {
    self.auto_recovery = value;
    self
  }

  pub fn with_auto_recovery(mut self, value: bool) -> Self {
    self.set_auto_recovery(value);
    self
  }

  pub fn set_organization_name(&mut self, value: &'a str) -> &mut Self {
    self.organization_name = value;
    self
  }

  pub fn with_organization_name(mut self, value: &'a str) -> Self {
    self.set_organization_name(value);
    self
  }

  pub fn set_app_name(&mut self, value: &'a str) -> &mut Self {
    self.app_name = value;
    self
  }

  pub fn with_app_name(mut self, value: &'a str) -> Self {
    self.set_app_name(value);
    self
  }

  pub fn load(&self) -> Result<T, Box<dyn Error>> {
    if self.auto_recovery {
      let path = self.get_user_config_path()?;
      if let Ok(value) = std::fs::read_to_string(&path) {
        Ok(toml::from_str(&value).unwrap_or(T::default()))
      } else {
        Ok(T::default())
      }
    } else {
      let path = self.get_user_config_path()?;
      let value = std::fs::read_to_string(&path)?;
      Ok(toml::from_str(&value)?)
    }
  }

  pub fn save(&self, app_config: &T) -> Result<(), Box<dyn Error>> {
    let path = self.get_user_config_path()?;
    let toml = toml::to_string(app_config)?;
    std::fs::write(&path, &toml.as_bytes())?;
    Ok(())
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

#[cfg(test)]
mod tests {
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
    let config1 = MyAppConfig::default();
    let manager = AppConfigManager::<MyAppConfig>::new("sumibi-yakitori");
    let config2 = manager.load().unwrap();
    assert_eq!(config1, config2);
    manager.save(&config2).unwrap();
  }
}
