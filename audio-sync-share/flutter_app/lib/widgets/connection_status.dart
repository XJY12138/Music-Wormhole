import 'package:flutter/material.dart';
import '../services/sync_service.dart';

class ConnectionStatus extends StatelessWidget {
  final SyncService syncService;

  const ConnectionStatus({super.key, required this.syncService});

  @override
  Widget build(BuildContext context) {
    if (!syncService.isConnected) {
      return const SizedBox.shrink();
    }

    return Container(
      decoration: BoxDecoration(
        color: const Color(0xFF1E1E3F),
        border: Border(
          top: BorderSide(
            color: Colors.white.withOpacity(0.1),
            width: 1,
          ),
        ),
      ),
      child: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              _buildConnectionInfo(),
              const SizedBox(height: 16),
              _buildPlaybackControls(),
              const SizedBox(height: 16),
              _buildSyncSettings(),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildConnectionInfo() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Row(
          children: [
            Container(
              width: 40,
              height: 40,
              decoration: BoxDecoration(
                color: const Color(0xFF6366F1).withOpacity(0.2),
                borderRadius: BorderRadius.circular(10),
              ),
              child: const Icon(
                Icons.audio_file,
                color: Color(0xFF6366F1),
                size: 22,
              ),
            ),
            const SizedBox(width: 12),
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Connected to ${syncService.connectedDevice?.name ?? "Unknown"}',
                  style: const TextStyle(
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    color: Colors.white,
                  ),
                ),
                Text(
                  '${syncService.currentLatency}ms latency • ${syncService.bitrate}kbps',
                  style: TextStyle(
                    fontSize: 12,
                    color: Colors.white.withOpacity(0.5),
                  ),
                ),
              ],
            ),
          ],
        ),
        IconButton(
          onPressed: () => syncService.disconnect(),
          icon: const Icon(Icons.close),
          color: Colors.white.withOpacity(0.7),
        ),
      ],
    );
  }

  Widget _buildPlaybackControls() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        _buildControlButton(
          icon: Icons.skip_previous,
          onPressed: syncService.playPrevious,
        ),
        const SizedBox(width: 16),
        _buildPlayPauseButton(),
        const SizedBox(width: 16),
        _buildControlButton(
          icon: Icons.skip_next,
          onPressed: syncService.playNext,
        ),
      ],
    );
  }

  Widget _buildPlayPauseButton() {
    return Container(
      width: 64,
      height: 64,
      decoration: BoxDecoration(
        gradient: const LinearGradient(
          colors: [Color(0xFF6366F1), Color(0xFF8B5CF6)],
        ),
        shape: BoxShape.circle,
        boxShadow: [
          BoxShadow(
            color: const Color(0xFF6366F1).withOpacity(0.4),
            blurRadius: 16,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: syncService.togglePlayback,
          borderRadius: BorderRadius.circular(32),
          child: Center(
            child: Icon(
              syncService.isPlaying ? Icons.pause : Icons.play_arrow,
              color: Colors.white,
              size: 32,
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildControlButton({
    required IconData icon,
    required VoidCallback onPressed,
  }) {
    return Container(
      width: 48,
      height: 48,
      decoration: BoxDecoration(
        color: Colors.white.withOpacity(0.1),
        shape: BoxShape.circle,
      ),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onPressed,
          borderRadius: BorderRadius.circular(24),
          child: Icon(
            icon,
            color: Colors.white.withOpacity(0.9),
            size: 24,
          ),
        ),
      ),
    );
  }

  Widget _buildSyncSettings() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        _buildSettingButton(
          icon: Icons.volume_up,
          label: 'Volume',
          value: '${syncService.volume}%',
          onTap: () => _showVolumeDialog(),
        ),
        _buildSettingButton(
          icon: Icons.tune,
          label: 'Sync',
          value: '${syncService.syncOffset}ms',
          onTap: () => _showSyncDialog(),
        ),
        _buildSettingButton(
          icon: Icons.devices,
          label: 'Source',
          value: syncService.isAllApps ? 'All' : 'App',
          onTap: () => _showSourceDialog(),
        ),
      ],
    );
  }

  Widget _buildSettingButton({
    required IconData icon,
    required String label,
    required String value,
    required VoidCallback onTap,
  }) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        decoration: BoxDecoration(
          color: Colors.white.withOpacity(0.05),
          borderRadius: BorderRadius.circular(10),
          border: Border.all(
            color: Colors.white.withOpacity(0.1),
          ),
        ),
        child: Column(
          children: [
            Icon(
              icon,
              color: Colors.white.withOpacity(0.7),
              size: 20,
            ),
            const SizedBox(height: 4),
            Text(
              value,
              style: const TextStyle(
                fontSize: 12,
                fontWeight: FontWeight.w600,
                color: Colors.white,
              ),
            ),
            Text(
              label,
              style: TextStyle(
                fontSize: 10,
                color: Colors.white.withOpacity(0.5),
              ),
            ),
          ],
        ),
      ),
    );
  }

  void _showVolumeDialog() {
    // TODO: Implement volume slider dialog
  }

  void _showSyncDialog() {
    // TODO: Implement sync adjustment dialog
  }

  void _showSourceDialog() {
    // TODO: Implement app selection dialog
  }
}
