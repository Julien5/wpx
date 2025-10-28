 import 'dart:developer' as developer;

void log(String message) {
  final now = DateTime.now();
  final formatted = "[${now.hour.toString().padLeft(2, '0')}:"
      "${now.minute.toString().padLeft(2, '0')}:"
      "${now.second.toString().padLeft(2, '0')}."
      "${now.millisecond.toString().padLeft(3, '0')}] $message";
  developer.log(formatted);
}