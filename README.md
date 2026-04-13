# Nari - Reverse Tethering VPN 🚀

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=flat&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Android](https://img.shields.io/badge/Android-3DDC84?style=flat&logo=android&logoColor=white)](https://developer.android.com/)
[![Download](https://img.shields.io/badge/Download-APK-red.svg?style=flat&logo=android)](./Nari-Vpn-App.apk)

Nari is a cutting-edge reverse tethering framework that perfectly tunnels an Android device's network traffic through a connected Desktop PC over a USB cable using ADB. It comprises a lightweight, highly immersive Android VPN client and an ultra-fast, memory-safe asynchronous Rust proxy server.

![Nari App Preview](./android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png)

## 🏗️ Architecture

1. **Android Client (`android/`)**: Built in Kotlin. Uses `VpnService` to capture all outgoing `0.0.0.0/0` device traffic via a `tun` interface. It then ships raw IPv4 packets through an ADB port-bridge to the PC.
2. **Desktop Relay (`relay_rust/`)**: An async `tokio` driven server running on Windows/Mac/Linux. It unwraps the IPv4 payloads, establishes transparent TCP connections from your PC to the remote server, and channels the data perfectly backwards.
3. **UDP DNS Interceptor**: Nari natively intercepts fragmented DNS requests on Port 53, translates them via the desktop's DNS configurations, and emulates the response.

---

## 🏃 Getting Started

### 1. Prerequisites
- [Rust Toolchain (Cargo)](https://rustup.rs/)
- [Android Studio / ADB](https://developer.android.com/studio) installed and in your environment PATH.
- USB Debugging enabled on your Android Developer Options.

### 2. Start the Desktop Proxy
Connect the Android device to your computer via USB. Open a terminal and run the Rust server:
```sh
cd relay_rust
cargo run --release
```
Leave this terminal window open. The server will bind to `0.0.0.0:4242`.

### 3. Establish the ADB Bridge
Open another terminal array and bridge your device's internal localhost:
```sh
adb reverse tcp:4242 tcp:4242
```

### 4. Install & Run Android App
**Fastest Method:** 
You can directly download and install the pre-compiled APK provided in the root directory.
```sh
adb install Nari-Vpn-App.apk
```

**Manual Compilation:** 
Alternatively, import the `android/` directory into Android Studio or compile it via Gradle:
```sh
cd android
./gradlew installDebug
```
Open the **Nari** app on your phone, tap the centralized power shield to activate, and your device will instantly start browsing securely using your host PC's connection!

---

## 📄 License
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
