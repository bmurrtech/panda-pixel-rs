<div align="center">

# <img src="assets/icon.png" alt="panda-pixel-rs Logo" width="32" height="32" style="vertical-align:middle;margin-right:8px;"> panda-pixel-rs

**Fast, 100% private, AI image compression, corrections, & conversion.**

*TinyPNG clone — a rust Red Panda alternative with bonus features!*

[![Star on GitHub](https://img.shields.io/badge/Star-on_GitHub-blue?logo=github)](https://github.com/bmurrtech/panda-pixel-rs)
[![Share this project](https://img.shields.io/badge/Share-this_project-1DA1F2?logo=share)](https://github.com/bmurrtech/panda-pixel-rs)
[![Downloads](https://img.shields.io/github/downloads/bmurrtech/panda-pixel-rs/total?color=success)](https://github.com/bmurrtech/panda-pixel-rs/releases)
[![Build Status](https://img.shields.io/github/actions/workflow/status/bmurrtech/panda-pixel-rs/ci.yml?branch=main)](https://github.com/bmurrtech/panda-pixel-rs/actions)
[![Latest Release](https://img.shields.io/github/v/release/bmurrtech/panda-pixel-rs?label=release)](https://github.com/bmurrtech/panda-pixel-rs/releases)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-blue.svg)](https://www.rust-lang.org/)
[![Built with Rust](https://img.shields.io/badge/built_with-Rust-orange.svg)](https://www.rust-lang.org/)
[![UI - Tauri v2](https://img.shields.io/badge/ui-Tauri_v2-FFC131.svg)](https://v2.tauri.app/)
[![Frontend - Leptos](https://img.shields.io/badge/frontend-Leptos-red.svg)](https://leptos.dev/)
[![License - GNU AGPLv3](https://img.shields.io/badge/license-GNU%20AGPLv3-blue.svg)](LICENSE)
[![Ko-fi](https://img.shields.io/badge/Ko--fi-Support-FF5E5B?logo=ko-fi)](https://ko-fi.com/bmurrtech)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Support-FFDD00?logo=buymeacoffee&logoColor=black)](https://www.buymeacoffee.com/bmurrtech)

<p align="center">
  <picture>
    <source srcset="assets/UI_1.webp?v=2" type="image/webp">
    <img src="assets/UI_1.webp?v=2" width="700" alt="panda-pixel-rs Screenshot">
  </picture>
  <br><br>
  <picture>
    <source srcset="assets/UI_2.webp?v=2" type="image/webp">
    <img src="assets/UI_2.webp?v=2" width="700" alt="panda-pixel-rs Screenshot 2">
  </picture>
</p>

</div>

## 💡 About

Panda Pixel is a local-first desktop image compression, conversion, and correction tool that performs all processing on your machine. It provides TinyPNG-like compression quality with complete privacy and offline operation.

- **Target Platforms**: macOS-first, Windows, Linux
- **Architecture**: Native desktop application (no Electron overhead)
- **Privacy**: 100% local processing - no uploads to external servers
- **Performance**: Native Rust performance with parallel processing

## ✨ Features

- 🚀 **Fast**: Native Rust performance with parallel processing
- 🔒 **Private**: 100% local processing - no uploads to external servers
- 🎨 **Multiple Formats**: PNG, JPEG, WebP, AVIF, TIFF, BMP, ICO, HEIC support
- 💡 **Smart Compression**: TinyPNG-like PNG quantization + oxipng optimization
- 🔄 **Format Conversion**: Convert between all supported formats while maintaining quality
- 🖥️ **Desktop App**: Native desktop application for macOS, Windows, and Linux
- ⚡ **Modern UI**: Built with Leptos for reactive, type-safe frontend
- 📦 **Single Binary**: Standalone desktop app with no dependencies
- 🎯 **Batch Processing**: Compress multiple images with progress tracking
- 🖱️ **Drag & Drop**: Native file drag & drop support
- ⚙️ **Advanced Options**: oxipng optimization, PNG lossy compression

## 🚀 Why Panda Pixel?

Most image compression tools compromise on privacy, performance, or cost:

- **Privacy**: Many tools upload your images to cloud servers
- **Cost**: Cloud services charge per image or have usage limits
- **Performance**: Web-based tools are slower and require internet
- **Transparency**: Closed-source tools don't reveal their algorithms

Panda Pixel eliminates these compromises:

- **Local Processing**: All compression happens on your device
- **Free & Open Source**: No usage limits or subscription fees
- **Fast**: Native Rust performance with parallel processing
- **Transparent**: Fully open source, auditable codebase


## 💻 Usage

### Desktop Application

1. **Select Images**: Click "📁 Select Images" or drag & drop files onto the window
2. **Adjust Compression**: Use the slider to choose quality:
   - **Low**: Best quality (70-90 range)
   - **Mid**: Balanced (50-80 range) - *recommended*
   - **Max**: Smallest file (20-60 range)
3. **Choose Format**: Select output format (Auto, PNG, JPEG, WebP, AVIF, TIFF, BMP, ICO)
4. **Advanced Options** (optional):
   - Enable oxipng optimization
   - Enable PNG lossy compression
5. **Compress/Convert**: Click "Compress/Convert" to process images
6. **Download**: Click a "Save" option to download compressed images to a folder

## 🧬 Supported Formats

| Input | Output | Notes |
|-------|--------|-------|
| PNG | PNG, WebP, AVIF, JPEG, TIFF, BMP, ICO | TinyPNG-like quantization |
| JPEG | JPEG, WebP, AVIF, PNG, TIFF, BMP, ICO | mozjpeg optimization |
| HEIC/HEIF | JPEG | Auto-converts like TinyPNG |
| WebP | All formats | Full decode/re-encode |
| TIFF, BMP | All formats | Standard image processing |

## 🚀 Performance

- **PNG**: Uses libimagequant for TinyPNG-like compression + oxipng optimization
- **JPEG**: mozjpeg encoder with progressive mode and trellis quantization
- **WebP**: High-quality lossy encoding optimized for web
- **AVIF**: Modern format with superior compression ratios
- **Parallel Processing**: Automatic CPU detection for optimal performance

## ⚡ Quick Start

### Download Latest Release (Recommended)

Download the latest prebuilt binaries from [GitHub Releases](https://github.com/bmurrtech/panda-pixel-rs/releases).

<p align="left">
  <img src="assets/release_assets.png" alt="GitHub release assets list" width="760">
</p>

1. Open the latest release page: [panda-pixel-rs releases](https://github.com/bmurrtech/panda-pixel-rs/releases).
2. Click the **Assets** arrow to expand downloadable files.
3. Download the file for your OS (`aarch64`, `x86_64`, or `universal` on macOS).
4. Install/open the app.

<details>
<summary>🔎 <strong>Click to expand for help with unsigned app warnings & installation issues</strong> ⚠️</summary>

<br/>

### macOS: Handling Unsigned App Warnings

Due to the app being unsigned or untested developer, you will likely see a security warning from Apple/Windows when launching Panda Pixel. This is normal for open source that hasn't gone through Apple's notarization process. **Proceed at your own risk.**

#### How to Open Panda Pixel on macOS:

1. **Double-click the downloaded `.dmg` and drag Panda Pixel into your Applications folder.**

2. **Launch the app:**
   - Open your Applications folder and double-click "Panda Pixel".
   - If you see a "Can't be opened" security message, click the small **“?”** icon in the dialog.

   <p align="left">
     <img src="assets/AppleSecurity_Help.png" alt="Apple help icon" width="400">
   </p>

3. **In the message dialog, click "Open Privacy & Security Settings for me".**
   - This takes you directly to the correct settings panel.

   <p align="left">
     <img src="assets/AppleSecurity_Nav.png" alt="Open Privacy & Security Settings" width="400">
   </p>

4. **Scroll down in Security & Privacy settings until you see a section mentioning "Panda Pixel.app".**
   - Click the **Open Anyway** button to bypass the security block (you will only need to do this once per version).

   <p align="left">
     <img src="assets/AppleSecurity_Allow.png" alt="Allow app from unidentified developer" width="400">
   </p>

5. Confirm any prompts as needed. Panda Pixel should now launch.

> _This warning appears because the app is unsigned (not notarized by Apple). This is expected for alpha/test releases. Proceed at your own discretion and risk._

---

#### Windows: SmartScreen Warning

- If you see a Windows SmartScreen warning, click **More info**, then **Run anyway**.

These prompts are expected for unsigned alpha builds and do not indicate a problem with the app.

</details>


## 🛠️ Build from Source

If your OS or architecture is not covered by release binaries, follow [docs/build.md](docs/build.md) for step-by-step build instructions.

## 🩹 Troubleshooting

Encountering issues during install or build? Check out [docs/troubleshooting.md](docs/troubleshooting.md) for solutions, common pitfalls, and environment-specific workarounds.

## 🛣️ Roadmap

- Local, Private AI-powered Image Enhancement
- Local, Private AI-Powered Background Removal
- Vector Output & SVG Creation

_Planned and exploratory work—see [docs/roadmap.md](docs/roadmap.md) for details._


## ❤️ Support Open Source

Panda Pixel is developed and maintained with love for privacy, accessibility, and open technology. If you find this project valuable, please consider:

- ⭐ **Starring the repo** to help others discover it  [![Star on GitHub](https://img.shields.io/badge/Star-on_GitHub-blue?logo=github)](https://github.com/bmurrtech/panda-pixel-rs)
- 🐼 **Sharing with friends** and creators who care about image quality & privacy [![Share this project](https://img.shields.io/badge/Share-this_project-1DA1F2?logo=share)](https://github.com/bmurrtech/panda-pixel-rs)
- 📝 **[Open an Issue](https://github.com/bmurrtech/panda-pixel-rs/issues/new/choose)** or **[Submit a Pull Request](https://github.com/bmurrtech/panda-pixel-rs/compare)**—contributions, feedback, and ideas are always welcome!
- ☕ **[Buying me a coffee](https://www.buymeacoffee.com/bmurrtech)**, or **[Ko-fi](https://ko-fi.com/bmurrtech)** for a little developer fuel  

<p align="center">

  <a href="https://www.buymeacoffee.com/bmurrtech" target="_blank" rel="noopener">
    <img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-Support%20the%20project-FFDD00?style=for-the-badge&logo=buymeacoffee&logoColor=black" alt="Buy Me a Coffee">
  </a>

  <a href="https://ko-fi.com/bmurrtech" target="_blank" rel="noopener">
    <img src="https://img.shields.io/badge/Ko--fi-Support%20the%20project-FF5E5B?style=for-the-badge&logo=ko-fi&logoColor=white" alt="Ko-fi">
  </a>

</p>

## 🤝 Contributing

Please see [docs/contributing.md](docs/contributing.md) for setup, layout, web vs desktop, and PR expectations. See [docs/testing.md](docs/testing.md) for `cargo test`, Clippy/fmt, WASM checks, and smoke workflows.

## ⭐ Acknowledgments

- **[libimagequant](https://pngquant.org/lib/)**: For TinyPNG-like PNG quantization
- **[oxipng](https://github.com/shssoichiro/oxipng)**: For PNG optimization
- **[mozjpeg](https://github.com/mozilla/mozjpeg)**: For JPEG compression
- **[Leptos](https://leptos.dev/)**: For Rust-compatible, reactive, type-safe frontend framework
- **[Trunk](https://trunkrs.dev/)**: For building, bundling, & shipping of Rust WASM applications to the web
- **[Tauri](https://tauri.app/)**: For Rust-based lightweight desktop application framework
- **[ort](https://ort.pyke.io/)**: ort is an open-source Rust binding for ONNX Runtime.
- **[APISR: Anime Production Inspired Real-World Anime Super-Resolution](https://github.com/Kiteretsu77/APISR)**: For APISR SISR models
- **[BiRefNet](https://github.com/ZhengPeng7/BiRefNet)**: For high-resolution AI background removal
- **[ONNX Runtime](https://onnxruntime.ai/)**: The Open Neural Network Exchange (ONNX) is an open standard for representing machine learning models.
- **[Optimum ONNX](https://github.com/huggingface/optimum-onnx)**: Enables exporting models to ONNX and running inference with ONNX Runtime.
- **[onnxruntime-web](https://www.npmjs.com/package/onnxruntime-web)**: Enables WebGPU-accelerated ONNX inference in browsers—just set `device: 'webgpu'`.
- **[Hugging Face ONNX Community](https://huggingface.co/onnx-community)**: For a growing library of ONNX format models.
- **[Xenova](https://huggingface.co/Xenova)**: For ONNX upscaling models and pioneering `Transformers.js`
- **[Hugging Face Transformers.js](https://github.com/huggingface/transformers.js)**: For powerful WebGPU-accelerated model support (WebGL successor).

## 📄 License

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0.html)

This repository is licensed under the [GNU Affero General Public License v3.0 (AGPL-3.0)](LICENSE). The badge links to the official license text; the [`LICENSE`](LICENSE) file in this repo is the binding copy for this project.

### License permissions and restrictions

*Keeping open-source open.*

| Use case | Permitted | Notes / conditions |
|----------|-----------|-------------------|
| Private / internal use | ✅ | No obligation to publish changes if you do not distribute or offer the software as a network service to others. |
| Modify for own private use | ✅ | Same as above; obligations attach when you distribute or run modified code as a networked service for users. |
| Share / distribute (unmodified) | ✅ | Must include AGPL license and corresponding source (or a written offer compliant with AGPL). |
| Distribute with modifications | ✅ | Modified source must be available under AGPL-3.0 to recipients. |
| Provide as SaaS / network service | ✅ | AGPL-3.0 requires that users interacting with your modified version over a network can obtain the complete corresponding source. |
| Closed / proprietary redistribution | ❌ | Public distribution or SaaS without full corresponding source under AGPL terms is not allowed. |
| Restricting access to source for users you serve | ❌ | Network users must be able to get the complete corresponding source as AGPL defines. |
| Hosted service without source sharing | ❌ | Must provide source to network users as required by AGPL-3.0 (remote network interaction). |
| Sublicensing under more restrictive terms | ❌ | AGPL terms must flow through; you cannot strip copyleft. |

### Commercial / proprietary licensing

If you need a **proprietary** or **non-AGPL** license (for example commercial use without copyleft, closed-source distribution, or SaaS without the AGPL source-offer obligations), **commercial licenses may be available.** Open a [GitHub Issue](https://github.com/bmurrtech/panda-pixel-rs/issues/new/choose) to discuss licensing and pricing.

---

## ⚠️ Disclaimer

This software is provided **as-is**, without warranty of any kind, express or implied. The author(s) are **not liable for any damages or losses** arising from the use of this project, including but not limited to data loss, system problems, or legal issues.

By using this software, **you accept full responsibility** for its use and agree to comply with the license terms and any **terms of service or usage restrictions of bundled or accessible dependencies** (for example, commercial vs. non-commercial use limitations of models such as BiRefNet or BRIA RMBG).  
- **You must ensure your use of integrated models and libraries (e.g., BiRefNet, RMBG-1.4, ONNX models) complies with their specific licenses and terms of use.**
- Some models are restricted to **personal/non-commercial use only**; using them for commercial purposes may violate upstream terms and is your sole responsibility.

See each dependency’s repository or documentation for complete licensing information and terms of use.