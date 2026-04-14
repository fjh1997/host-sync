import 'dart:convert';
import 'package:http/http.dart' as http;
import 'storage_service.dart';

class SyncService {
  static const _gistFileName = 'hostsync_data.enc';
  static const _gistDescription = 'HostSync Encrypted Server Data';
  final StorageService _storage;

  SyncService(this._storage);

  Map<String, String> get _headers => {
        'Authorization': 'Bearer ${_storage.githubToken}',
        'Accept': 'application/vnd.github+json',
      };

  /// Uploads encrypted server data to GitHub Gist.
  Future<bool> upload() async {
    final data = _storage.getRawEncrypted();
    if (data == null || data.isEmpty) return false;

    final gistId = _storage.gistId;
    if (gistId != null) {
      return _updateGist(gistId, data);
    } else {
      return _createGist(data);
    }
  }

  /// Downloads encrypted data from GitHub Gist and saves locally.
  Future<bool> download() async {
    final gistId = _storage.gistId;
    if (gistId == null) {
      // Try to find existing gist
      final found = await _findGist();
      if (found == null) return false;
    }

    final response = await http.get(
      Uri.https('api.github.com', '/gists/${_storage.gistId}'),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      final gist = jsonDecode(response.body);
      final files = gist['files'] as Map<String, dynamic>;
      if (files.containsKey(_gistFileName)) {
        final content = files[_gistFileName]['content'] as String;
        await _storage.setRawEncrypted(content);
        return true;
      }
    }
    return false;
  }

  Future<bool> _createGist(String data) async {
    final response = await http.post(
      Uri.https('api.github.com', '/gists'),
      headers: _headers,
      body: jsonEncode({
        'description': _gistDescription,
        'public': false,
        'files': {
          _gistFileName: {'content': data},
        },
      }),
    );

    if (response.statusCode == 201) {
      final gist = jsonDecode(response.body);
      await _storage.setGistId(gist['id'] as String);
      return true;
    }
    return false;
  }

  Future<bool> _updateGist(String gistId, String data) async {
    final response = await http.patch(
      Uri.https('api.github.com', '/gists/$gistId'),
      headers: _headers,
      body: jsonEncode({
        'files': {
          _gistFileName: {'content': data},
        },
      }),
    );
    return response.statusCode == 200;
  }

  Future<String?> _findGist() async {
    final response = await http.get(
      Uri.https('api.github.com', '/gists', {'per_page': '100'}),
      headers: _headers,
    );

    if (response.statusCode == 200) {
      final gists = jsonDecode(response.body) as List;
      for (final gist in gists) {
        final files = gist['files'] as Map<String, dynamic>;
        if (files.containsKey(_gistFileName)) {
          final id = gist['id'] as String;
          await _storage.setGistId(id);
          return id;
        }
      }
    }
    return null;
  }
}
