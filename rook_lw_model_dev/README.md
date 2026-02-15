# Rook LifeWatch Model Development

Python scripts and Jupyter notebooks for YOLOv8 model modification, export, and experimentation.

## Setup

### Quick Start

```bash
# Run the setup script (installs uv, creates venv, installs dependencies)
source ./setup-env.sh

# Start JupyterLab
./start-jupyter.sh
```

## Usage

### Model Export with Embeddings

See notebooks and scripts for:
- Exporting YOLOv8 models with multiple outputs (detections + embeddings)
- Testing embedding extraction
- Visualizing model architecture
- Benchmarking inference performance

## Model Files

Model files (`.pt`, `.onnx`) are gitignored. Download or place them in this directory as needed.

Example:
```bash
# Download YOLOv8 nano model
wget https://github.com/ultralytics/assets/releases/download/v0.0.0/yolov8n.pt
```
