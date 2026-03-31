import 'package:flutter/material.dart';

class DeviceInfo {
  final String id;
  final String name;
  final String ipAddress;
  final String platform;
  final bool isSender;
  final int latency;
  final DateTime lastSeen;

  DeviceInfo({
    required this.id,
    required this.name,
    required this.ipAddress,
    required this.platform,
    required this.isSender,
    this.latency = 0,
    required this.lastSeen,
  });

  IconData get platformIcon {
    switch (platform.toLowerCase()) {
      case 'android':
        return Icons.android;
      case 'ios':
        return Icons.phone_iphone;
      case 'windows':
        return Icons.laptop_windows;
      case 'macos':
        return Icons.laptop_mac;
      case 'linux':
        return Icons.laptop;
      default:
        return Icons.devices;
    }
  }

  Color get platformColor {
    switch (platform.toLowerCase()) {
      case 'android':
        return const Color(0xFF3DDC84);
      case 'ios':
        return Colors.grey;
      case 'windows':
        return const Color(0xFF00A4EF);
      case 'macos':
        return const Color(0xFFA2AAAD);
      case 'linux':
        return const Color(0xFFFCC624);
      default:
        return Colors.grey;
    }
  }
}
