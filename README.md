# tylerjw.dev

[![pre-commit.ci status](https://results.pre-commit.ci/badge/github/tylerjw/tylerjw.dev/main.svg)](https://results.pre-commit.ci/latest/github/tylerjw/tylerjw.dev/main)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/tylerjw/tylerjw.dev/deploy.yaml?label=Build%20and%20Deploy)](https://github.com/tylerjw/tylerjw.dev/actions/workflows/deploy.yaml)

## Run linters

```bash
pixi run lint
```

## Static Site

To build and serve the site locally, watching for changes:
```bash
pixi run blog
```

## Slides

To watch for changes to the slides python files in the slides directory and re-run them when they change:
```bash
pixi run slides
```

## Generate QR codes for slides

Use [qrcode-monkey](https://www.qrcode-monkey.com/).

## Resizing Images

Enter the pixi shell
```bash
pixi shell
```

To resize an image to be 1080 pixels wide:
```bash
magick PXL_20250903_161814058.MP.jpg -resize 1080x PXL_20250903_161814058.MP-r.jpg
```

To get the width and height of an image:
```bash
identify -format "%wx%h\n" PXL_20250904_181454627.jpg
```

## Env Setup

Install system dependencies:
- inkscape `yay -S inkscape`
- Rust `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- pixi `curl -fsSL https://pixi.sh/install.sh | bash`
