import 'package:flutter/material.dart';
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/utils/utils.dart';
import 'package:wpx/src/widgets/datetime_range_picker.dart';

Future<void> openControlTimeDialog({
  required BuildContext context,
  required Waypoint? previousControl,
  required Waypoint? nextControl,
  required String currentTimeIso,
  required Function(DateTime) onTimeChanged,
}) async {
  DateTime currentDateTime = parseDateTime(currentTimeIso);
  DateTime minDateTime = parseDateTime(previousControl!.info!.time);
  DateTime maxDateTime = parseDateTime(nextControl!.info!.time);

  final DateTime? picked = await showDateTimeRangePickerDialog(
    context: context,
    min: minDateTime,
    minLabel: previousControl.name,
    max: maxDateTime,
    maxLabel: nextControl.name,
    initial: currentDateTime,
  );

  if (picked != null) {
    onTimeChanged(picked);
  }
}
