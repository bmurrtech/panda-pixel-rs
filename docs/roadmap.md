# Roadmap

Planned and exploratory work. Nothing here is a commitment or guarantee of shipping order.

---

## Vector Output & SVG Creation

- [ ] **Support PNG, JPEG, JPG, and other common raster formats as input**
- [ ] **SVG output:**
  - [ ] Option 1: Embed original raster as `<image>` in SVG
  - [ ] Option 2: Vector tracing for real SVG (auto-detect logo/lineart vs. photo)
- [ ] **Tracing Pipeline:**
  - [ ] Load image (handle transparency if present)
  - [ ] Detect subject/background (foreground extraction if useful)
  - [ ] Preprocess: denoise, threshold, color cluster as appropriate
  - [ ] Trace edges and regions to generate SVG paths
  - [ ] Simplify paths (polygons/Béziers)
  - [ ] Write SVG using `svg` crate or XML builder
- [ ] **Settings:** Toggle raster-vs-vector, tracing detail, and subject isolation
- [ ] **Test output (preview with resvg or similar)**

---

## Local, Private AI-powered Image Enhancement

- [ ] **Image upscaling**: Fast single-image super-resolution (SISR) for high-quality upscaling
  - [ ] **Real-world images**
    - [ ] 2× and 4× (e.g. [Xenova/swin2SR-classical-sr-x2-64](https://huggingface.co/Xenova/swin2SR-classical-sr-x2-64), [andrewdalpino/UltraZoom-2X](https://huggingface.co/andrewdalpino/UltraZoom-2X))
  - [ ] **APISR (anime production inspired super-resolution)**
    - [ ] 4× ([Xenova/4x_APISR_GRL_GAN_generator-onnx](https://huggingface.co/Xenova/4x_APISR_GRL_GAN_generator-onnx))
- [ ] **Resolution improvement**: AI-assisted enhancement while compressing

- [ ] **ONNX Runtime integration**: [ort](https://ort.pyke.io/) for optimized ONNX inference
  - [ ] Hardware acceleration (CUDA, TensorRT, OpenVINO, QNN, CANN)
  - [ ] Efficient I/O binding
  - [ ] Cross-platform deployment including WASM

---

## Local, private AI-powered background removal

- [ ] **Commercial-use background removal**: Local ONNX for Bilateral Reference for High-Resolution Dichotomous Image Segmentation ([BiRefNet](https://github.com/ZhengPeng7/BiRefNet))
  - [ ] Optional mask output (save mask in addition to cutout)
- [ ] **Non-commercial background removal**: Local ONNX ([briaai/RMBG-1.4](https://huggingface.co/briaai/RMBG-1.4)) for personal / non-commercial use
