import 'package:flutter/material.dart';
import 'package:wpx/src/utils/utils.dart';

DateTime bestDateTimeForTime(
  DateTime currentDateTime,
  int targetHour,
  int targetMinute,
) {
  DateTime? best;
  double bestDiff = double.infinity;

  for (int dayOffset = -10; dayOffset < 10; dayOffset++) {
    final candidate = DateTime(
      currentDateTime.year,
      currentDateTime.month,
      currentDateTime.day + dayOffset,
      targetHour,
      targetMinute,
    );

    final diffMicrosec =
        (currentDateTime.microsecondsSinceEpoch -
                candidate.microsecondsSinceEpoch)
            .abs();

    if (diffMicrosec < bestDiff) {
      bestDiff = diffMicrosec.toDouble();
      best = candidate;
    }
  }

  if (best != null) {
    return best;
  }

  return DateTime(
    currentDateTime.year,
    currentDateTime.month,
    currentDateTime.day,
    targetHour,
    targetMinute,
  );
}

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
    DateTime newDateTime = bestDateTimeForTime(
      currentDateTime,
      picked.hour,
      picked.minute,
    );
    onTimeChanged(newDateTime);
  }
}
