# Finova Network - Bot Detection Service
# Multi-stage build for optimized production image
# Supports AI/ML models for behavior analysis and pattern detection

FROM python:3.11-slim as base

# Set environment variables
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    PIP_DISABLE_PIP_VERSION_CHECK=1 \
    DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    git \
    libpq-dev \
    libssl-dev \
    libffi-dev \
    libjpeg-dev \
    libpng-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Create app user for security
RUN groupadd -r finova && useradd -r -g finova -d /app -s /bin/bash finova

# Set work directory
WORKDIR /app

# Install Python dependencies
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Development stage
FROM base as development

# Install development dependencies
COPY requirements-dev.txt .
RUN pip install --no-cache-dir -r requirements-dev.txt

# Copy source code
COPY . .

# Change ownership to app user
RUN chown -R finova:finova /app

USER finova

EXPOSE 8000

CMD ["python", "src/main.py"]

# Production stage
FROM base as production

# Copy only production requirements
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy application code
COPY src/ ./src/
COPY models/ ./models/
COPY config/ ./config/

# Create necessary directories
RUN mkdir -p /app/logs /app/data /app/cache

# Set proper ownership and permissions
RUN chown -R finova:finova /app && \
    chmod -R 755 /app && \
    chmod -R 777 /app/logs /app/data /app/cache

# Health check
HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

USER finova

EXPOSE 8000

# Use gunicorn for production
CMD ["gunicorn", "--bind", "0.0.0.0:8000", "--workers", "4", "--worker-class", "uvicorn.workers.UvicornWorker", "--timeout", "300", "--keep-alive", "2", "--max-requests", "1000", "--max-requests-jitter", "100", "src.main:app"]

# Testing stage
FROM development as testing

# Install test dependencies
RUN pip install --no-cache-dir pytest pytest-asyncio pytest-cov

# Run tests
RUN python -m pytest tests/ -v --cov=src --cov-report=html

# Security scanning stage
FROM production as security

USER root

# Install security tools
RUN pip install --no-cache-dir bandit safety

# Run security checks
RUN bandit -r src/ -f json -o /tmp/bandit-report.json || true
RUN safety check --json --output /tmp/safety-report.json || true

USER finova
