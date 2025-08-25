"""
Finova Network - AI Content Analyzer
Image Preprocessing Module

Enterprise-grade image processing for content quality assessment,
originality detection, and engagement prediction.

Author: Finova Network Development Team
Version: 1.0.0
License: MIT
"""

import cv2
import numpy as np
import hashlib
import base64
import logging
from typing import Dict, List, Tuple, Optional, Any, Union
from PIL import Image, ImageEnhance, ImageFilter, ExifTags
from io import BytesIO
import imagehash
from sklearn.feature_extraction import image as sk_image
from sklearn.cluster import KMeans
import face_recognition
import torch
import torchvision.transforms as transforms
from transformers import CLIPProcessor, CLIPModel
import requests
from pathlib import Path
import asyncio
import aiohttp
from dataclasses import dataclass
from enum import Enum
import json
import time
from concurrent.futures import ThreadPoolExecutor
import psutil

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

class ImageQuality(Enum):
    """Image quality levels for content assessment"""
    LOW = 0.5
    MEDIUM = 1.0
    HIGH = 1.5
    PREMIUM = 2.0

class ContentType(Enum):
    """Content type classification"""
    ORIGINAL = "original"
    MEME = "meme"
    SCREENSHOT = "screenshot"
    PROMOTIONAL = "promotional"
    EDUCATIONAL = "educational"
    ENTERTAINMENT = "entertainment"
    NEWS = "news"
    PERSONAL = "personal"

@dataclass
class ImageMetadata:
    """Comprehensive image metadata structure"""
    filename: str
    size: Tuple[int, int]
    file_size: int
    format: str
    mode: str
    has_transparency: bool
    exif_data: Dict[str, Any]
    creation_time: Optional[str]
    camera_info: Optional[Dict[str, str]]
    gps_data: Optional[Dict[str, float]]
    hash_perceptual: str
    hash_difference: str
    hash_average: str
    hash_wavelet: str
    color_profile: Dict[str, Any]
    compression_quality: Optional[int]

@dataclass
class QualityMetrics:
    """Image quality assessment metrics"""
    sharpness_score: float
    brightness_score: float
    contrast_score: float
    saturation_score: float
    noise_level: float
    blur_detection: float
    resolution_score: float
    aspect_ratio_score: float
    composition_score: float
    aesthetic_score: float
    overall_quality: float
    quality_level: ImageQuality

@dataclass
class OriginalityMetrics:
    """Content originality assessment"""
    uniqueness_score: float
    duplicate_probability: float
    template_match_score: float
    watermark_detected: bool
    stock_photo_probability: float
    ai_generated_probability: float
    editing_level: float
    authenticity_score: float

@dataclass
class EngagementPrediction:
    """Predicted engagement metrics"""
    like_probability: float
    share_probability: float
    comment_probability: float
    viral_potential: float
    platform_fitness: Dict[str, float]
    optimal_posting_score: float
    target_audience_match: float

