import 'package:flutter/material.dart';
import 'package:uuid/uuid.dart';
import '../models/server.dart';

class ServerFormScreen extends StatefulWidget {
  final Server? server; // null = add new, non-null = edit

  const ServerFormScreen({super.key, this.server});

  @override
  State<ServerFormScreen> createState() => _ServerFormScreenState();
}

class _ServerFormScreenState extends State<ServerFormScreen> {
  final _formKey = GlobalKey<FormState>();
  late final TextEditingController _nameCtrl;
  late final TextEditingController _hostCtrl;
  late final TextEditingController _portCtrl;
  late final TextEditingController _userCtrl;
  late final TextEditingController _passwordCtrl;
  late final TextEditingController _identityFileCtrl;
  late final TextEditingController _keyCtrl;
  late final TextEditingController _passphraseCtrl;
  late final TextEditingController _notesCtrl;
  late String _authType;
  bool _showPassword = false;

  @override
  void initState() {
    super.initState();
    final s = widget.server;
    _nameCtrl = TextEditingController(text: s?.name ?? '');
    _hostCtrl = TextEditingController(text: s?.host ?? '');
    _portCtrl = TextEditingController(text: (s?.port ?? 22).toString());
    _userCtrl = TextEditingController(text: s?.username ?? 'root');
    _passwordCtrl = TextEditingController(text: s?.password ?? '');
    _identityFileCtrl = TextEditingController(text: s?.identityFile ?? '');
    _keyCtrl = TextEditingController(text: s?.privateKey ?? '');
    _passphraseCtrl = TextEditingController(text: s?.passphrase ?? '');
    _notesCtrl = TextEditingController(text: s?.notes ?? '');
    _authType = s?.authType ?? 'password';
  }

  @override
  void dispose() {
    _nameCtrl.dispose();
    _hostCtrl.dispose();
    _portCtrl.dispose();
    _userCtrl.dispose();
    _passwordCtrl.dispose();
    _identityFileCtrl.dispose();
    _keyCtrl.dispose();
    _passphraseCtrl.dispose();
    _notesCtrl.dispose();
    super.dispose();
  }

  void _save() {
    if (!_formKey.currentState!.validate()) return;

    final server = Server(
      id: widget.server?.id ?? const Uuid().v4(),
      name: _nameCtrl.text.trim(),
      host: _hostCtrl.text.trim(),
      port: int.tryParse(_portCtrl.text) ?? 22,
      username: _userCtrl.text.trim(),
      authType: _authType,
      identityFile: _authType == 'key'
          ? (_identityFileCtrl.text.trim().isEmpty
              ? null
              : _identityFileCtrl.text.trim())
          : null,
      password: _authType == 'password' ? _passwordCtrl.text : null,
      privateKey: _authType == 'key'
          ? (_keyCtrl.text.trim().isEmpty ? null : _keyCtrl.text)
          : null,
      passphrase: _authType == 'key'
          ? (_passphraseCtrl.text.isEmpty ? null : _passphraseCtrl.text)
          : null,
      notes: _notesCtrl.text.trim().isEmpty ? null : _notesCtrl.text.trim(),
      createdAt: widget.server?.createdAt ?? DateTime.now(),
      updatedAt: DateTime.now(),
    );

    Navigator.of(context).pop(server);
  }

