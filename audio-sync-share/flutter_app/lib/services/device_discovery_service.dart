import 'package:flutter/material.dart';
import '../models/device_info.dart';

class DeviceDiscoveryService extends ChangeNotifier {
  final List<DeviceInfo> _devices = [];
  bool _isScanning = false;
  String? _error;

  List<DeviceInfo> get devices => _devices;
  bool get isScanning => _isScanning;
  String? get error => _error;

  Future<void> startDiscovery() async {
    _isScanning = true;
    _error = null;
    notifyListeners();

    try {
      // TODO: Implement mDNS discovery
      await Future.delayed(const Duration(seconds: 2));
      
      // Mock devices for demonstration
      _devices.clear();
      _devices.addAll([
        DeviceInfo(
          id: 'device_1',
          name: 'Living Room Speaker',
          ipAddress: '192.168.1.105',
          platform: 'Android',
          isSender: false,
          latency: 15,
          lastSeen: DateTime.now(),
        ),
        DeviceInfo(
          id: 'device_2',
          name: 'Bedroom PC',
          ipAddress: '192.168.1.110',
          platform: 'Windows',
          isSender: true,
          latency: 25,
          lastSeen: DateTime.now(),
        ),
      ]);
    } catch (e) {
      _error = 'Failed to discover devices: $e';
    } finally {
      _isScanning = false;
      notifyListeners();
    }
  }

  void stopDiscovery() {
    _isScanning = false;
    notifyListeners();
  }

  void addDevice(DeviceInfo device) {
    final existingIndex = _devices.indexWhere((d) => d.id == device.id);
    if (existingIndex >= 0) {
      _devices[existingIndex] = device;
    } else {
      _devices.add(device);
    }
    notifyListeners();
  }

  void removeDevice(String deviceId) {
    _devices.removeWhere((d) => d.id == deviceId);
    notifyListeners();
  }

  void clearDevices() {
    _devices.clear();
    notifyListeners();
  }
}
