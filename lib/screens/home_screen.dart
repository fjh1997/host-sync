import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/server.dart';
import '../services/storage_service.dart';
import '../services/sync_service.dart';
import '../services/terminal_service.dart';
import '../services/ssh_config_service.dart';
import '../widgets/server_card.dart';
import 'login_screen.dart';
import 'server_form_screen.dart';
import 'terminal_screen.dart';

class HomeScreen extends StatefulWidget {
  final StorageService storage;
  const HomeScreen({super.key, required this.storage});

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  List<Server> _servers = [];
  bool _syncing = false;
  String? _searchQuery;

  @override
  void initState() {
    super.initState();
    _loadServers();
  }

  Future<void> _loadServers() async {
    final servers = await widget.storage.loadServers();
    setState(() => _servers = servers);
  }

  Future<void> _saveAndReload() async {
    await widget.storage.saveServers(_servers);
    setState(() {});
  }

  List<Server> get _filteredServers {
    if (_searchQuery == null || _searchQuery!.isEmpty) return _servers;
    final q = _searchQuery!.toLowerCase();
    return _servers
        .where((s) =>
            s.name.toLowerCase().contains(q) ||
            s.host.toLowerCase().contains(q) ||
            s.username.toLowerCase().contains(q))
        .toList();
  }

  // ── Server CRUD ─────────────────────────────────────────────────

  Future<void> _addServer() async {
    final result = await Navigator.of(context).push<Server>(
      MaterialPageRoute(builder: (_) => const ServerFormScreen()),
    );
    if (result != null) {
      _servers.add(result);
      await _saveAndReload();
    }
  }

  Future<void> _editServer(Server server) async {
    final result = await Navigator.of(context).push<Server>(
      MaterialPageRoute(builder: (_) => ServerFormScreen(server: server)),
    );
    if (result != null) {
      final idx = _servers.indexWhere((s) => s.id == result.id);
      if (idx >= 0) {
        _servers[idx] = result;
        await _saveAndReload();
      }
    }
  }

