import 'package:flutter/material.dart';
import 'package:wpx/src/utils/utils.dart';

Future<void> openControlTimeDialog({
  required BuildContext context,
  required String currentTimeIso,
  required Function(DateTime) onTimeChanged,
}) async {
  DateTime currentDateTime = parseDateTime(currentTimeIso);

  final TimeOfDay? picked = await showTimePicker(
    context: context,
    initialTime: TimeOfDay(
      hour: currentDateTime.hour,
      minute: currentDateTime.minute,
    ),
    builder: (context, child) {
      return MediaQuery(
        data: MediaQuery.of(context).copyWith(alwaysUse24HourFormat: true),
        child: child!,
      );
    },
  );

  if (picked != null) {
    DateTime newDateTime = bestEndTime(
      null,
      currentDateTime,
      null,
      picked.hour,
      picked.minute,
    );
    onTimeChanged(newDateTime);
  }
}
