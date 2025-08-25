import { Buffer } from 'buffer';
import CryptoJS from 'crypto-js';
import * as ed25519 from '@noble/ed25519';
import { sha256 } from '@noble/hashes/sha256';
import { sha512 } from '@noble/hashes/sha512';
import { randomBytes } from '@noble/hashes/utils';
import { base58, base64 } from '@scure/base';
import { Keypair, PublicKey } from '@solana/web3.js';
import nacl from 'tweetnacl';

/**
 * Finova Network Cryptographic Utilities
 * Comprehensive crypto functions for social-fi mining, authentication, and security
 */

export interface CryptoConfig {
  encryptionAlgorithm: string;
  keyDerivationRounds: number;
  saltLength: number;
  ivLength: number;
  tagLength: number;
}

export interface KeyPair {
  privateKey: Uint8Array;
  publicKey: Uint8Array;
  address: string;
}

export interface EncryptedData {
  data: string;
  iv: string;
  salt: string;
  tag: string;
  algorithm: string;
}

export interface MiningProof {
  userAddress: string;
  timestamp: number;
  nonce: string;
  difficulty: number;
  hash: string;
  signature: string;
}

export interface BiometricHash {
  hash: string;
  salt: string;
  iterations: number;
  algorithm: string;
}

export class FinovaCrypto {
  private static readonly DEFAULT_CONFIG: CryptoConfig = {
    encryptionAlgorithm: 'AES-GCM',
    keyDerivationRounds: 100000,
    saltLength: 32,
    ivLength: 12,
    tagLength: 16
  };

  /**
   * Generate secure random bytes
   */
  static generateRandomBytes(length: number): Uint8Array {
    try {
      return randomBytes(length);
    } catch (error) {
      throw new Error(`Failed to generate random bytes: ${error}`);
    }
  }

  /**
   * Generate cryptographically secure random string
   */
  static generateRandomString(length: number = 32): string {
    const charset = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    const randomBytes = this.generateRandomBytes(length);
    return Array.from(randomBytes)
      .map(byte => charset[byte % charset.length])
      .join('');
  }

  /**
   * Generate UUID v4
   */
  static generateUUID(): string {
    const randomBytes = this.generateRandomBytes(16);
    randomBytes[6] = (randomBytes[6] & 0x0f) | 0x40; // Version 4
    randomBytes[8] = (randomBytes[8] & 0x3f) | 0x80; // Variant 10
    
    const hex = Array.from(randomBytes)
      .map(b => b.toString(16).padStart(2, '0'))
      .join('');
    
    return [
      hex.slice(0, 8),
      hex.slice(8, 12),
      hex.slice(12, 16),
      hex.slice(16, 20),
      hex.slice(20, 32)
    ].join('-');
  }

  /**
   * SHA-256 hashing
   */
  static sha256Hash(data: string | Uint8Array): string {
    const input = typeof data === 'string' ? new TextEncoder().encode(data) : data;
    const hash = sha256(input);
    return Array.from(hash).map(b => b.toString(16).padStart(2, '0')).join('');
  }

  /**
   * SHA-512 hashing
   */
  static sha512Hash(data: string | Uint8Array): string {
    const input = typeof data === 'string' ? new TextEncoder().encode(data) : data;
    const hash = sha512(input);
    return Array.from(hash).map(b => b.toString(16).padStart(2, '0')).join('');
  }

  /**
   * HMAC-SHA256
   */
  static hmacSha256(data: string, key: string): string {
    const hmac = CryptoJS.HmacSHA256(data, key);
    return hmac.toString(CryptoJS.enc.Hex);
  }

  /**
   * HMAC-SHA512
   */
  static hmacSha512(data: string, key: string): string {
    const hmac = CryptoJS.HmacSHA512(data, key);
    return hmac.toString(CryptoJS.enc.Hex);
  }

  /**
   * PBKDF2 key derivation
   */
  static deriveKey(
    password: string,
    salt: string,
    iterations: number = 100000,
    keyLength: number = 32
  ): string {
    const key = CryptoJS.PBKDF2(password, salt, {
      keySize: keyLength / 4,
      iterations: iterations,
      hasher: CryptoJS.algo.SHA256
    });
    return key.toString(CryptoJS.enc.Hex);
  }

  /**
   * Argon2-like key derivation (simplified implementation)
   */
  static argon2Derive(
    password: string,
    salt: string,
    iterations: number = 3,
    memory: number = 4096,
    parallelism: number = 1
  ): string {
    // Simplified Argon2-like implementation using PBKDF2 with higher iterations
    const adjustedIterations = iterations * memory * parallelism;
    return this.deriveKey(password, salt, adjustedIterations, 32);
  }

