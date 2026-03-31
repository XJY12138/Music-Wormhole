import 'package:flutter/material.dart';
import '../models/device_info.dart';

class SyncService extends ChangeNotifier {
  DeviceInfo? _connectedDevice;
  bool _isConnected = false;
  bool _isPlaying = false;
  int _currentLatency = 0;
  int _bitrate = 320;
  int _volume = 80;
  int _syncOffset = 0;
  bool _isAllApps = true;

  DeviceInfo? get connectedDevice => _connectedDevice;
  bool get isConnected => _isConnected;
  bool get isPlaying => _isPlaying;
  int get currentLatency => _currentLatency;
  int get bitrate => _bitrate;
  int get volume => _volume;
  int get syncOffset => _syncOffset;
  bool get isAllApps => _isAllApps;
  String? get connectedDeviceId => _connectedDevice?.id;

  Future<void> connectToDevice(DeviceInfo device) async {
    try {
      // TODO: Implement actual connection logic
      await Future.delayed(const Duration(milliseconds: 500));
      
      _connectedDevice = device;
      _isConnected = true;
      _currentLatency = device.latency;
      notifyListeners();
    } catch (e) {
      rethrow;
    }
  }

  void disconnect() {
    _connectedDevice = null;
    _isConnected = false;
    _isPlaying = false;
    notifyListeners();
  }

  void togglePlayback() {
    _isPlaying = !_isPlaying;
    notifyListeners();
  }

  void playPrevious() {
    // TODO: Implement previous track
  }

  void playNext() {
    // TODO: Implement next track
  }

  void setVolume(int volume) {
    _volume = volume.clamp(0, 100);
    notifyListeners();
  }

  void setSyncOffset(int offset) {
    _syncOffset = offset;
    notifyListeners();
  }

  void setAudioSource(bool allApps) {
    _isAllApps = allApps;
    notifyListeners();
  }

  void updateLatency(int latency) {
    _currentLatency = latency;
    notifyListeners();
  }
}