class ImageProcessor:
    """
    Advanced image processing system for Finova Network
    Handles quality assessment, originality detection, and engagement prediction
    """
    
    def __init__(self, config: Optional[Dict[str, Any]] = None):
        """Initialize the image processor with configuration"""
        self.config = config or self._get_default_config()
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        
        # Initialize AI models
        self._init_models()
        
        # Initialize processing statistics
        self.stats = {
            'processed_count': 0,
            'total_processing_time': 0.0,
            'average_processing_time': 0.0,
            'quality_distribution': {level.name: 0 for level in ImageQuality},
            'error_count': 0
        }
        
        # Thread pool for parallel processing
        self.executor = ThreadPoolExecutor(max_workers=self.config['max_workers'])
        
        logger.info(f"ImageProcessor initialized on {self.device}")

    def _get_default_config(self) -> Dict[str, Any]:
        """Get default configuration settings"""
        return {
            'max_image_size': (2048, 2048),
            'min_image_size': (64, 64),
            'supported_formats': ['JPEG', 'PNG', 'WEBP', 'BMP', 'TIFF'],
            'max_file_size': 50 * 1024 * 1024,  # 50MB
            'quality_thresholds': {
                'sharpness_min': 0.3,
                'brightness_range': (20, 230),
                'contrast_min': 0.4,
                'noise_max': 0.7
            },
            'originality_thresholds': {
                'duplicate_max': 0.15,
                'uniqueness_min': 0.6,
                'authenticity_min': 0.7
            },
            'engagement_weights': {
                'quality': 0.3,
                'originality': 0.25,
                'aesthetic': 0.2,
                'relevance': 0.15,
                'trending': 0.1
            },
            'max_workers': min(4, psutil.cpu_count()),
            'cache_enabled': True,
            'debug_mode': False
        }

    def _init_models(self):
        """Initialize AI models for image analysis"""
        try:
            # CLIP model for semantic understanding
            self.clip_model = CLIPModel.from_pretrained("openai/clip-vit-base-patch32")
            self.clip_processor = CLIPProcessor.from_pretrained("openai/clip-vit-base-patch32")
            self.clip_model.to(self.device)
            
            # Image transformation pipeline
            self.transform = transforms.Compose([
                transforms.Resize((224, 224)),
                transforms.ToTensor(),
                transforms.Normalize(mean=[0.485, 0.456, 0.406], std=[0.229, 0.224, 0.225])
            ])
            
            logger.info("AI models initialized successfully")
            
        except Exception as e:
            logger.error(f"Failed to initialize AI models: {e}")
            self.clip_model = None
            self.clip_processor = None

    async def process_image(self, image_input: Union[str, bytes, Image.Image], 
                          platform: str = "general") -> Dict[str, Any]:
        """
        Main image processing pipeline
        
        Args:
            image_input: Image file path, bytes, or PIL Image
            platform: Target social media platform
            
        Returns:
            Comprehensive analysis results
        """
        start_time = time.time()
        
        try:
            # Load and validate image
            image = await self._load_image(image_input)
            if image is None:
                raise ValueError("Failed to load image")
            
            # Extract metadata
            metadata = await self._extract_metadata(image, image_input)
            
            # Run parallel analysis
            tasks = [
                self._assess_quality(image),
                self._assess_originality(image, metadata),
                self._predict_engagement(image, platform, metadata)
            ]
            
            quality_metrics, originality_metrics, engagement_prediction = await asyncio.gather(*tasks)
            
            # Calculate composite scores
            composite_score = self._calculate_composite_score(
                quality_metrics, originality_metrics, engagement_prediction
            )
            
            # Generate recommendations
            recommendations = self._generate_recommendations(
                quality_metrics, originality_metrics, engagement_prediction, platform
            )
            
            # Update statistics
            processing_time = time.time() - start_time
            self._update_stats(processing_time, quality_metrics.quality_level)
            
            return {
                'success': True,
                'metadata': metadata,
                'quality_metrics': quality_metrics,
                'originality_metrics': originality_metrics,
                'engagement_prediction': engagement_prediction,
                'composite_score': composite_score,
                'recommendations': recommendations,
                'processing_time': processing_time,
                'platform': platform
            }
            
        except Exception as e:
            logger.error(f"Image processing failed: {e}")
            self.stats['error_count'] += 1
            return {
                'success': False,
                'error': str(e),
                'processing_time': time.time() - start_time
            }

    async def _load_image(self, image_input: Union[str, bytes, Image.Image]) -> Optional[Image.Image]:
        """Load and validate image from various input types"""
        try:
            if isinstance(image_input, Image.Image):
                image = image_input.copy()
            elif isinstance(image_input, bytes):
                image = Image.open(BytesIO(image_input))
            elif isinstance(image_input, str):
                if image_input.startswith(('http://', 'https://')):
                    async with aiohttp.ClientSession() as session:
                        async with session.get(image_input) as response:
                            image_bytes = await response.read()
                            image = Image.open(BytesIO(image_bytes))
                else:
                    image = Image.open(image_input)
            else:
                raise ValueError(f"Unsupported image input type: {type(image_input)}")
            
            # Validate image
            if not self._validate_image(image):
                return None
                
            # Convert to RGB if necessary
            if image.mode != 'RGB':
                image = image.convert('RGB')
                
            return image
            
        except Exception as e:
            logger.error(f"Failed to load image: {e}")
            return None

    def _validate_image(self, image: Image.Image) -> bool:
        """Validate image meets requirements"""
        # Check format
        if image.format not in self.config['supported_formats']:
            logger.warning(f"Unsupported format: {image.format}")
            return False
        
        # Check dimensions
        width, height = image.size
        min_w, min_h = self.config['min_image_size']
        max_w, max_h = self.config['max_image_size']
        
        if width < min_w or height < min_h:
            logger.warning(f"Image too small: {width}x{height}")
            return False
            
        if width > max_w or height > max_h:
            # Resize if too large
            image.thumbnail(self.config['max_image_size'], Image.Resampling.LANCZOS)
            
        return True

    async def _extract_metadata(self, image: Image.Image, 
                              image_input: Union[str, bytes, Image.Image]) -> ImageMetadata:
        """Extract comprehensive image metadata"""
        # Basic image info
        width, height = image.size
        file_size = len(image_input) if isinstance(image_input, bytes) else 0
        
        # EXIF data extraction
        exif_data = {}
        camera_info = {}
        gps_data = {}
        creation_time = None
        
        try:
            exif = image.getexif()
            if exif:
                for tag_id, value in exif.items():
                    tag = ExifTags.TAGS.get(tag_id, tag_id)
                    exif_data[tag] = value
                    
                    # Extract camera info
                    if tag in ['Make', 'Model', 'Software']:
                        camera_info[tag] = str(value)
                    elif tag == 'DateTime':
                        creation_time = str(value)
                    elif tag == 'GPSInfo':
                        gps_data = self._parse_gps_data(value)
        except Exception as e:
            logger.warning(f"EXIF extraction failed: {e}")

        # Generate perceptual hashes
        hash_perceptual = str(imagehash.phash(image))
        hash_difference = str(imagehash.dhash(image))
        hash_average = str(imagehash.average_hash(image))
        hash_wavelet = str(imagehash.whash(image))
        
        # Color profile analysis
        color_profile = self._analyze_color_profile(image)
        
        # Compression quality estimation
        compression_quality = self._estimate_compression_quality(image)
        
        return ImageMetadata(
            filename=getattr(image_input, 'name', 'unknown'),
            size=(width, height),
            file_size=file_size,
            format=image.format or 'Unknown',
            mode=image.mode,
            has_transparency='transparency' in image.info,
            exif_data=exif_data,
            creation_time=creation_time,
            camera_info=camera_info,
            gps_data=gps_data,
            hash_perceptual=hash_perceptual,
            hash_difference=hash_difference,
            hash_average=hash_average,
            hash_wavelet=hash_wavelet,
            color_profile=color_profile,
            compression_quality=compression_quality
        )

    def _parse_gps_data(self, gps_info: Dict) -> Dict[str, float]:
        """Parse GPS data from EXIF"""
        try:
            def convert_to_degrees(value):
                d, m, s = value
                return d + (m / 60.0) + (s / 3600.0)
            
            gps_data = {}
            if 'GPSLatitude' in gps_info:
                gps_data['latitude'] = convert_to_degrees(gps_info['GPSLatitude'])
                if gps_info.get('GPSLatitudeRef') == 'S':
                    gps_data['latitude'] = -gps_data['latitude']
                    
            if 'GPSLongitude' in gps_info:
                gps_data['longitude'] = convert_to_degrees(gps_info['GPSLongitude'])
                if gps_info.get('GPSLongitudeRef') == 'W':
                    gps_data['longitude'] = -gps_data['longitude']
                    
            return gps_data
        except:
            return {}

    def _analyze_color_profile(self, image: Image.Image) -> Dict[str, Any]:
        """Analyze image color characteristics"""
        # Convert to numpy array for analysis
        img_array = np.array(image)
        
        # Color distribution
        colors = img_array.reshape(-1, 3)
        
        # Dominant colors using KMeans
        kmeans = KMeans(n_clusters=5, random_state=42, n_init=10)
        dominant_colors = kmeans.fit(colors).cluster_centers_.astype(int)
        
        # Color statistics
        color_stats = {
            'mean_rgb': np.mean(colors, axis=0).tolist(),
            'std_rgb': np.std(colors, axis=0).tolist(),
            'dominant_colors': dominant_colors.tolist(),
            'brightness': np.mean(colors),
            'contrast': np.std(colors),
            'saturation': self._calculate_saturation(colors)
        }
        
        return color_stats

    def _calculate_saturation(self, colors: np.ndarray) -> float:
        """Calculate average saturation of image"""
        # Convert RGB to HSV
        rgb_normalized = colors / 255.0
        max_vals = np.max(rgb_normalized, axis=1)
        min_vals = np.min(rgb_normalized, axis=1)
        
        # Calculate saturation
        saturation = np.where(max_vals == 0, 0, (max_vals - min_vals) / max_vals)
        return float(np.mean(saturation))

    def _estimate_compression_quality(self, image: Image.Image) -> Optional[int]:
        """Estimate JPEG compression quality"""
        try:
            if hasattr(image, '_getexif') and image.format == 'JPEG':
                # Estimate based on file size and dimensions
                width, height = image.size
                expected_size = width * height * 3  # Uncompressed RGB
                
                # Simple quality estimation
                if hasattr(image, 'fp') and image.fp:
                    actual_size = len(image.fp.read())
                    ratio = actual_size / expected_size
                    
                    if ratio > 0.3:
                        return 95
                    elif ratio > 0.15:
                        return 85
                    elif ratio > 0.08:
                        return 75
                    elif ratio > 0.04:
                        return 65
                    else:
                        return 50
        except:
            pass
        return None

    async def _assess_quality(self, image: Image.Image) -> QualityMetrics:
        """Comprehensive image quality assessment"""
        img_array = np.array(image)
        gray = cv2.cvtColor(img_array, cv2.COLOR_RGB2GRAY)
        
        # Sharpness (Laplacian variance)
        sharpness_score = cv2.Laplacian(gray, cv2.CV_64F).var() / 1000.0
        sharpness_score = min(1.0, sharpness_score)
        
        # Brightness
        brightness = np.mean(gray)
        brightness_score = 1.0 - abs(brightness - 128) / 128.0
        
        # Contrast (standard deviation)
        contrast = np.std(gray) / 128.0
        contrast_score = min(1.0, contrast)
        
        # Saturation
        saturation_score = self._calculate_saturation(img_array.reshape(-1, 3))
        
        # Noise level estimation
        noise_level = self._estimate_noise(gray)
        
        # Blur detection
        blur_detection = self._detect_blur(gray)
        
        # Resolution score
        width, height = image.size
        resolution_score = min(1.0, (width * height) / (1920 * 1080))
        
        # Aspect ratio score (favor common ratios)
        aspect_ratio = width / height
        common_ratios = [16/9, 4/3, 1/1, 3/4, 9/16]
        aspect_ratio_score = max([1.0 - abs(aspect_ratio - ratio) for ratio in common_ratios])
        
        # Composition score (rule of thirds, symmetry)
        composition_score = self._assess_composition(gray)
        
        # Aesthetic score using CLIP
        aesthetic_score = await self._assess_aesthetics(image)
        
        # Calculate overall quality
        weights = {
            'sharpness': 0.2,
            'brightness': 0.15,
            'contrast': 0.15,
            'saturation': 0.1,
            'noise': 0.1,
            'blur': 0.1,
            'resolution': 0.1,
            'composition': 0.05,
            'aesthetic': 0.05
        }
        
        overall_quality = (
            sharpness_score * weights['sharpness'] +
            brightness_score * weights['brightness'] +
            contrast_score * weights['contrast'] +
            saturation_score * weights['saturation'] +
            (1.0 - noise_level) * weights['noise'] +
            (1.0 - blur_detection) * weights['blur'] +
            resolution_score * weights['resolution'] +
            composition_score * weights['composition'] +
            aesthetic_score * weights['aesthetic']
        )
        
        # Determine quality level
        if overall_quality >= 0.8:
            quality_level = ImageQuality.PREMIUM
        elif overall_quality >= 0.6:
            quality_level = ImageQuality.HIGH
        elif overall_quality >= 0.4:
            quality_level = ImageQuality.MEDIUM
        else:
            quality_level = ImageQuality.LOW
        
        return QualityMetrics(
            sharpness_score=sharpness_score,
            brightness_score=brightness_score,
            contrast_score=contrast_score,
            saturation_score=saturation_score,
            noise_level=noise_level,
            blur_detection=blur_detection,
            resolution_score=resolution_score,
            aspect_ratio_score=aspect_ratio_score,
            composition_score=composition_score,
            aesthetic_score=aesthetic_score,
            overall_quality=overall_quality,
            quality_level=quality_level
        )

    def _estimate_noise(self, gray_image: np.ndarray) -> float:
        """Estimate noise level in grayscale image"""
        # Use Laplacian to detect edges, then analyze non-edge areas
        laplacian = cv2.Laplacian(gray_image, cv2.CV_64F)
        edges = np.abs(laplacian) > np.std(laplacian)
        
        # Calculate noise in non-edge areas
        non_edge_pixels = gray_image[~edges]
        if len(non_edge_pixels) > 0:
            noise = np.std(non_edge_pixels) / 255.0
            return min(1.0, noise * 4)  # Scale to 0-1 range
        return 0.5

    def _detect_blur(self, gray_image: np.ndarray) -> float:
        """Detect blur using frequency domain analysis"""
        # FFT-based blur detection
        f_transform = np.fft.fft2(gray_image)
        f_shift = np.fft.fftshift(f_transform)
        magnitude_spectrum = np.log(np.abs(f_shift) + 1)
        
        # Measure high frequency content
        height, width = magnitude_spectrum.shape
        center_y, center_x = height // 2, width // 2
        
        # Create high-pass filter
        y, x = np.ogrid[:height, :width]
        mask = (x - center_x) ** 2 + (y - center_y) ** 2 > (min(height, width) / 4) ** 2
        
        high_freq_energy = np.mean(magnitude_spectrum[mask])
        total_energy = np.mean(magnitude_spectrum)
        
        blur_score = 1.0 - (high_freq_energy / total_energy if total_energy > 0 else 0)
        return max(0.0, min(1.0, blur_score))

    def _assess_composition(self, gray_image: np.ndarray) -> float:
        """Assess image composition using rule of thirds and symmetry"""
        height, width = gray_image.shape
        
        # Rule of thirds
        third_h, third_w = height // 3, width // 3
        
        # Check for interesting content at intersection points
        intersections = [
            (third_w, third_h), (2 * third_w, third_h),
            (third_w, 2 * third_h), (2 * third_w, 2 * third_h)
        ]
        
        composition_score = 0.0
        for x, y in intersections:
            # Check local variance around intersection
            region = gray_image[max(0, y-10):min(height, y+10), 
                              max(0, x-10):min(width, x+10)]
            if region.size > 0:
                local_variance = np.var(region) / 255.0
                composition_score += local_variance
        
        composition_score /= len(intersections)
        
        # Symmetry detection
        left_half = gray_image[:, :width//2]
        right_half = np.fliplr(gray_image[:, width//2:])
        if left_half.shape == right_half.shape:
            symmetry_score = 1.0 - np.mean(np.abs(left_half - right_half)) / 255.0
            composition_score = (composition_score + symmetry_score) / 2
        
        return min(1.0, composition_score)

    async def _assess_aesthetics(self, image: Image.Image) -> float:
        """Assess aesthetic quality using CLIP model"""
        if not self.clip_model:
            return 0.5  # Default score if model not available
        
        try:
            # Define aesthetic prompts
            aesthetic_prompts = [
                "a beautiful high quality photograph",
                "professional photography with good composition",
                "aesthetically pleasing image with good lighting",
                "visually appealing content"
            ]
            
            non_aesthetic_prompts = [
                "blurry low quality image",
                "poorly lit photograph",
                "amateur snapshot with bad composition"
            ]
            
            # Process image and text
            inputs = self.clip_processor(
                text=aesthetic_prompts + non_aesthetic_prompts,
                images=image,
                return_tensors="pt",
                padding=True
            ).to(self.device)
            
            # Get similarity scores
            with torch.no_grad():
                outputs = self.clip_model(**inputs)
                logits_per_image = outputs.logits_per_image
                probs = logits_per_image.softmax(dim=1)
            
            # Calculate aesthetic score
            aesthetic_prob = probs[0][:len(aesthetic_prompts)].sum().item()
            return min(1.0, max(0.0, aesthetic_prob))
            
        except Exception as e:
            logger.warning(f"Aesthetic assessment failed: {e}")
            return 0.5

    async def _assess_originality(self, image: Image.Image, 
                                metadata: ImageMetadata) -> OriginalityMetrics:
        """Assess content originality and authenticity"""
        
        # Uniqueness based on perceptual hashing
        uniqueness_score = await self._check_hash_uniqueness(metadata)
        
        # Template matching for common meme formats
        template_match_score = self._detect_template_usage(image)
        
        # Watermark detection
        watermark_detected = self._detect_watermarks(image)
        
        # Stock photo probability
        stock_photo_probability = await self._detect_stock_photo(image)
        
        # AI-generated content detection
        ai_generated_probability = self._detect_ai_generated(image, metadata)
        
        # Editing level assessment
        editing_level = self._assess_editing_level(image, metadata)
        
        # Calculate duplicate probability
        duplicate_probability = 1.0 - uniqueness_score
        
        # Overall authenticity score
        authenticity_score = (
            uniqueness_score * 0.3 +
            (1.0 - template_match_score) * 0.2 +
            (1.0 - watermark_detected) * 0.15 +
            (1.0 - stock_photo_probability) * 0.15 +
            (1.0 - ai_generated_probability) * 0.1 +
            (1.0 - editing_level) * 0.1
        )
        
        return OriginalityMetrics(
            uniqueness_score=uniqueness_score,
            duplicate_probability=duplicate_probability,
            template_match_score=template_match_score,
            watermark_detected=watermark_detected,
            stock_photo_probability=stock_photo_probability,
            ai_generated_probability=ai_generated_probability,
            editing_level=editing_level,
            authenticity_score=authenticity_score
        )

    async def _check_hash_uniqueness(self, metadata: ImageMetadata) -> float:
        """Check hash uniqueness against database of known images"""
        # In a real implementation, this would query a database
        # For now, return a mock uniqueness score based on hash diversity
        
        hashes = [
            metadata.hash_perceptual,
            metadata.hash_difference,
            metadata.hash_average,
            metadata.hash_wavelet
        ]
        
        # Simple uniqueness estimation based on hash entropy
        hash_diversity = len(set(hashes)) / len(hashes)
        
        # Mock database check simulation
        common_patterns = ['0000', '1111', 'ffff', 'aaaa']
        similarity_penalty = sum(1 for hash_val in hashes 
                               for pattern in common_patterns 
                               if pattern in hash_val.lower()) / (len(hashes) * len(common_patterns))
        
        uniqueness = hash_diversity * (1.0 - similarity_penalty)
        return max(0.0, min(1.0, uniqueness))

    def _detect_template_usage(self, image: Image.Image) -> float:
        """Detect usage of common meme templates or formats"""
        # Convert to grayscale for template matching
        gray = cv2.cvtColor(np.array(image), cv2.COLOR_RGB2GRAY)
        
        # Common aspect ratios for memes
        width, height = image.size
        aspect_ratio = width / height
        
        # Common meme aspect ratios
        meme_ratios = [1.0, 16/9, 4/3, 3/4]
        ratio_match = max([1.0 - abs(aspect_ratio - ratio) for ratio in meme_ratios])
        
        # Text detection (memes often have text overlays)
        # Simple edge detection to find text-like regions
        edges = cv2.Canny(gray, 50, 150)
        contours, _ = cv2.findContours(edges, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)
        
        # Look for rectangular regions that might be text
        text_regions = 0
        for contour in contours:
            x, y, w, h = cv2.boundingRect(contour)
            if w > 50 and h > 20 and w/h > 2:  # Text-like proportions
                text_regions += 1
        
        text_score = min(1.0, text_regions / 10.0)  # Normalize to 0-1
        
        # Combine indicators
        template_score = (ratio_match * 0.6 + text_score * 0.4)
        return min(1.0, template_score)

    def _detect_watermarks(self, image: Image.Image) -> bool:
        """Detect presence of watermarks"""
        gray = cv2.cvtColor(np.array(image), cv2.COLOR_RGB2GRAY)
        
        # Look for semi-transparent overlays (common in watermarks)
        # This is a simplified detection - real implementation would be more sophisticated
        
        # Check corners and edges for watermark patterns
        height, width = gray.shape
        corners = [
            gray[0:height//4, 0:width//4],  # Top-left
            gray[0:height//4, -width//4:],  # Top-right
            gray[-height//4:, 0:width//4],  # Bottom-left
            gray[-height//4:, -width//4:]   # Bottom-right
        ]
        
        watermark_indicators = 0
        for corner in corners:
            # Look for consistent patterns that might indicate watermarks
            std_dev = np.std(corner)
            mean_val = np.mean(corner)
            
            # Watermarks often have low contrast but consistent patterns
            if std_dev < 20 and mean_val > 200:  # Light watermark
                watermark_indicators += 1
            elif std_dev < 15 and mean_val < 50:  # Dark watermark
                watermark_indicators += 1
        
        return watermark_indicators >= 2

    async def _detect_stock_photo(self, image: Image.Image) -> float:
        """Detect if image is likely a stock photo"""
        if not self.clip_model:
            return 0.3  # Default probability
        
        try:
            stock_indicators = [
                "professional stock photography",
                "commercial stock image with perfect lighting",
                "generic business photo with models",
                "staged corporate photography"
            ]
            
            authentic_indicators = [
                "personal candid photograph",
                "amateur snapshot",
                "real life moment captured naturally",
                "authentic user generated content"
            ]
            
            inputs = self.clip_processor(
                text=stock_indicators + authentic_indicators,
                images=image,
                return_tensors="pt",
                padding=True
            ).to(self.device)
            
            with torch.no_grad():
                outputs = self.clip_model(**inputs)
                logits_per_image = outputs.logits_per_image
                probs = logits_per_image.softmax(dim=1)
            
            stock_prob = probs[0][:len(stock_indicators)].sum().item()
            return min(1.0, max(0.0, stock_prob))
            
        except Exception as e:
            logger.warning(f"Stock photo detection failed: {e}")
            return 0.3

    def _detect_ai_generated(self, image: Image.Image, metadata: ImageMetadata) -> float:
        """Detect AI-generated content probability"""
        indicators = []
        
        # Check EXIF data for AI generation signatures
        if not metadata.camera_info:
            indicators.append(0.3)  # No camera info might indicate AI
        
        # Perfect/unrealistic qualities often indicate AI
        img_array = np.array(image)
        
        # Extremely uniform regions (common in AI)
        gray = cv2.cvtColor(img_array, cv2.COLOR_RGB2GRAY)
        uniformity = 1.0 - (np.std(gray) / 128.0)
        if uniformity > 0.8:
            indicators.append(0.4)
        
        # Impossible lighting/shadows
        # This would require more sophisticated analysis in production
        
        # Frequency domain analysis for AI artifacts
        f_transform = np.fft.fft2(gray)
        magnitude_spectrum = np.abs(f_transform)
        
        # AI images sometimes have specific frequency patterns
        low_freq = np.mean(magnitude_spectrum[:gray.shape[0]//4, :gray.shape[1]//4])
        high_freq = np.mean(magnitude_spectrum[3*gray.shape[0]//4:, 3*gray.shape[1]//4:])
        
        if low_freq / high_freq > 10:  # Unusual frequency distribution
            indicators.append(0.2)
        
        # Calculate average probability
        return np.mean(indicators) if indicators else 0.2

    def _assess_editing_level(self, image: Image.Image, metadata: ImageMetadata) -> float:
        """Assess level of image editing/manipulation"""
        editing_indicators = []
        
        # EXIF software analysis
        if metadata.exif_data.get('Software'):
            software = metadata.exif_data['Software'].lower()
            editing_software = ['photoshop', 'gimp', 'lightroom', 'facetune', 'snapseed']
            if any(editor in software for editor in editing_software):
                editing_indicators.append(0.7)
        
        # Compression artifacts analysis
        if metadata.compression_quality and metadata.compression_quality < 70:
            editing_indicators.append(0.3)  # Heavy compression might indicate editing
        
        # Color analysis for unnatural enhancements
        color_profile = metadata.color_profile
        saturation = color_profile.get('saturation', 0.5)
        if saturation > 0.8:  # Oversaturated
            editing_indicators.append(0.4)
        
        # Edge analysis for sharpening artifacts
        gray = cv2.cvtColor(np.array(image), cv2.COLOR_RGB2GRAY)
        edges = cv2.Canny(gray, 50, 150)
        edge_density = np.sum(edges > 0) / edges.size
        
        if edge_density > 0.15:  # High edge density might indicate sharpening
            editing_indicators.append(0.3)
        
        return np.mean(editing_indicators) if editing_indicators else 0.1

    async def _predict_engagement(self, image: Image.Image, platform: str, 
                                metadata: ImageMetadata) -> EngagementPrediction:
        """Predict engagement potential across platforms"""
        
        # Platform-specific scoring
        platform_scores = await self._calculate_platform_fitness(image, platform)
        
        # Base engagement factors
        aesthetic_appeal = await self._assess_aesthetics(image)
        visual_interest = self._calculate_visual_interest(image)
        trend_alignment = await self._assess_trend_alignment(image, platform)
        optimal_timing = self._calculate_optimal_posting_score(metadata)
        audience_targeting = self._calculate_audience_match(image, platform)
        
        # Engagement probability calculations
        like_probability = (
            aesthetic_appeal * 0.4 +
            visual_interest * 0.3 +
            platform_scores.get(platform, 0.5) * 0.3
        )
        
        share_probability = (
            trend_alignment * 0.4 +
            visual_interest * 0.3 +
            audience_targeting * 0.3
        )
        
        comment_probability = (
            visual_interest * 0.5 +
            trend_alignment * 0.3 +
            audience_targeting * 0.2
        )
        
        # Viral potential calculation
        viral_factors = [aesthetic_appeal, visual_interest, trend_alignment, audience_targeting]
        viral_potential = np.mean(viral_factors) * np.std(viral_factors)  # High mean + high variance
        
        return EngagementPrediction(
            like_probability=min(1.0, like_probability),
            share_probability=min(1.0, share_probability),
            comment_probability=min(1.0, comment_probability),
            viral_potential=min(1.0, viral_potential),
            platform_fitness=platform_scores,
            optimal_posting_score=optimal_timing,
            target_audience_match=audience_targeting
        )

    async def _calculate_platform_fitness(self, image: Image.Image, 
                                        primary_platform: str) -> Dict[str, float]:
        """Calculate fitness scores for different social platforms"""
        width, height = image.size
        aspect_ratio = width / height
        
        platform_preferences = {
            'instagram': {
                'aspect_ratios': [1.0, 4/5, 9/16],  # Square, portrait, stories
                'min_resolution': (1080, 1080),
                'content_types': ['lifestyle', 'aesthetic', 'personal'],
                'color_preference': 'vibrant'
            },
            'tiktok': {
                'aspect_ratios': [9/16],  # Vertical
                'min_resolution': (720, 1280),
                'content_types': ['dynamic', 'trending', 'youth'],
                'color_preference': 'bold'
            },
            'youtube': {
                'aspect_ratios': [16/9],  # Landscape
                'min_resolution': (1280, 720),
                'content_types': ['educational', 'entertainment'],
                'color_preference': 'professional'
            },
            'facebook': {
                'aspect_ratios': [16/9, 1.91/1, 1/1],
                'min_resolution': (1200, 630),
                'content_types': ['social', 'news', 'family'],
                'color_preference': 'natural'
            },
            'twitter': {
                'aspect_ratios': [16/9, 2/1],
                'min_resolution': (1200, 675),
                'content_types': ['news', 'commentary', 'viral'],
                'color_preference': 'high_contrast'
            }
        }
        
        scores = {}
        for platform, prefs in platform_preferences.items():
            score = 0.0
            
            # Aspect ratio fitness
            ratio_scores = [1.0 - abs(aspect_ratio - ratio) for ratio in prefs['aspect_ratios']]
            aspect_score = max(ratio_scores)
            score += aspect_score * 0.4
            
            # Resolution fitness
            min_w, min_h = prefs['min_resolution']
            resolution_score = min(1.0, min(width/min_w, height/min_h))
            score += resolution_score * 0.3
            
            # Content type fitness (using CLIP if available)
            if self.clip_model:
                content_score = await self._assess_content_type_fitness(image, prefs['content_types'])
                score += content_score * 0.3
            else:
                score += 0.5 * 0.3  # Default score
            
            scores[platform] = min(1.0, score)
        
        return scores

    async def _assess_content_type_fitness(self, image: Image.Image, 
                                         content_types: List[str]) -> float:
        """Assess how well image fits specific content types"""
        if not self.clip_model:
            return 0.5
        
        try:
            type_prompts = [f"a {content_type} image" for content_type in content_types]
            generic_prompts = ["a generic photograph", "ordinary image content"]
            
            inputs = self.clip_processor(
                text=type_prompts + generic_prompts,
                images=image,
                return_tensors="pt",
                padding=True
            ).to(self.device)
            
            with torch.no_grad():
                outputs = self.clip_model(**inputs)
                logits_per_image = outputs.logits_per_image
                probs = logits_per_image.softmax(dim=1)
            
            type_prob = probs[0][:len(type_prompts)].sum().item()
            return min(1.0, max(0.0, type_prob))
            
        except Exception as e:
            logger.warning(f"Content type fitness assessment failed: {e}")
            return 0.5

    def _calculate_visual_interest(self, image: Image.Image) -> float:
        """Calculate visual interest score"""
        img_array = np.array(image)
        gray = cv2.cvtColor(img_array, cv2.COLOR_RGB2GRAY)
        
        # Edge density (more edges = more visual interest)
        edges = cv2.Canny(gray, 50, 150)
        edge_density = np.sum(edges > 0) / edges.size
        
        # Color variance
        color_variance = np.var(img_array.reshape(-1, 3), axis=0).mean() / (255**2)
        
        # Texture analysis using Local Binary Patterns
        from skimage.feature import local_binary_pattern
        radius = 3
        n_points = 8 * radius
        lbp = local_binary_pattern(gray, n_points, radius, method='uniform')
        texture_variance = np.var(lbp) / (2**n_points)
        
        # Face detection adds interest
        try:
            face_locations = face_recognition.face_locations(img_array)
            face_score = min(1.0, len(face_locations) * 0.3)
        except:
            face_score = 0.0
        
        # Combine factors
        visual_interest = (
            edge_density * 0.3 +
            color_variance * 0.3 +
            texture_variance * 0.2 +
            face_score * 0.2
        )
        
        return min(1.0, visual_interest)

    async def _assess_trend_alignment(self, image: Image.Image, platform: str) -> float:
        """Assess alignment with current trends"""
        # In production, this would connect to trend APIs
        # For now, return a mock score based on image characteristics
        
        current_trends = {
            'instagram': ['minimalist', 'aesthetic', 'lifestyle', 'nature'],
            'tiktok': ['viral', 'challenge', 'dance', 'comedy', 'trending'],
            'youtube': ['educational', 'tutorial', 'review', 'vlog'],
            'facebook': ['family', 'news', 'community', 'events'],
            'twitter': ['breaking', 'opinion', 'meme', 'discussion']
        }
        
        platform_trends = current_trends.get(platform, ['general'])
        
        if self.clip_model:
            try:
                trend_prompts = [f"trending {trend} content" for trend in platform_trends]
                
                inputs = self.clip_processor(
                    text=trend_prompts,
                    images=image,
                    return_tensors="pt",
                    padding=True
                ).to(self.device)
                
                with torch.no_grad():
                    outputs = self.clip_model(**inputs)
                    logits_per_image = outputs.logits_per_image
                    probs = logits_per_image.softmax(dim=1)
                
                trend_score = probs[0].max().item()
                return min(1.0, max(0.0, trend_score))
                
            except Exception as e:
                logger.warning(f"Trend alignment assessment failed: {e}")
        
        return 0.5  # Default trend score

    def _calculate_optimal_posting_score(self, metadata: ImageMetadata) -> float:
        """Calculate optimal posting time score based on image metadata"""
        # In production, this would consider:
        # - Time zones of target audience
        # - Historical engagement patterns
        # - Platform-specific optimal times
        
        # For now, provide a basic score based on image freshness
        creation_time = metadata.creation_time
        if creation_time:
            try:
                from datetime import datetime
                created = datetime.strptime(creation_time, '%Y:%m:%d %H:%M:%S')
                now = datetime.now()
                age_hours = (now - created).total_seconds() / 3600
                
                # Fresh content scores higher
                freshness_score = max(0.0, 1.0 - (age_hours / 168))  # 1 week decay
                return freshness_score
            except:
                pass
        
        return 0.7  # Default score for unknown timing

    def _calculate_audience_match(self, image: Image.Image, platform: str) -> float:
        """Calculate target audience matching score"""
        # Demographics inference based on image content
        # In production, this would use more sophisticated models
        
        audience_indicators = {
            'age_groups': {
                'youth': ['trendy', 'colorful', 'dynamic'],
                'adult': ['professional', 'lifestyle', 'quality'],
                'senior': ['traditional', 'family', 'simple']
            },
            'interests': {
                'tech': ['gadgets', 'screens', 'modern'],
                'lifestyle': ['fashion', 'food', 'travel'],
                'business': ['professional', 'corporate', 'formal']
            }
        }
        
        # Platform demographics (simplified)
        platform_demographics = {
            'tiktok': {'primary_age': 'youth', 'interests': ['entertainment', 'trends']},
            'instagram': {'primary_age': 'adult', 'interests': ['lifestyle', 'visual']},
            'facebook': {'primary_age': 'adult', 'interests': ['social', 'news']},
            'linkedin': {'primary_age': 'adult', 'interests': ['business', 'professional']},
            'twitter': {'primary_age': 'adult', 'interests': ['news', 'discussion']}
        }
        
        platform_demo = platform_demographics.get(platform, {})
        
        # Basic matching score (would be more sophisticated in production)
        base_score = 0.6
        
        # Adjust based on image characteristics
        img_array = np.array(image)
        brightness = np.mean(img_array)
        saturation = np.std(img_array) / 255.0
        
        # Youth content tends to be brighter and more saturated
        if platform_demo.get('primary_age') == 'youth':
            if brightness > 128 and saturation > 0.3:
                base_score += 0.2
        elif platform_demo.get('primary_age') == 'adult':
            if brightness > 100 and saturation < 0.7:
                base_score += 0.1
        
        return min(1.0, base_score)

    def _calculate_composite_score(self, quality: QualityMetrics,
                                 originality: OriginalityMetrics,
                                 engagement: EngagementPrediction) -> Dict[str, float]:
        """Calculate composite scoring for Finova reward system"""
        
        weights = self.config['engagement_weights']
        
        # Base Finova score calculation
        finova_score = (
            quality.overall_quality * weights['quality'] +
            originality.authenticity_score * weights['originality'] +
            quality.aesthetic_score * weights['aesthetic'] +
            engagement.viral_potential * weights['trending'] +
            engagement.target_audience_match * weights['relevance']
        )
        
        # XP multiplier calculation
        xp_multiplier = quality.quality_level.value * (1.0 + originality.uniqueness_score * 0.5)
        
        # Mining boost calculation  
        mining_boost = 1.0 + (finova_score * 2.0) + (quality.quality_level.value - 1.0) * 0.5
        
        # RP network value
        rp_network_value = originality.authenticity_score * engagement.viral_potential
        
        return {
            'finova_score': finova_score,
            'xp_multiplier': min(2.0, xp_multiplier),  # Cap at 2x
            'mining_boost': min(3.0, mining_boost),    # Cap at 3x  
            'rp_network_value': rp_network_value,
            'quality_tier': quality.quality_level.name,
            'recommendation_confidence': np.mean([quality.overall_quality, 
                                                originality.authenticity_score])
        }

    def _generate_recommendations(self, quality: QualityMetrics,
                                originality: OriginalityMetrics,
                                engagement: EngagementPrediction,
                                platform: str) -> Dict[str, Any]:
        """Generate actionable recommendations for content improvement"""
        
        recommendations = {
            'improvements': [],
            'platform_optimizations': [],
            'engagement_tips': [],
            'quality_enhancements': [],
            'originality_suggestions': []
        }
        
        # Quality improvements
        if quality.sharpness_score < 0.5:
            recommendations['quality_enhancements'].append(
                "Increase image sharpness - avoid camera shake and ensure proper focus"
            )
        
        if quality.brightness_score < 0.6:
            recommendations['quality_enhancements'].append(
                "Optimize lighting conditions - current brightness may be too dark or too bright"
            )
        
        if quality.contrast_score < 0.4:
            recommendations['quality_enhancements'].append(
                "Enhance contrast to make your image more visually appealing"
            )
        
        # Originality suggestions
        if originality.authenticity_score < 0.6:
            recommendations['originality_suggestions'].append(
                "Add more personal touches to increase content authenticity"
            )
        
        if originality.duplicate_probability > 0.3:
            recommendations['originality_suggestions'].append(
                "This content appears similar to existing images - try a unique angle or approach"
            )
        
        # Platform optimizations
        platform_score = engagement.platform_fitness.get(platform, 0.5)
        if platform_score < 0.7:
            recommendations['platform_optimizations'].append(
                f"Optimize image dimensions and style for {platform} - current fit score: {platform_score:.1%}"
            )
        
        # Engagement tips
        if engagement.viral_potential < 0.4:
            recommendations['engagement_tips'].extend([
                "Consider adding trending elements to increase viral potential",
                "Focus on visual storytelling to boost engagement"
            ])
        
        if engagement.target_audience_match < 0.5:
            recommendations['engagement_tips'].append(
                f"Better align content with {platform} audience preferences"
            )
        
        return recommendations

    def _update_stats(self, processing_time: float, quality_level: ImageQuality):
        """Update processing statistics"""
        self.stats['processed_count'] += 1
        self.stats['total_processing_time'] += processing_time
        self.stats['average_processing_time'] = (
            self.stats['total_processing_time'] / self.stats['processed_count']
        )
        self.stats['quality_distribution'][quality_level.name] += 1

    def get_performance_stats(self) -> Dict[str, Any]:
        """Get current performance statistics"""
        return {
            'processed_images': self.stats['processed_count'],
            'average_processing_time': self.stats['average_processing_time'],
            'total_processing_time': self.stats['total_processing_time'],
            'quality_distribution': self.stats['quality_distribution'],
            'error_rate': self.stats['error_count'] / max(1, self.stats['processed_count']),
            'system_info': {
                'device': str(self.device),
                'available_memory': psutil.virtual_memory().available / (1024**3),  # GB
                'cpu_usage': psutil.cpu_percent()
            }
        }

    async def batch_process_images(self, image_inputs: List[Union[str, bytes, Image.Image]], 
                                 platform: str = "general") -> List[Dict[str, Any]]:
        """Process multiple images in parallel"""
        
        tasks = [self.process_image(img, platform) for img in image_inputs]
        results = await asyncio.gather(*tasks, return_exceptions=True)
        
        # Handle exceptions
        processed_results = []
        for i, result in enumerate(results):
            if isinstance(result, Exception):
                processed_results.append({
                    'success': False,
                    'error': str(result),
                    'image_index': i
                })
            else:
                processed_results.append(result)
        
        return processed_results

    def cleanup(self):
        """Cleanup resources"""
        if hasattr(self, 'executor'):
            self.executor.shutdown(wait=True)
        
        # Clear GPU memory if using CUDA
        if self.device.type == 'cuda':
            torch.cuda.empty_cache()
        
        logger.info("ImageProcessor cleanup completed")

# Example usage and testing
if __name__ == "__main__":
    import asyncio
    
    async def test_image_processor():
        """Test the image processor with sample usage"""
        processor = ImageProcessor()
        
        # Test with a sample image (you would provide actual image path/data)
        try:
            # Example: process a local image file
            result = await processor.process_image("sample_image.jpg", "instagram")
            
            if result['success']:
                print("Processing successful!")
                print(f"Overall quality: {result['quality_metrics'].overall_quality:.2f}")
                print(f"Originality score: {result['originality_metrics'].authenticity_score:.2f}")
                print(f"Viral potential: {result['engagement_prediction'].viral_potential:.2f}")
                print(f"Finova score: {result['composite_score']['finova_score']:.2f}")
                
                # Print recommendations
                for category, recs in result['recommendations'].items():
                    if recs:
                        print(f"\n{category.replace('_', ' ').title()}:")
                        for rec in recs:
                            print(f"  - {rec}")
            else:
                print(f"Processing failed: {result['error']}")
                
        except Exception as e:
            print(f"Test failed: {e}")
        
        # Print performance stats
        stats = processor.get_performance_stats()
        print(f"\nPerformance Stats:")
        print(f"Processed images: {stats['processed_images']}")
        print(f"Average processing time: {stats['average_processing_time']:.2f}s")
        
        # Cleanup
        processor.cleanup()
    
    # Run the test
    # asyncio.run(test_image_processor())
    