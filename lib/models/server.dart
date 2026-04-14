import 'dart:convert';

class Server {
  final String id;
  String name;       // SSH config: Host alias
  String host;       // SSH config: HostName
  int port;          // SSH config: Port
  String username;   // SSH config: User
  String authType;   // 'password' or 'key'
  String? identityFile; // SSH config: IdentityFile (path to key)
  String? password;
  String? privateKey;   // inline key content (for mobile / sync)
  String? passphrase;
  String? notes;
  DateTime createdAt;
  DateTime updatedAt;

  Server({
    required this.id,
    required this.name,
    required this.host,
    this.port = 22,
    required this.username,
    this.authType = 'password',
    this.identityFile,
    this.password,
    this.privateKey,
    this.passphrase,
    this.notes,
    DateTime? createdAt,
    DateTime? updatedAt,
  })  : createdAt = createdAt ?? DateTime.now(),
        updatedAt = updatedAt ?? DateTime.now();

  /// The SSH config–compatible Host block for this server.
  String toSshConfigBlock() {
    final buf = StringBuffer();
    buf.writeln('Host $name');
    buf.writeln('    HostName $host');
    if (port != 22) buf.writeln('    Port $port');
    buf.writeln('    User $username');
    if (identityFile != null && identityFile!.isNotEmpty) {
      buf.writeln('    IdentityFile $identityFile');
    }
    return buf.toString();
  }

  Map<String, dynamic> toJson() => {
        'id': id,
        'name': name,
        'host': host,
        'port': port,
        'username': username,
        'authType': authType,
        'identityFile': identityFile,
        'password': password,
        'privateKey': privateKey,
        'passphrase': passphrase,
        'notes': notes,
        'createdAt': createdAt.toIso8601String(),
        'updatedAt': updatedAt.toIso8601String(),
      };

  factory Server.fromJson(Map<String, dynamic> json) => Server(
        id: json['id'] as String,
        name: json['name'] as String,
        host: json['host'] as String,
        port: json['port'] as int? ?? 22,
        username: json['username'] as String,
        authType: json['authType'] as String? ?? 'password',
        identityFile: json['identityFile'] as String?,
        password: json['password'] as String?,
        privateKey: json['privateKey'] as String?,
        passphrase: json['passphrase'] as String?,
        notes: json['notes'] as String?,
        createdAt: DateTime.parse(json['createdAt'] as String),
        updatedAt: DateTime.parse(json['updatedAt'] as String),
      );

  static String encodeList(List<Server> servers) =>
      jsonEncode(servers.map((s) => s.toJson()).toList());

  static List<Server> decodeList(String data) =>
      (jsonDecode(data) as List).map((e) => Server.fromJson(e)).toList();
}
