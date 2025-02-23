# tylerjw.dev

[![pre-commit.ci status](https://results.pre-commit.ci/badge/github/tylerjw/tylerjw.dev/main.svg)](https://results.pre-commit.ci/latest/github/tylerjw/tylerjw.dev/main)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/tylerjw/tylerjw.dev/deploy.yaml?label=Build%20and%20Deploy)](https://github.com/tylerjw/tylerjw.dev/actions/workflows/deploy.yaml)

## Env Setup

### Static Site (blog)

To build locally [install zola](https://www.getzola.org/documentation/getting-started/installation/) then run:

```bash
zola serve
```

### Slides

apt dependencies
```bash
sudo apt install inkscape inotify-tools
```

Python
0. Create virtual environment
```bash
python3 -m venv .venv
```
1. Activate virtual environment
```bash
source .venv/bin/activate
```
2. Prepare pip
```bash
python3 -m pip install --upgrade pip
```
3. Install dependencies
```bash
python3 -m pip install -r slides/requirements.txt
```

## Generate QR codes for slides

Use [qrcode-monkey](https://www.qrcode-monkey.com/).