  /**
   * AES-GCM encryption
   */
  static encrypt(data: string, password: string): EncryptedData {
    try {
      const salt = this.generateRandomBytes(this.DEFAULT_CONFIG.saltLength);
      const iv = this.generateRandomBytes(this.DEFAULT_CONFIG.ivLength);
      
      const key = this.deriveKey(
        password,
        Buffer.from(salt).toString('hex'),
        this.DEFAULT_CONFIG.keyDerivationRounds
      );

      const cipher = CryptoJS.AES.encrypt(data, key, {
        iv: CryptoJS.enc.Hex.parse(Buffer.from(iv).toString('hex')),
        mode: CryptoJS.mode.GCM,
        padding: CryptoJS.pad.NoPadding
      });

      return {
        data: cipher.ciphertext.toString(CryptoJS.enc.Base64),
        iv: Buffer.from(iv).toString('base64'),
        salt: Buffer.from(salt).toString('base64'),
        tag: cipher.tag?.toString(CryptoJS.enc.Base64) || '',
        algorithm: this.DEFAULT_CONFIG.encryptionAlgorithm
      };
    } catch (error) {
      throw new Error(`Encryption failed: ${error}`);
    }
  }

  /**
   * AES-GCM decryption
   */
  static decrypt(encryptedData: EncryptedData, password: string): string {
    try {
      const salt = Buffer.from(encryptedData.salt, 'base64');
      const iv = Buffer.from(encryptedData.iv, 'base64');
      
      const key = this.deriveKey(
        password,
        salt.toString('hex'),
        this.DEFAULT_CONFIG.keyDerivationRounds
      );

      const decrypted = CryptoJS.AES.decrypt(
        {
          ciphertext: CryptoJS.enc.Base64.parse(encryptedData.data),
          tag: CryptoJS.enc.Base64.parse(encryptedData.tag)
        },
        key,
        {
          iv: CryptoJS.enc.Hex.parse(iv.toString('hex')),
          mode: CryptoJS.mode.GCM,
          padding: CryptoJS.pad.NoPadding
        }
      );

      return decrypted.toString(CryptoJS.enc.Utf8);
    } catch (error) {
      throw new Error(`Decryption failed: ${error}`);
    }
  }

  /**
   * Generate Ed25519 keypair for Solana
   */
  static async generateSolanaKeypair(): Promise<KeyPair> {
    try {
      const keypair = Keypair.generate();
      return {
        privateKey: keypair.secretKey,
        publicKey: keypair.publicKey.toBytes(),
        address: keypair.publicKey.toBase58()
      };
    } catch (error) {
      throw new Error(`Failed to generate Solana keypair: ${error}`);
    }
  }

  /**
   * Generate Ed25519 keypair from seed
   */
  static async generateKeypairFromSeed(seed: string): Promise<KeyPair> {
    try {
      const seedHash = this.sha256Hash(seed);
      const seedBytes = Buffer.from(seedHash, 'hex').slice(0, 32);
      const keypair = Keypair.fromSeed(seedBytes);
      
      return {
        privateKey: keypair.secretKey,
        publicKey: keypair.publicKey.toBytes(),
        address: keypair.publicKey.toBase58()
      };
    } catch (error) {
      throw new Error(`Failed to generate keypair from seed: ${error}`);
    }
  }

  /**
   * Sign data with Ed25519 private key
   */
  static async signData(data: string | Uint8Array, privateKey: Uint8Array): Promise<string> {
    try {
      const message = typeof data === 'string' ? new TextEncoder().encode(data) : data;
      const signature = await ed25519.sign(message, privateKey.slice(0, 32));
      return Buffer.from(signature).toString('base64');
    } catch (error) {
      throw new Error(`Signing failed: ${error}`);
    }
  }

  /**
   * Verify Ed25519 signature
   */
  static async verifySignature(
    data: string | Uint8Array,
    signature: string,
    publicKey: string | Uint8Array
  ): Promise<boolean> {
    try {
      const message = typeof data === 'string' ? new TextEncoder().encode(data) : data;
      const sig = Buffer.from(signature, 'base64');
      const pubKey = typeof publicKey === 'string' ? 
        new PublicKey(publicKey).toBytes() : publicKey;
      
      return await ed25519.verify(sig, message, pubKey);
    } catch (error) {
      console.warn(`Signature verification failed: ${error}`);
      return false;
    }
  }

