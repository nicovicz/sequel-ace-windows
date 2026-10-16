//! SSH_ASKPASS helper binary.
//!
//! Dipanggil oleh `ssh` saat membutuhkan password/passphrase. Membaca password
//! dari env `SSH_ASKPASS_PASSWORD` (di-set oleh parent process saat spawn ssh)
//! dan menampilkannya ke stdout.
//!
//! Ini menggantikan temporary script file (fase 8) dengan binary yang
//! di-compile bersama proyek — password tidak pernah menyentuh disk.

fn main() {
    // ssh memanggil SSH_ASKPASS dengan prompt sebagai argv[1].
    // Kita abaikan prompt dan hanya output password dari env.
    let password = std::env::var("SSH_ASKPASS_PASSWORD").unwrap_or_default();
    println!("{password}");
}
