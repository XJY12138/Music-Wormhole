import 'package:flutter/material.dart';
import '../models/device_info.dart';

class DeviceCard extends StatelessWidget {
  final DeviceInfo device;
  final bool isConnected;
  final VoidCallback onTap;

  const DeviceCard({
    super.key,
    required this.device,
    required this.isConnected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedContainer(
      duration: const Duration(milliseconds: 300),
      margin: const EdgeInsets.only(bottom: 12),
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(16),
          child: Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: isConnected
                  ? const Color(0xFF6366F1).withOpacity(0.15)
                  : Colors.white.withOpacity(0.05),
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: isConnected
                    ? const Color(0xFF6366F1).withOpacity(0.5)
                    : Colors.white.withOpacity(0.1),
                width: 1.5,
              ),
              boxShadow: [
                if (isConnected)
                  BoxShadow(
                    color: const Color(0xFF6366F1).withOpacity(0.2),
                    blurRadius: 12,
                    offset: const Offset(0, 4),
                  ),
              ],
            ),
            child: Row(
              children: [
                _buildPlatformIcon(),
                const SizedBox(width: 16),
                Expanded(child: _buildDeviceInfo()),
                _buildConnectionIndicator(),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildPlatformIcon() {
    return Container(
      width: 56,
      height: 56,
      decoration: BoxDecoration(
        color: device.platformColor.withOpacity(0.15),
        borderRadius: BorderRadius.circular(14),
        border: Border.all(
          color: device.platformColor.withOpacity(0.3),
          width: 1.5,
        ),
      ),
      child: Icon(
        device.platformIcon,
        color: device.platformColor,
        size: 28,
      ),
    );
  }

  Widget _buildDeviceInfo() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          device.name,
          style: const TextStyle(
            fontSize: 18,
            fontWeight: FontWeight.w600,
            color: Colors.white,
            letterSpacing: -0.3,
          ),
        ),
        const SizedBox(height: 6),
        Row(
          children: [
            Icon(
              Icons.wifi,
              size: 14,
              color: Colors.white.withOpacity(0.5),
            ),
            const SizedBox(width: 4),
            Text(
              device.ipAddress,
              style: TextStyle(
                fontSize: 13,
                color: Colors.white.withOpacity(0.5),
              ),
            ),
            if (device.latency > 0) ...[
              const SizedBox(width: 12),
              Icon(
                Icons.timer,
                size: 14,
                color: Colors.white.withOpacity(0.5),
              ),
              const SizedBox(width: 4),
              Text(
                '${device.latency}ms',
                style: TextStyle(
                  fontSize: 13,
                  color: _getLatencyColor(device.latency),
                  fontWeight: FontWeight.w500,
                ),
              ),
            ],
          ],
        ),
        const SizedBox(height: 4),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 3),
          decoration: BoxDecoration(
            color: device.isSender
                ? const Color(0xFF10B981).withOpacity(0.2)
                : const Color(0xFF8B5CF6).withOpacity(0.2),
            borderRadius: BorderRadius.circular(6),
          ),
          child: Text(
            device.isSender ? 'SENDER' : 'RECEIVER',
            style: TextStyle(
              fontSize: 10,
              fontWeight: FontWeight.bold,
              color: device.isSender
                  ? const Color(0xFF10B981)
                  : const Color(0xFF8B5CF6),
              letterSpacing: 0.5,
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildConnectionIndicator() {
    if (isConnected) {
      return Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: const Color(0xFF10B981).withOpacity(0.2),
          shape: BoxShape.circle,
        ),
        child: const Icon(
          Icons.check,
          color: Color(0xFF10B981),
          size: 20,
        ),
      );
    }

    return Icon(
      Icons.chevron_right,
      color: Colors.white.withOpacity(0.3),
      size: 24,
    );
  }

  Color _getLatencyColor(int latency) {
    if (latency < 20) {
      return const Color(0xFF10B981);
    } else if (latency < 50) {
      return const Color(0xFFF59E0B);
    } else {
      return const Color(0xFFEF4444);
    }
  }
}
