import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:http/http.dart' as http;
import 'storage_service.dart';

class AuthService {
  // Users must create their own GitHub OAuth App and fill these in.
  // Settings > Developer settings > OAuth Apps > New OAuth App
  // Callback URL: http://localhost:9876/callback
  static const clientId = 'YOUR_GITHUB_CLIENT_ID';
  static const clientSecret = 'YOUR_GITHUB_CLIENT_SECRET';
  static const _redirectUri = 'http://localhost:9876/callback';
  static const _scopes = 'gist,read:user';

  final StorageService _storage;

  AuthService(this._storage);

  /// Starts GitHub OAuth Device Flow or browser-based flow.
  /// Returns the access token on success.
  Future<String?> login() async {
    final completer = Completer<String?>();

    // Start local HTTP server to receive the callback
    final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 9876);

    // Build authorization URL
    final authUrl = Uri.https('github.com', '/login/oauth/authorize', {
      'client_id': clientId,
      'redirect_uri': _redirectUri,
      'scope': _scopes,
      'state': DateTime.now().millisecondsSinceEpoch.toString(),
    });

    // Open browser (handled by caller via url_launcher)
    // We return the URL and listen for callback
    _authUrl = authUrl;

    server.listen((request) async {
      try {
        if (request.uri.path == '/callback') {
          final code = request.uri.queryParameters['code'];
          if (code != null) {
            // Exchange code for token
            final tokenResponse = await http.post(
              Uri.https('github.com', '/login/oauth/access_token'),
              headers: {'Accept': 'application/json'},
              body: {
                'client_id': clientId,
                'client_secret': clientSecret,
                'code': code,
                'redirect_uri': _redirectUri,
              },
            );

            final tokenData = jsonDecode(tokenResponse.body);
            final token = tokenData['access_token'] as String?;

            if (token != null) {
              await _storage.setGithubToken(token);
              await _fetchUserInfo(token);

              request.response
                ..statusCode = 200
                ..headers.contentType = ContentType.html
                ..write('''
                  <html><body style="display:flex;justify-content:center;align-items:center;height:100vh;font-family:system-ui;background:#0d1117;color:#f0f6fc;">
                  <div style="text-align:center"><h1>&#10004; Login Successful</h1><p>You can close this window and return to HostSync.</p></div>
                  </body></html>
                ''');
              await request.response.close();
              completer.complete(token);
            } else {
              request.response
                ..statusCode = 400
                ..write('Failed to obtain token');
              await request.response.close();
              completer.complete(null);
            }
          }
        }
      } catch (e) {
        completer.complete(null);
      } finally {
        await server.close();
      }
    });

    // Auto-timeout after 2 minutes
    Future.delayed(const Duration(minutes: 2), () {
      if (!completer.isCompleted) {
        server.close();
        completer.complete(null);
      }
    });

    return completer.future;
  }

  Uri? _authUrl;
  Uri? get authUrl => _authUrl;

  Future<void> _fetchUserInfo(String token) async {
    final response = await http.get(
      Uri.https('api.github.com', '/user'),
      headers: {
        'Authorization': 'Bearer $token',
        'Accept': 'application/vnd.github+json',
      },
    );
    if (response.statusCode == 200) {
      final data = jsonDecode(response.body);
      await _storage.setGithubUsername(data['login'] ?? '');
      await _storage.setGithubAvatar(data['avatar_url'] ?? '');
    }
  }

  Future<void> logout() async {
    await _storage.logout();
  }
}
