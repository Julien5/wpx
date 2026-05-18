import 'package:flutter/material.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/widgets/datetime_range_picker.dart';

Future<void> openControlTimeDialog({
  required BuildContext context,
  required Parameters parameters,
  required Waypoint? previousControl,
  required Waypoint? nextControl,
  required Waypoint currentControl,
  required Function(DateTime) onTimeChanged,
}) async {
  final DateTime? picked = await showDateTimeRangePickerDialog(
    context: context,
    parameters: parameters,
    previous: previousControl,
    next: nextControl,
    current: currentControl,
  );

  if (picked != null) {
    onTimeChanged(picked);
  }
}