  Future<void> _deleteServer(Server server) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete Server'),
        content: Text('Delete "${server.name}"? This cannot be undone.'),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(ctx, false),
              child: const Text('Cancel')),
          TextButton(
              onPressed: () => Navigator.pop(ctx, true),
              child:
                  const Text('Delete', style: TextStyle(color: Colors.red))),
        ],
      ),
    );
    if (confirm == true) {
      _servers.removeWhere((s) => s.id == server.id);
      await _saveAndReload();
    }
  }

  Future<void> _connectToServer(Server server) async {
    if (TerminalService.isDesktop) {
      final ok = await TerminalService.launchNativeTerminal(server);
      if (!ok && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Failed to open terminal')),
        );
      }
    } else {
      if (!mounted) return;
      Navigator.of(context).push(
        MaterialPageRoute(builder: (_) => TerminalScreen(server: server)),
      );
    }
  }

  // ── Cloud sync ──────────────────────────────────────────────────

  Future<void> _syncUpload() async {
    setState(() => _syncing = true);
    final sync = SyncService(widget.storage);
    final ok = await sync.upload();
    if (mounted) {
      setState(() => _syncing = false);
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(
        content: Text(ok ? 'Uploaded to cloud' : 'Upload failed'),
      ));
    }
  }

  Future<void> _syncDownload() async {
    setState(() => _syncing = true);
    final sync = SyncService(widget.storage);
    final ok = await sync.download();
    if (ok) await _loadServers();
    if (mounted) {
      setState(() => _syncing = false);
      ScaffoldMessenger.of(context).showSnackBar(SnackBar(
        content: Text(ok ? 'Downloaded from cloud' : 'Download failed'),
      ));
    }
  }

  // ── SSH Config import / export ──────────────────────────────────

  Future<void> _importSshConfig() async {
    final source = await showDialog<String>(
      context: context,
      builder: (ctx) => SimpleDialog(
        title: const Text('Import SSH Config'),
        children: [
          if (TerminalService.isDesktop)
            SimpleDialogOption(
              onPressed: () => Navigator.pop(ctx, 'system'),
              child: const ListTile(
                leading: Icon(Icons.computer),
                title: Text('From ~/.ssh/config'),
                subtitle: Text('Import system SSH config'),
              ),
            ),
          SimpleDialogOption(
            onPressed: () => Navigator.pop(ctx, 'paste'),
            child: const ListTile(
              leading: Icon(Icons.paste),
              title: Text('Paste config text'),
              subtitle: Text('Paste SSH config content'),
            ),
          ),
        ],
      ),
    );

    if (source == null || !mounted) return;

    List<Server> imported = [];

    if (source == 'system') {
      imported = await SshConfigService.parseSystemConfig();
    } else if (source == 'paste') {
      final text = await _showPasteDialog();
      if (text != null && text.isNotEmpty) {
        imported = SshConfigService.parse(text);
      }
    }

    if (imported.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('No hosts found to import')),
        );
      }
      return;
    }

    // Show what will be imported
    if (!mounted) return;
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: Text('Import ${imported.length} host(s)?'),
        content: SizedBox(
          width: double.maxFinite,
          child: ListView.builder(
            shrinkWrap: true,
            itemCount: imported.length,
            itemBuilder: (_, i) => ListTile(
              dense: true,
              leading: const Icon(Icons.computer, size: 20),
              title: Text(imported[i].name),
              subtitle: Text(
                  '${imported[i].username}@${imported[i].host}:${imported[i].port}'),
            ),
          ),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(ctx, false),
              child: const Text('Cancel')),
          FilledButton(
              onPressed: () => Navigator.pop(ctx, true),
              child: const Text('Import')),
        ],
      ),
    );

    if (confirmed == true) {
      // Merge: skip hosts whose name already exists
      final existingNames = _servers.map((s) => s.name).toSet();
      var added = 0;
      for (final s in imported) {
        if (!existingNames.contains(s.name)) {
          _servers.add(s);
          existingNames.add(s.name);
          added++;
        }
      }
      await _saveAndReload();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(SnackBar(
          content: Text('Imported $added new host(s)'),
        ));
      }
    }
  }

  Future<String?> _showPasteDialog() async {
    final ctrl = TextEditingController();
    final result = await showDialog<String>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Paste SSH Config'),
        content: SizedBox(
          width: double.maxFinite,
          height: 300,
          child: TextField(
            controller: ctrl,
            maxLines: null,
            expands: true,
            textAlignVertical: TextAlignVertical.top,
            style: const TextStyle(fontFamily: 'monospace', fontSize: 13),
            decoration: const InputDecoration(
              hintText: 'Host myserver\n    HostName 192.168.1.1\n    User root\n    Port 22\n    IdentityFile ~/.ssh/id_rsa',
              border: OutlineInputBorder(),
              alignLabelWithHint: true,
            ),
          ),
        ),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(ctx),
              child: const Text('Cancel')),
          FilledButton(
              onPressed: () => Navigator.pop(ctx, ctrl.text),
              child: const Text('Parse')),
        ],
      ),
    );
    ctrl.dispose();
    return result;
  }

  Future<void> _exportSshConfig() async {
    if (_servers.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('No servers to export')),
      );
      return;
    }

    final target = await showDialog<String>(
      context: context,
      builder: (ctx) => SimpleDialog(
        title: const Text('Export SSH Config'),
        children: [
          SimpleDialogOption(
            onPressed: () => Navigator.pop(ctx, 'clipboard'),
            child: const ListTile(
              leading: Icon(Icons.copy),
              title: Text('Copy to clipboard'),
              subtitle: Text('Standard SSH config format'),
            ),
          ),
          if (TerminalService.isDesktop)
            SimpleDialogOption(
              onPressed: () => Navigator.pop(ctx, 'system'),
              child: const ListTile(
                leading: Icon(Icons.save_alt),
                title: Text('Merge into ~/.ssh/config'),
                subtitle: Text('Appends HostSync entries'),
              ),
            ),
        ],
      ),
    );

    if (target == null || !mounted) return;

    if (target == 'clipboard') {
      final config = SshConfigService.generate(_servers);
      await Clipboard.setData(ClipboardData(text: config));
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('SSH config copied to clipboard')),
        );
      }
    } else if (target == 'system') {
      final confirm = await showDialog<bool>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Merge into system SSH config?'),
          content: const Text(
              'This will add/update HostSync-managed Host entries in '
              '~/.ssh/config. Existing non-HostSync entries are preserved.'),
          actions: [
            TextButton(
                onPressed: () => Navigator.pop(ctx, false),
                child: const Text('Cancel')),
            FilledButton(
                onPressed: () => Navigator.pop(ctx, true),
                child: const Text('Merge')),
          ],
        ),
      );
      if (confirm == true) {
        await SshConfigService.mergeIntoSystemConfig(_servers);
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(
                content: Text('Merged into ~/.ssh/config successfully')),
          );
        }
      }
    }
  }

  // ── Logout ──────────────────────────────────────────────────────

  Future<void> _logout() async {
    await widget.storage.logout();
    if (!mounted) return;
    Navigator.of(context).pushAndRemoveUntil(
      MaterialPageRoute(
          builder: (_) => LoginScreen(storage: widget.storage)),
      (_) => false,
    );
  }

  // ── Build ───────────────────────────────────────────────────────

  @override
  Widget build(BuildContext context) {
    final username = widget.storage.githubUsername ?? 'User';
    final avatar = widget.storage.githubAvatar;

    return Scaffold(
      appBar: AppBar(
        title: const Text('HostSync'),
        actions: [
          // Sync buttons
          if (_syncing)
            const Padding(
              padding: EdgeInsets.all(12),
              child: SizedBox(
                  width: 24,
                  height: 24,
                  child: CircularProgressIndicator(strokeWidth: 2)),
            )
          else ...[
            IconButton(
              icon: const Icon(Icons.cloud_upload_outlined),
              tooltip: 'Upload to cloud',
              onPressed: _syncUpload,
            ),
            IconButton(
              icon: const Icon(Icons.cloud_download_outlined),
              tooltip: 'Download from cloud',
              onPressed: _syncDownload,
            ),
          ],
          // SSH Config & account menu
          PopupMenuButton<String>(
            onSelected: (v) {
              switch (v) {
                case 'import':
                  _importSshConfig();
                case 'export':
                  _exportSshConfig();
                case 'logout':
                  _logout();
              }
            },
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (avatar != null)
                    CircleAvatar(
                      radius: 14,
                      backgroundImage: NetworkImage(avatar),
                    )
                  else
                    const CircleAvatar(
                        radius: 14, child: Icon(Icons.person, size: 18)),
                  const SizedBox(width: 6),
                  Text(username, style: const TextStyle(fontSize: 14)),
                  const Icon(Icons.arrow_drop_down),
                ],
              ),
            ),
            itemBuilder: (_) => [
              const PopupMenuItem(
                value: 'import',
                child: ListTile(
                  leading: Icon(Icons.file_download, size: 20),
                  title: Text('Import SSH Config'),
                  dense: true,
                ),
              ),
              const PopupMenuItem(
                value: 'export',
                child: ListTile(
                  leading: Icon(Icons.file_upload, size: 20),
                  title: Text('Export SSH Config'),
                  dense: true,
                ),
              ),
              const PopupMenuDivider(),
              const PopupMenuItem(
                value: 'logout',
                child: ListTile(
                  leading: Icon(Icons.logout, size: 20),
                  title: Text('Logout'),
                  dense: true,
                ),
              ),
            ],
          ),
        ],
      ),
      body: Column(
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
            child: TextField(
              decoration: InputDecoration(
                hintText: 'Search servers...',
                prefixIcon: const Icon(Icons.search),
                border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(12)),
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                isDense: true,
              ),
              onChanged: (v) => setState(() => _searchQuery = v),
            ),
          ),
          Expanded(
            child: _filteredServers.isEmpty
                ? Center(
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Icon(Icons.dns_outlined,
                            size: 64, color: Colors.grey.shade400),
                        const SizedBox(height: 16),
                        Text(
                          _servers.isEmpty
                              ? 'No servers yet'
                              : 'No matching servers',
                          style: TextStyle(color: Colors.grey.shade500),
                        ),
                        if (_servers.isEmpty) ...[
                          const SizedBox(height: 12),
                          TextButton.icon(
                            onPressed: _addServer,
                            icon: const Icon(Icons.add),
                            label: const Text('Add your first server'),
                          ),
                          const SizedBox(height: 8),
                          TextButton.icon(
                            onPressed: _importSshConfig,
                            icon: const Icon(Icons.file_download),
                            label: const Text('Import from SSH config'),
                          ),
                        ],
                      ],
                    ),
                  )
                : ListView.builder(
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    itemCount: _filteredServers.length,
                    itemBuilder: (_, i) {
                      final server = _filteredServers[i];
                      return ServerCard(
                        server: server,
                        onConnect: () => _connectToServer(server),
                        onEdit: () => _editServer(server),
                        onDelete: () => _deleteServer(server),
                      );
                    },
                  ),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton(
        onPressed: _addServer,
        child: const Icon(Icons.add),
      ),
    );
  }
}
