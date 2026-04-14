import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:dartssh2/dartssh2.dart';
import 'package:xterm/xterm.dart';
import '../models/server.dart';

class TerminalScreen extends StatefulWidget {
  final Server server;
  const TerminalScreen({super.key, required this.server});

  @override
  State<TerminalScreen> createState() => _TerminalScreenState();
}

class _TerminalScreenState extends State<TerminalScreen> {
  late Terminal _terminal;
  SSHClient? _client;
  SSHSession? _session;
  bool _connected = false;
  bool _connecting = true;

  @override
  void initState() {
    super.initState();
    _terminal = Terminal(maxLines: 10000);
    _connect();
  }

  Future<void> _connect() async {
    _terminal.write('Connecting to ${widget.server.host}:${widget.server.port}...\r\n');

    try {
      final socket = await SSHSocket.connect(
        widget.server.host,
        widget.server.port,
        timeout: const Duration(seconds: 10),
      );

      if (widget.server.authType == 'key' &&
          widget.server.privateKey != null &&
          widget.server.privateKey!.isNotEmpty) {
        _client = SSHClient(
          socket,
          username: widget.server.username,
          identities: SSHKeyPair.fromPem(
            widget.server.privateKey!,
            widget.server.passphrase,
          ),
        );
      } else {
        _client = SSHClient(
          socket,
          username: widget.server.username,
          onPasswordRequest: () => widget.server.password ?? '',
        );
      }

      _session = await _client!.shell(
        pty: SSHPtyConfig(
          width: _terminal.viewWidth,
          height: _terminal.viewHeight,
        ),
      );

      setState(() {
        _connected = true;
        _connecting = false;
      });

      // Pipe SSH output to terminal
      _session!.stdout.listen((data) {
        _terminal.write(String.fromCharCodes(data));
      });

      _session!.stderr.listen((data) {
        _terminal.write(String.fromCharCodes(data));
      });

      // Pipe terminal input to SSH
      _terminal.onOutput = (data) {
        _session?.write(utf8.encode(data));
      };

      // Handle resize
      _terminal.onResize = (width, height, pixelWidth, pixelHeight) {
        _session?.resizeTerminal(width, height);
      };

      // Wait for session to end
      await _session!.done;
      if (mounted) {
        _terminal.write('\r\n[Connection closed]\r\n');
        setState(() => _connected = false);
      }
    } catch (e) {
      if (mounted) {
        _terminal.write('\r\n[Error: $e]\r\n');
        setState(() {
          _connecting = false;
          _connected = false;
        });
      }
    }
  }

  void _disconnect() {
    _session?.close();
    _client?.close();
    setState(() => _connected = false);
  }

  @override
  void dispose() {
    _session?.close();
    _client?.close();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(widget.server.name),
        actions: [
          if (_connecting)
            const Padding(
              padding: EdgeInsets.all(12),
              child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2)),
            ),
          if (_connected)
            IconButton(
              icon: const Icon(Icons.close),
              tooltip: 'Disconnect',
              onPressed: _disconnect,
            ),
          if (!_connected && !_connecting)
            IconButton(
              icon: const Icon(Icons.refresh),
              tooltip: 'Reconnect',
              onPressed: () {
                setState(() => _connecting = true);
                _connect();
              },
            ),
        ],
      ),
      body: TerminalView(
        _terminal,
        autofocus: true,
        padding: const EdgeInsets.all(4),
      ),
    );
  }
}
