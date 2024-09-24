use std::path::{Path, PathBuf};
use tabled::builder::Builder;
use tabled::object::Columns;
use tabled::{Modify, Width};
use directories::UserDirs;
use std::fs;
fn main() {
  // Kullanıcı dizinini al
  let user_dirs = UserDirs::new().expect("Kullanıcı dizinleri alınamadı.");
  let home_dir = user_dirs.home_dir();

  let directories = [
      home_dir.join("AppData\\Local\\BraveSoftware\\Brave-Browser"),
      home_dir.join("AppData\\Local\\Microsoft\\Edge"),
      home_dir.join("AppData\\Roaming\\Opera Software\\Opera GX Stable"),
      home_dir.join("AppData\\Roaming\\Opera Software\\Opera Stable"),
      home_dir.join("AppData\\Roaming\\Opera Software\\Opera Neon"),
      home_dir.join("AppData\\Local\\Google\\Chrome"),
  ];

  // Mevcut dizinleri saklamak için bir vektör
  let mut existing_dirs: Vec<PathBuf> = Vec::new();

  for dir in &directories {
      if dir.exists() {
          existing_dirs.push(dir.clone()); // Mevcut dizini vektöre ekle
      } 
  }

  // Her dizindeki Local State ve Login Data dosyalarını ara
  for chrome_dir in existing_dirs {
      let browser_name = chrome_dir.file_name().unwrap().to_string_lossy();

      if let Err(e) = find_local_state_and_login_data(&chrome_dir, &browser_name) {
          eprintln!("Hata: {}", e);
      }
  }
}

fn find_local_state_and_login_data(dir: &PathBuf, browser_name: &str) -> std::io::Result<()> {
  let mut local_state_path: Option<PathBuf> = None;

  // Dizin içindeki dosyaları oku
  for entry in fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();

      if path.is_dir() {
          // Eğer bir dizin ise, recursive olarak ara
          find_local_state_and_login_data(&path, browser_name)?;
      } else if path.is_file() {
          if path.file_name().map_or(false, |f| f == "Local State") {
              // Local State dosyasını bul
              local_state_path = Some(path.clone());
             // println!("{} - Bulunan Local State: {:?}", browser_name, path);
          }
      }
  }

  // Eğer Local State dosyası bulunduysa, Login Data dosyalarını ara
  if let Some(local_state) = local_state_path {
      let login_data_dir = local_state.parent().unwrap();

      // Login Data dosyalarını bul
      find_login_data(login_data_dir, browser_name, &local_state)?;
  }

  Ok(())
}

fn find_login_data(dir: &Path, browser_name: &str, local_state: &PathBuf) -> std::io::Result<()> {
  // Dizin içindeki dosyaları oku
  for entry in fs::read_dir(dir)? {
      let entry = entry?;
      let path = entry.path();

      if path.is_dir() {
          // Eğer bir dizin ise, recursive olarak ara
          find_login_data(&path, browser_name, local_state)?;
      } else if path.is_file() && path.file_name().map_or(false, |f| f == "Login Data") {
          // Eğer Login Data dosyası ise
          //println!("{} - Bulunan Login Data: {:?}", browser_name, path);

          // Master anahtarını al
          let master_key = browser_passwords_rust::get_master_key(local_state);
          let password = browser_passwords_rust::get_password(&path, &master_key);

          // Tarayıcı adını ve parolayı yazdır
          println!("Tarayici: {}", browser_name);
          print(&password);
      }
  }

  Ok(())
}

fn print(password: &Vec<Vec<String>>) {
  let mut builder = Builder::default();
  builder.set_columns(["url", "username", "password"]);
  for p in password {
    builder.add_record(p);
  }
  let table = builder
    .build()
    .with(Modify::new(Columns::first()).with(Width::wrap(50).keep_words()));
  println!("{}", table);
}