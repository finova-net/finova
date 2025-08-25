# Finova Network - Content Analyzer AI Service Dockerfile
# Enterprise-grade, production-ready container for AI content analysis
# Supports text, image, and video content quality assessment

# Multi-stage build for optimized production image
FROM python:3.11-slim as builder

# Set build arguments
ARG BUILD_DATE
ARG VCS_REF
ARG VERSION=1.0.0

# Labels for container metadata
LABEL maintainer="Finova Network <dev@finova.network>" \
      version="${VERSION}" \
      description="Finova Network Content Analyzer AI Service" \
      org.label-schema.build-date="${BUILD_DATE}" \
      org.label-schema.vcs-ref="${VCS_REF}" \
      org.label-schema.vcs-url="https://github.com/finova-network/finova-contracts" \
      org.label-schema.schema-version="1.0"

# Set environment variables
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    DEBIAN_FRONTEND=noninteractive

# Install system dependencies and security updates
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    libpq-dev \
    libssl-dev \
    libffi-dev \
    libjpeg-dev \
    libpng-dev \
    libwebp-dev \
    ffmpeg \
    libsm6 \
    libxext6 \
    libxrender-dev \
    libglib2.0-0 \
    libgtk-3-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user for security
RUN groupadd -r finova && useradd -r -g finova finova

# Set working directory
WORKDIR /app

# Copy requirements first for better cache utilization
COPY requirements.txt ./
COPY requirements-dev.txt ./

# Install Python dependencies
RUN pip install --upgrade pip setuptools wheel && \
    pip install --no-cache-dir -r requirements.txt && \
    pip install --no-cache-dir torch torchvision torchaudio --index-url https://download.pytorch.org/whl/cpu && \
    pip install --no-cache-dir transformers accelerate datasets && \
    pip install --no-cache-dir opencv-python-headless pillow && \
    pip install --no-cache-dir scikit-learn pandas numpy && \
    pip install --no-cache-dir fastapi uvicorn gunicorn && \
    pip install --no-cache-dir prometheus-client structlog && \
    pip install --no-cache-dir redis celery && \
    pip install --no-cache-dir python-multipart aiofiles && \
    pip install --no-cache-dir httpx asyncio-mqtt && \
    rm -rf /root/.cache/pip

# Production stage
FROM python:3.11-slim as production

# Copy environment variables from builder
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    FINOVA_ENV=production \
    FINOVA_SERVICE=content-analyzer \
    WORKERS=4 \
    MAX_WORKERS=8 \
    WORKER_TIMEOUT=300 \
    WORKER_CLASS=uvicorn.workers.UvicornWorker \
    HOST=0.0.0.0 \
    PORT=8000 \
    LOG_LEVEL=info \
    PROMETHEUS_PORT=9090

# Install only runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    libssl3 \
    libffi8 \
    libjpeg62-turbo \
    libpng16-16 \
    libwebp7 \
    ffmpeg \
    libsm6 \
    libxext6 \
    libxrender1 \
    libglib2.0-0 \
    curl \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Create non-root user
RUN groupadd -r finova && useradd -r -g finova finova

# Set working directory
WORKDIR /app

# Copy Python packages from builder
COPY --from=builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --from=builder /usr/local/bin /usr/local/bin

# Create necessary directories with proper permissions
RUN mkdir -p /app/logs /app/data /app/models /app/temp /app/uploads && \
    chown -R finova:finova /app

# Copy application code
COPY --chown=finova:finova src/ ./src/
COPY --chown=finova:finova config/ ./config/
COPY --chown=finova:finova scripts/ ./scripts/
COPY --chown=finova:finova tests/ ./tests/
COPY --chown=finova:finova docker-entrypoint.sh ./
COPY --chown=finova:finova healthcheck.py ./

# Copy configuration files
COPY --chown=finova:finova pyproject.toml ./
COPY --chown=finova:finova setup.py ./
COPY --chown=finova:finova README.md ./

# Make entrypoint script executable
RUN chmod +x docker-entrypoint.sh

# Create model cache directory
ENV HF_HOME=/app/models/huggingface
ENV TRANSFORMERS_CACHE=/app/models/transformers
ENV TORCH_HOME=/app/models/torch
RUN mkdir -p $HF_HOME $TRANSFORMERS_CACHE $TORCH_HOME && \
    chown -R finova:finova /app/models

# Security: Switch to non-root user
USER finova

# Pre-download essential models (optional, comment out for faster build)
RUN python -c "
from transformers import AutoTokenizer, AutoModel
import torch

# Download content quality classifier
try:
    tokenizer = AutoTokenizer.from_pretrained('microsoft/DialoGPT-medium')
    model = AutoModel.from_pretrained('microsoft/DialoGPT-medium')
    print('Content classifier model cached successfully')
except Exception as e:
    print(f'Model caching failed: {e}')

# Download CLIP for image analysis
try:
    from transformers import CLIPProcessor, CLIPModel
    processor = CLIPProcessor.from_pretrained('openai/clip-vit-base-patch32')
    model = CLIPModel.from_pretrained('openai/clip-vit-base-patch32')
    print('CLIP model cached successfully')
except Exception as e:
    print(f'CLIP model caching failed: {e}')
"

# Expose ports
EXPOSE 8000 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD python healthcheck.py || exit 1

# Volume for persistent data
VOLUME ["/app/data", "/app/logs", "/app/models"]

# Set entrypoint
ENTRYPOINT ["./docker-entrypoint.sh"]

# Default command
CMD ["gunicorn", "src.main:app", \
     "--bind", "0.0.0.0:8000", \
     "--workers", "4", \
     "--worker-class", "uvicorn.workers.UvicornWorker", \
     "--worker-timeout", "300", \
     "--max-requests", "1000", \
     "--max-requests-jitter", "100", \
     "--preload", \
     "--access-logfile", "/app/logs/access.log", \
     "--error-logfile", "/app/logs/error.log", \
     "--log-level", "info"]

# Build hooks for CI/CD
ONBUILD COPY requirements.txt /tmp/
ONBUILD RUN pip install --no-cache-dir -r /tmp/requirements.txt

# Metadata
ARG GIT_COMMIT
ARG BUILD_DATE
ARG VERSION

LABEL git.commit="${GIT_COMMIT}" \
      build.date="${BUILD_DATE}" \
      version="${VERSION}" \
      finova.service="content-analyzer" \
      finova.tier="ai-services" \
      finova.environment="production"