  /**
   * Create mining proof with timestamp and nonce
   */
  static async createMiningProof(
    userAddress: string,
    privateKey: Uint8Array,
    difficulty: number = 4
  ): Promise<MiningProof> {
    try {
      const timestamp = Date.now();
      let nonce = 0;
      let hash: string;
      
      // Proof of work - find hash starting with required zeros
      do {
        nonce++;
        const data = `${userAddress}:${timestamp}:${nonce}`;
        hash = this.sha256Hash(data);
      } while (!hash.startsWith('0'.repeat(difficulty)));
      
      const proofData = `${userAddress}:${timestamp}:${nonce}:${hash}`;
      const signature = await this.signData(proofData, privateKey);
      
      return {
        userAddress,
        timestamp,
        nonce: nonce.toString(),
        difficulty,
        hash,
        signature
      };
    } catch (error) {
      throw new Error(`Failed to create mining proof: ${error}`);
    }
  }

  /**
   * Verify mining proof
   */
  static async verifyMiningProof(proof: MiningProof, publicKey: string): Promise<boolean> {
    try {
      // Verify hash
      const data = `${proof.userAddress}:${proof.timestamp}:${proof.nonce}`;
      const expectedHash = this.sha256Hash(data);
      
      if (expectedHash !== proof.hash) return false;
      if (!proof.hash.startsWith('0'.repeat(proof.difficulty))) return false;
      
      // Verify signature
      const proofData = `${proof.userAddress}:${proof.timestamp}:${proof.nonce}:${proof.hash}`;
      return await this.verifySignature(proofData, proof.signature, publicKey);
    } catch (error) {
      console.warn(`Mining proof verification failed: ${error}`);
      return false;
    }
  }

  /**
   * Generate biometric hash for PoH (Proof of Humanity)
   */
  static generateBiometricHash(
    biometricData: string,
    userSalt?: string,
    iterations: number = 100000
  ): BiometricHash {
    try {
      const salt = userSalt || this.generateRandomString(32);
      const hash = this.deriveKey(biometricData, salt, iterations, 64);
      
      return {
        hash,
        salt,
        iterations,
        algorithm: 'PBKDF2-SHA256'
      };
    } catch (error) {
      throw new Error(`Failed to generate biometric hash: ${error}`);
    }
  }

  /**
   * Verify biometric hash
   */
  static verifyBiometricHash(
    biometricData: string,
    storedHash: BiometricHash,
    tolerance: number = 0.85
  ): boolean {
    try {
      const computedHash = this.deriveKey(
        biometricData,
        storedHash.salt,
        storedHash.iterations,
        64
      );
      
      // Simple hash comparison - in production, use fuzzy matching
      const similarity = this.calculateHashSimilarity(computedHash, storedHash.hash);
      return similarity >= tolerance;
    } catch (error) {
      console.warn(`Biometric hash verification failed: ${error}`);
      return false;
    }
  }

  /**
   * Calculate hash similarity (simplified Hamming distance)
   */
  static calculateHashSimilarity(hash1: string, hash2: string): number {
    if (hash1.length !== hash2.length) return 0;
    
    let matches = 0;
    for (let i = 0; i < hash1.length; i++) {
      if (hash1[i] === hash2[i]) matches++;
    }
    
    return matches / hash1.length;
  }

  /**
   * Generate JWT-compatible signature
   */
  static generateJWTSignature(header: string, payload: string, secret: string): string {
    const data = `${header}.${payload}`;
    return this.hmacSha256(data, secret);
  }

  /**
   * Verify JWT signature
   */
  static verifyJWTSignature(
    header: string,
    payload: string,
    signature: string,
    secret: string
  ): boolean {
    const expectedSignature = this.generateJWTSignature(header, payload, secret);
    return this.constantTimeEqual(signature, expectedSignature);
  }

  /**
   * Constant-time string comparison to prevent timing attacks
   */
  static constantTimeEqual(a: string, b: string): boolean {
    if (a.length !== b.length) return false;
    
    let result = 0;
    for (let i = 0; i < a.length; i++) {
      result |= a.charCodeAt(i) ^ b.charCodeAt(i);
    }
    
    return result === 0;
  }

  /**
   * Generate secure session token
   */
  static generateSessionToken(userId: string, deviceId: string): string {
    const timestamp = Date.now().toString();
    const randomString = this.generateRandomString(16);
    const data = `${userId}:${deviceId}:${timestamp}:${randomString}`;
    return base64.encode(new TextEncoder().encode(data));
  }

