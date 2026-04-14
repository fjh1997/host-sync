import 'dart:io';
import 'package:path_provider/path_provider.dart';
import '../models/server.dart';

class TerminalService {
  static bool get isDesktop =>
      Platform.isWindows || Platform.isLinux || Platform.isMacOS;

  static bool get isMobile => Platform.isAndroid || Platform.isIOS;

  /// Launches the system's native terminal with an SSH command.
  ///
  /// Strategy:
  ///  1. Write a temporary SSH config file containing the server block.
  ///  2. Invoke `ssh -F <config> <Host>` so the native SSH client reads all
  ///     parameters (HostName, Port, User, IdentityFile) from the config.
  ///  This keeps the launched command clean and fully SSH-config-compatible.
  static Future<bool> launchNativeTerminal(Server server) async {
    try {
      // Write a temp config file for this connection
      final configPath = await _writeTempConfig(server);
      final sshArgs = ['-F', configPath, server.name];

      if (Platform.isWindows) {
        // Try Windows Terminal first, fallback to cmd
        await Process.start(
          'cmd',
          ['/c', 'start', 'wt', 'ssh', ...sshArgs],
          mode: ProcessStartMode.detached,
        ).catchError((_) async {
          return Process.start(
            'cmd',
            ['/c', 'start', 'cmd', '/k', 'ssh', ...sshArgs],
            mode: ProcessStartMode.detached,
          );
        });
        return true;
      } else if (Platform.isMacOS) {
        final escaped = sshArgs.map((a) => a.contains(' ') ? '"$a"' : a).join(' ');
        final script =
            'tell application "Terminal" to do script "ssh $escaped"';
        await Process.start('osascript', ['-e', script],
            mode: ProcessStartMode.detached);
        return true;
      } else if (Platform.isLinux) {
        for (final term in [
          'gnome-terminal',
          'konsole',
          'xfce4-terminal',
          'xterm'
        ]) {
          try {
            if (term == 'gnome-terminal') {
              await Process.start(term, ['--', 'ssh', ...sshArgs],
                  mode: ProcessStartMode.detached);
            } else if (term == 'konsole') {
              await Process.start(term, ['-e', 'ssh', ...sshArgs],
                  mode: ProcessStartMode.detached);
            } else {
              await Process.start(
                  term, ['-e', 'ssh ${sshArgs.join(' ')}'],
                  mode: ProcessStartMode.detached);
            }
            return true;
          } catch (_) {
            continue;
          }
        }
        return false;
      }
    } catch (_) {
      return false;
    }
    return false;
  }

  /// Writes a temporary SSH config file for the given server and returns
  /// its path. The file is placed in the app's temp directory.
  static Future<String> _writeTempConfig(Server server) async {
    final dir = await getTemporaryDirectory();
    final configDir = Directory('${dir.path}/hostsync_ssh');
    if (!await configDir.exists()) {
      await configDir.create(recursive: true);
    }

    // If there's an inline private key but no IdentityFile path, write the
    // key to a temp file and reference it.
    String? tempKeyPath;
    if (server.authType == 'key' &&
        server.privateKey != null &&
        server.privateKey!.isNotEmpty &&
        (server.identityFile == null || server.identityFile!.isEmpty)) {
      final keyFile = File('${configDir.path}/key_${server.id}');
      await keyFile.writeAsString(server.privateKey!);
      // SSH requires key files to have restricted permissions
      if (!Platform.isWindows) {
        await Process.run('chmod', ['600', keyFile.path]);
      }
      tempKeyPath = keyFile.path;
    }

    final effectiveIdentity =
        tempKeyPath ?? server.identityFile;

    final buf = StringBuffer();
    buf.writeln('Host ${server.name}');
    buf.writeln('    HostName ${server.host}');
    if (server.port != 22) buf.writeln('    Port ${server.port}');
    buf.writeln('    User ${server.username}');
    if (effectiveIdentity != null && effectiveIdentity.isNotEmpty) {
      buf.writeln('    IdentityFile $effectiveIdentity');
    }
    buf.writeln('    StrictHostKeyChecking no');
    buf.writeln('    UserKnownHostsFile /dev/null');

    final configFile = File('${configDir.path}/config_${server.id}');
    await configFile.writeAsString(buf.toString());
    return configFile.path;
  }
}
