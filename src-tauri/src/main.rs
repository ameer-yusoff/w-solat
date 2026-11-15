#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // Benarkan WebView2 memainkan bunyi notifikasi (AudioContext) tanpa klik terdahulu
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--autoplay-policy=no-user-gesture-required",
    );
    w_solat_lib::run();
}
