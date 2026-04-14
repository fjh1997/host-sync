import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';
import 'package:encrypt/encrypt.dart' as encrypt;
import 'package:pointycastle/export.dart';

class CryptoService {
  static const int _keyLength = 32;
  static const int _ivLength = 16;
  static const int _saltLength = 16;
  static const int _iterations = 100000;

  /// Derives a 256-bit key from a passphrase using PBKDF2.
  static Uint8List _deriveKey(String passphrase, Uint8List salt) {
    final params = Pbkdf2Parameters(salt, _iterations, _keyLength);
    final generator = PBKDF2KeyDerivator(HMac(SHA256Digest(), 64))
      ..init(params);
    return generator.process(Uint8List.fromList(utf8.encode(passphrase)));
  }

  /// Encrypts plaintext with AES-256-CBC + PBKDF2 key derivation.
  /// Output format: base64(salt + iv + ciphertext)
  static String encryptData(String plaintext, String passphrase) {
    final random = Random.secure();
    final salt =
        Uint8List.fromList(List.generate(_saltLength, (_) => random.nextInt(256)));
    final iv =
        Uint8List.fromList(List.generate(_ivLength, (_) => random.nextInt(256)));

    final key = _deriveKey(passphrase, salt);
    final encrypter = encrypt.Encrypter(
        encrypt.AES(encrypt.Key(key), mode: encrypt.AESMode.cbc));
    final encrypted = encrypter.encrypt(plaintext, iv: encrypt.IV(iv));

    final combined = Uint8List.fromList([...salt, ...iv, ...encrypted.bytes]);
    return base64Encode(combined);
  }

  /// Decrypts data encrypted by [encryptData].
  static String decryptData(String ciphertext, String passphrase) {
    final combined = base64Decode(ciphertext);
    final salt = Uint8List.fromList(combined.sublist(0, _saltLength));
    final iv =
        Uint8List.fromList(combined.sublist(_saltLength, _saltLength + _ivLength));
    final encryptedBytes = Uint8List.fromList(combined.sublist(_saltLength + _ivLength));

    final key = _deriveKey(passphrase, salt);
    final encrypter = encrypt.Encrypter(
        encrypt.AES(encrypt.Key(key), mode: encrypt.AESMode.cbc));
    return encrypter.decrypt(encrypt.Encrypted(encryptedBytes),
        iv: encrypt.IV(iv));
  }

  /// Generates a random encryption key (hex encoded) for first-time setup.
  static String generateRandomKey() {
    final random = Random.secure();
    final bytes = List.generate(32, (_) => random.nextInt(256));
    return bytes.map((b) => b.toRadixString(16).padLeft(2, '0')).join();
  }
}