  @override
  Widget build(BuildContext context) {
    final isEditing = widget.server != null;
    return Scaffold(
      appBar: AppBar(
        title: Text(isEditing ? 'Edit Server' : 'Add Server'),
        actions: [
          TextButton.icon(
            onPressed: _save,
            icon: const Icon(Icons.check),
            label: const Text('Save'),
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(20),
        child: Form(
          key: _formKey,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // -- Name (Host alias) --
              TextFormField(
                controller: _nameCtrl,
                decoration: const InputDecoration(
                  labelText: 'Host Alias (Name)',
                  hintText: 'e.g. prod-web  (used as SSH config Host)',
                  prefixIcon: Icon(Icons.label),
                  border: OutlineInputBorder(),
                ),
                validator: (v) {
                  if (v == null || v.trim().isEmpty) return 'Name is required';
                  if (v.trim().contains(' ')) return 'No spaces (SSH Host alias)';
                  return null;
                },
              ),
              const SizedBox(height: 4),
              Padding(
                padding: const EdgeInsets.only(left: 12),
                child: Text(
                  'This becomes the "Host" value in SSH config',
                  style: TextStyle(fontSize: 12, color: Colors.grey.shade500),
                ),
              ),
              const SizedBox(height: 16),

              // -- HostName + Port --
              Row(
                children: [
                  Expanded(
                    flex: 3,
                    child: TextFormField(
                      controller: _hostCtrl,
                      decoration: const InputDecoration(
                        labelText: 'HostName',
                        hintText: 'e.g. 192.168.1.100 or example.com',
                        prefixIcon: Icon(Icons.dns),
                        border: OutlineInputBorder(),
                      ),
                      validator: (v) => v == null || v.trim().isEmpty
                          ? 'HostName is required'
                          : null,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    child: TextFormField(
                      controller: _portCtrl,
                      decoration: const InputDecoration(
                        labelText: 'Port',
                        border: OutlineInputBorder(),
                      ),
                      keyboardType: TextInputType.number,
                      validator: (v) {
                        final port = int.tryParse(v ?? '');
                        if (port == null || port < 1 || port > 65535) {
                          return 'Invalid';
                        }
                        return null;
                      },
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 16),

              // -- User --
              TextFormField(
                controller: _userCtrl,
                decoration: const InputDecoration(
                  labelText: 'User',
                  prefixIcon: Icon(Icons.person),
                  border: OutlineInputBorder(),
                ),
                validator: (v) => v == null || v.trim().isEmpty
                    ? 'User is required'
                    : null,
              ),
              const SizedBox(height: 20),

              // -- Auth type --
              Text('Authentication',
                  style: Theme.of(context).textTheme.titleSmall),
              const SizedBox(height: 8),
              SegmentedButton<String>(
                segments: const [
                  ButtonSegment(
                    value: 'password',
                    label: Text('Password'),
                    icon: Icon(Icons.password),
                  ),
                  ButtonSegment(
                    value: 'key',
                    label: Text('SSH Key'),
                    icon: Icon(Icons.vpn_key),
                  ),
                ],
                selected: {_authType},
                onSelectionChanged: (s) =>
                    setState(() => _authType = s.first),
              ),
              const SizedBox(height: 16),

              if (_authType == 'password')
                TextFormField(
                  controller: _passwordCtrl,
                  obscureText: !_showPassword,
                  decoration: InputDecoration(
                    labelText: 'Password',
                    prefixIcon: const Icon(Icons.lock),
                    border: const OutlineInputBorder(),
                    suffixIcon: IconButton(
                      icon: Icon(_showPassword
                          ? Icons.visibility_off
                          : Icons.visibility),
                      onPressed: () =>
                          setState(() => _showPassword = !_showPassword),
                    ),
                  ),
                ),

              if (_authType == 'key') ...[
                TextFormField(
                  controller: _identityFileCtrl,
                  decoration: const InputDecoration(
                    labelText: 'IdentityFile (path)',
                    hintText: 'e.g. ~/.ssh/id_rsa',
                    prefixIcon: Icon(Icons.folder_open),
                    border: OutlineInputBorder(),
                    helperText: 'SSH config IdentityFile path (desktop)',
                  ),
                ),
                const SizedBox(height: 12),
                TextFormField(
                  controller: _keyCtrl,
                  maxLines: 5,
                  decoration: const InputDecoration(
                    labelText: 'Private Key Content (optional)',
                    hintText: '-----BEGIN OPENSSH PRIVATE KEY-----\n...',
                    prefixIcon: Icon(Icons.vpn_key),
                    border: OutlineInputBorder(),
                    alignLabelWithHint: true,
                    helperText: 'Inline key for mobile / cloud sync',
                  ),
                  style: const TextStyle(fontFamily: 'monospace', fontSize: 12),
                ),
                const SizedBox(height: 12),
                TextFormField(
                  controller: _passphraseCtrl,
                  obscureText: true,
                  decoration: const InputDecoration(
                    labelText: 'Key Passphrase (optional)',
                    prefixIcon: Icon(Icons.lock_outline),
                    border: OutlineInputBorder(),
                  ),
                ),
              ],

              const SizedBox(height: 16),
              TextFormField(
                controller: _notesCtrl,
                maxLines: 2,
                decoration: const InputDecoration(
                  labelText: 'Notes (optional)',
                  prefixIcon: Icon(Icons.notes),
                  border: OutlineInputBorder(),
                  alignLabelWithHint: true,
                ),
              ),

              const SizedBox(height: 12),
              // SSH config preview
              Container(
                width: double.infinity,
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  color: Theme.of(context).colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('SSH Config Preview',
                        style: Theme.of(context).textTheme.labelMedium),
                    const SizedBox(height: 4),
                    Text(
                      _buildPreview(),
                      style: const TextStyle(
                        fontFamily: 'monospace',
                        fontSize: 12,
                      ),
                    ),
                  ],
                ),
              ),

              const SizedBox(height: 24),
              SizedBox(
                width: double.infinity,
                height: 48,
                child: FilledButton.icon(
                  onPressed: _save,
                  icon: const Icon(Icons.save),
                  label: Text(isEditing ? 'Update Server' : 'Add Server'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  String _buildPreview() {
    final name = _nameCtrl.text.trim().isEmpty ? '<alias>' : _nameCtrl.text.trim();
    final host = _hostCtrl.text.trim().isEmpty ? '<hostname>' : _hostCtrl.text.trim();
    final port = _portCtrl.text.trim();
    final user = _userCtrl.text.trim().isEmpty ? 'root' : _userCtrl.text.trim();
    final identity = _identityFileCtrl.text.trim();

    final buf = StringBuffer();
    buf.writeln('Host $name');
    buf.writeln('    HostName $host');
    if (port.isNotEmpty && port != '22') buf.writeln('    Port $port');
    buf.writeln('    User $user');
    if (_authType == 'key' && identity.isNotEmpty) {
      buf.writeln('    IdentityFile $identity');
    }
    return buf.toString().trimRight();
  }
}
