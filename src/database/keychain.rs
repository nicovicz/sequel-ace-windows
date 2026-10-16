//! SSH password auth via SSH_ASKPASS + Keychain via `keyring` crate.
//!
//! Mac memakai `SequelAceTunnelAssistant` helper binary sebagai `SSH_ASKPASS`
//! (`SPSSHTunnel.m:635-636`) + Windows Credential Manager untuk menyimpan
//! password. Port ini:
//!
//! 1. **SSH_ASKPASS**: tulis temporary script yang echo password, set
//!    `SSH_ASKPASS` + `DISPLAY=:0` + `SSH_ASKPASS_REQUIRE=force` (OpenSSH 8.4+),
//!    lalu hapus script setelah tunnel siap. Persis mac yang menulis helper
//!    binary, tapi kita memakai script karena tidak bisa compile binary
//!    tambahan di sini.
//! 2. **Keychain**: `keyring` crate yang memakai Windows Credential Manager,
//!    macOS Keychain, atau Linux Secret Service — sesuai platform.
//!    Item name/account mengikuti `SAKeychain.name(favoriteName:id:)`
//!    (doc `audit/utility-keychain-cluster.md` §6).

/// Item keychain untuk SSH password (mac: `SAKeychain.sshName:sshAccount`,
/// doc `utility-keychain-cluster.md` §6, 223-229).
const KEYCHAIN_SERVICE: &str = "Sequel Ace";
const KEYCHAIN_SSH_PREFIX: &str = "SSH";

/// Simpan SSH password di keychain (Windows Credential Manager / macOS Keychain).
#[allow(dead_code)] // dipakai app.rs save_favorite/load_favorite/delete_favorite
pub fn save_ssh_password(
    favorite_name: &str,
    favorite_id: &str,
    password: &str,
) -> Result<(), String> {
    let entry = keyring::Entry::new(
        KEYCHAIN_SERVICE,
        &ssh_keychain_name(favorite_name, favorite_id),
    )
    .map_err(|error| format!("Could not create keychain entry: {error}"))?;
    entry
        .set_password(password)
        .map_err(|error| format!("Could not save password to keychain: {error}"))
}

/// Baca SSH password dari keychain.
#[allow(dead_code)] // dipakai app.rs load_favorite
pub fn read_ssh_password(favorite_name: &str, favorite_id: &str) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(
        KEYCHAIN_SERVICE,
        &ssh_keychain_name(favorite_name, favorite_id),
    )
    .map_err(|error| format!("Could not create keychain entry: {error}"))?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("Could not read password from keychain: {error}")),
    }
}

/// Hapus SSH password dari keychain.
#[allow(dead_code)] // dipakai app.rs delete_favorite
pub fn delete_ssh_password(favorite_name: &str, favorite_id: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(
        KEYCHAIN_SERVICE,
        &ssh_keychain_name(favorite_name, favorite_id),
    )
    .map_err(|error| format!("Could not create keychain entry: {error}"))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()), // sudah tidak ada
        Err(error) => Err(format!("Could not delete password from keychain: {error}")),
    }
}

/// Nama item keychain untuk SSH password.
/// Mac: `SAKeychain.sshName(favoriteName:id:)` = `"<favoriteName> SSH <id>"`.
fn ssh_keychain_name(favorite_name: &str, favorite_id: &str) -> String {
    format!("{KEYCHAIN_SSH_PREFIX} {favorite_name} {favorite_id}")
}

/// Simpan MySQL password di keychain.
pub fn save_mysql_password(
    favorite_name: &str,
    favorite_id: &str,
    password: &str,
) -> Result<(), String> {
    let entry = keyring::Entry::new(
        KEYCHAIN_SERVICE,
        &mysql_keychain_name(favorite_name, favorite_id),
    )
    .map_err(|error| format!("Could not create keychain entry: {error}"))?;
    entry
        .set_password(password)
        .map_err(|error| format!("Could not save password to keychain: {error}"))
}

/// Baca MySQL password dari keychain.
pub fn read_mysql_password(
    favorite_name: &str,
    favorite_id: &str,
) -> Result<Option<String>, String> {
    let entry = keyring::Entry::new(
        KEYCHAIN_SERVICE,
        &mysql_keychain_name(favorite_name, favorite_id),
    )
    .map_err(|error| format!("Could not create keychain entry: {error}"))?;
    match entry.get_password() {
        Ok(password) => Ok(Some(password)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(error) => Err(format!("Could not read password from keychain: {error}")),
    }
}

/// Hapus MySQL password dari keychain.
#[allow(dead_code)]
pub fn delete_mysql_password(favorite_name: &str, favorite_id: &str) -> Result<(), String> {
    let entry = keyring::Entry::new(
        KEYCHAIN_SERVICE,
        &mysql_keychain_name(favorite_name, favorite_id),
    )
    .map_err(|error| format!("Could not create keychain entry: {error}"))?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(error) => Err(format!("Could not delete password from keychain: {error}")),
    }
}

/// Nama item keychain untuk MySQL password.
fn mysql_keychain_name(favorite_name: &str, favorite_id: &str) -> String {
    format!("{favorite_name} {favorite_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keychain_name_format_matches_mac() {
        // Mac: SAKeychain.sshName("MyFavorite", "abc-123") = "SSH MyFavorite abc-123"
        assert_eq!(
            ssh_keychain_name("MyFavorite", "abc-123"),
            "SSH MyFavorite abc-123"
        );
    }

    #[test]
    fn mysql_keychain_name_format() {
        assert_eq!(
            mysql_keychain_name("MyFavorite", "abc-123"),
            "MyFavorite abc-123"
        );
    }
}
