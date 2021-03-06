use serde::{de::DeserializeOwned, Serialize};
use std::{cell::RefCell, error::Error, marker::PhantomData, path::PathBuf, rc::Rc};

// pub trait AppConfig: Sized + Serialize + DeserializeOwned + Default {}

pub struct AppConfigManager<T>
where
  T: Sized + Serialize + DeserializeOwned + Default,
{
  data: Rc<RefCell<T>>,
  organization_name: String,
  app_name: String,
  auto_recovery: bool,
  auto_saving: bool,
  // options: AppConfigManagerOptions<'a>,
  // _marker: PhantomData<fn() -> T>,
}

// #[derive(Debug, Clone, PartialEq)]
// pub struct AppConfigManagerOptions<'a> { }

impl<T> AppConfigManager<T>
where
  T: Sized + Serialize + DeserializeOwned + Default,
{
  pub fn new(data: Rc<RefCell<T>>, organization_name: impl Into<String>) -> Self {
    Self {
      data,
      organization_name: organization_name.into(),
      app_name: std::env!("CARGO_CRATE_NAME").into(),
      auto_recovery: true,
      auto_saving: true,
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
    *self.data.as_ref().borrow_mut() = if self.auto_recovery {
      let path = self.get_user_config_path()?;
      if let Ok(value) = std::fs::read_to_string(&path) {
        toml::from_str(&value).unwrap_or(T::default())
      } else {
        T::default()
      }
    } else {
      let path = self.get_user_config_path()?;
      let value = std::fs::read_to_string(&path)?;
      toml::from_str(&value)?
    };
    Ok(())
  }

  pub fn save(&self) -> Result<(), Box<dyn Error>> {
    let path = self.get_user_config_path()?;
    let toml = toml::to_string(&*self.data.as_ref().borrow())?;
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

impl<T> Drop for AppConfigManager<T>
where
  T: Sized + Serialize + DeserializeOwned + Default,
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
    let manager = AppConfigManager::new(config.clone(), "sumibi-yakitori");
    manager.load().unwrap();
    assert_eq!(*config.borrow(), MyAppConfig::default());
    manager.save().unwrap();
  }
}