  /**
   * Create anti-bot challenge
   */
  static createAntiBotChallenge(difficulty: number = 3): {
    challenge: string;
    solution: string;
  } {
    const timestamp = Date.now();
    const random = this.generateRandomString(16);
    const challenge = `${timestamp}:${random}`;
    
    let nonce = 0;
    let hash: string;
    
    do {
      nonce++;
      hash = this.sha256Hash(`${challenge}:${nonce}`);
    } while (!hash.startsWith('0'.repeat(difficulty)));
    
    return {
      challenge,
      solution: nonce.toString()
    };
  }

  /**
   * Verify anti-bot challenge solution
   */
  static verifyAntiBotSolution(
    challenge: string,
    solution: string,
    difficulty: number = 3,
    maxAge: number = 300000 // 5 minutes
  ): boolean {
    try {
      const [timestampStr] = challenge.split(':');
      const timestamp = parseInt(timestampStr);
      
      // Check if challenge is still valid
      if (Date.now() - timestamp > maxAge) return false;
      
      // Verify solution
      const hash = this.sha256Hash(`${challenge}:${solution}`);
      return hash.startsWith('0'.repeat(difficulty));
    } catch (error) {
      console.warn(`Anti-bot verification failed: ${error}`);
      return false;
    }
  }

  /**
   * Encrypt sensitive data for storage
   */
  static encryptForStorage(data: any, masterKey: string): string {
    const jsonData = JSON.stringify(data);
    const encrypted = this.encrypt(jsonData, masterKey);
    return base64.encode(new TextEncoder().encode(JSON.stringify(encrypted)));
  }

  /**
   * Decrypt sensitive data from storage
   */
  static decryptFromStorage(encryptedData: string, masterKey: string): any {
    try {
      const decoded = new TextDecoder().decode(base64.decode(encryptedData));
      const encryptedObj = JSON.parse(decoded) as EncryptedData;
      const decrypted = this.decrypt(encryptedObj, masterKey);
      return JSON.parse(decrypted);
    } catch (error) {
      throw new Error(`Failed to decrypt data from storage: ${error}`);
    }
  }

  /**
   * Generate deterministic address from public key
   */
  static generateAddress(publicKey: Uint8Array, prefix: string = 'fin'): string {
    const hash = this.sha256Hash(publicKey);
    const checksum = hash.slice(0, 8);
    const address = `${prefix}1${base58.encode(Buffer.concat([publicKey, Buffer.from(checksum, 'hex')]))}`;
    return address;
  }

  /**
   * Validate address format and checksum
   */
  static validateAddress(address: string, prefix: string = 'fin'): boolean {
    try {
      if (!address.startsWith(`${prefix}1`)) return false;
      
      const decoded = base58.decode(address.slice(prefix.length + 1));
      if (decoded.length !== 40) return false; // 32 bytes pubkey + 8 bytes checksum
      
      const publicKey = decoded.slice(0, 32);
      const checksum = decoded.slice(32);
      const expectedChecksum = this.sha256Hash(publicKey).slice(0, 8);
      
      return Buffer.from(checksum).toString('hex') === expectedChecksum;
    } catch (error) {
      return false;
    }
  }

  /**
   * Create secure backup seed phrase
   */
  static generateSeedPhrase(entropy?: Uint8Array): string {
    const wordlist = [
      'abandon', 'ability', 'able', 'about', 'above', 'absent', 'absorb', 'abstract',
      'absurd', 'abuse', 'access', 'accident', 'account', 'accuse', 'achieve', 'acid',
      // ... add full BIP39 wordlist for production use
      'zone', 'zoo'
    ];
    
    const entropyBytes = entropy || this.generateRandomBytes(16); // 128 bits
    const words: string[] = [];
    
    for (let i = 0; i < entropyBytes.length; i += 2) {
      const index = (entropyBytes[i] << 8) | entropyBytes[i + 1];
      words.push(wordlist[index % wordlist.length]);
    }
    
    return words.join(' ');
  }

  /**
   * Derive keypair from seed phrase
   */
  static async deriveKeypairFromSeedPhrase(seedPhrase: string): Promise<KeyPair> {
    const seed = this.sha512Hash(seedPhrase).slice(0, 64); // 256 bits
    return this.generateKeypairFromSeed(seed);
  }
}

// Export utility functions
export const crypto = FinovaCrypto;
