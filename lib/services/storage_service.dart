import 'package:shared_preferences/shared_preferences.dart';
import '../models/server.dart';
import 'crypto_service.dart';

class StorageService {
  static const _serversKey = 'encrypted_servers';
  static const _encKeyKey = 'encryption_key';
  static const _tokenKey = 'github_token';
  static const _gistIdKey = 'gist_id';
  static const _usernameKey = 'github_username';
  static const _avatarKey = 'github_avatar';

  late SharedPreferences _prefs;

  Future<void> init() async {
    _prefs = await SharedPreferences.getInstance();
  }

  /// Returns or generates the local encryption key.
  String getEncryptionKey() {
    var key = _prefs.getString(_encKeyKey);
    if (key == null) {
      key = CryptoService.generateRandomKey();
      _prefs.setString(_encKeyKey, key);
    }
    return key;
  }

  Future<List<Server>> loadServers() async {
    final encrypted = _prefs.getString(_serversKey);
    if (encrypted == null || encrypted.isEmpty) return [];
    try {
      final decrypted =
          CryptoService.decryptData(encrypted, getEncryptionKey());
      return Server.decodeList(decrypted);
    } catch (_) {
      return [];
    }
  }

  Future<void> saveServers(List<Server> servers) async {
    final json = Server.encodeList(servers);
    final encrypted = CryptoService.encryptData(json, getEncryptionKey());
    await _prefs.setString(_serversKey, encrypted);
  }

  // --- GitHub auth state ---
  String? get githubToken => _prefs.getString(_tokenKey);
  Future<void> setGithubToken(String token) =>
      _prefs.setString(_tokenKey, token);
  Future<void> clearGithubToken() => _prefs.remove(_tokenKey);

  String? get gistId => _prefs.getString(_gistIdKey);
  Future<void> setGistId(String id) => _prefs.setString(_gistIdKey, id);

  String? get githubUsername => _prefs.getString(_usernameKey);
  Future<void> setGithubUsername(String u) =>
      _prefs.setString(_usernameKey, u);

  String? get githubAvatar => _prefs.getString(_avatarKey);
  Future<void> setGithubAvatar(String url) =>
      _prefs.setString(_avatarKey, url);

  bool get isLoggedIn => githubToken != null;

  Future<void> logout() async {
    await clearGithubToken();
    await _prefs.remove(_gistIdKey);
    await _prefs.remove(_usernameKey);
    await _prefs.remove(_avatarKey);
  }

  /// Returns the raw encrypted data for sync upload.
  String? getRawEncrypted() => _prefs.getString(_serversKey);

  /// Sets raw encrypted data from sync download.
  Future<void> setRawEncrypted(String data) =>
      _prefs.setString(_serversKey, data);
}
