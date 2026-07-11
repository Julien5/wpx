import 'dart:developer' as developer;
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/root.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;

double scaleDown(Size object, Size drawArea) {
  double sw = drawArea.width / object.width;
  double sh = drawArea.height / object.height;
  return [sw, sh, 1.0].reduce(min);
}

List<double> fromKm(List<double> list) {
  List<double> ret = list;
  for (int k = 0; k < list.length; ++k) {
    ret[k] = list[k] * 1000;
  }
  return ret;
}

enum ScreenOrientation { desktop, landscape, portrait }

ScreenOrientation screenOrientation(Size size) {
  if (size.width > 1000 && size.height > 500) {
    return ScreenOrientation.desktop;
  }
  if (size.width > size.height) {
    return ScreenOrientation.landscape;
  }
  return ScreenOrientation.portrait;
}

(int, int) sizeAsTuple(Size s) {
  assert(s.width.isFinite);
  assert(s.height.isFinite);
  return (s.width.floor(), s.height.floor());
}

Size makeFinite(Size size) {
  // this size is passed to the backend for rendering
  int max = 1280 * 1280 * 1280;
  int w = max;
  int h = max;
  if (size.width.isFinite) {
    w = size.width.floor();
  }
  if (size.height.isFinite) {
    h = size.height.floor();
  }
  return Size(w.toDouble(), h.toDouble());
}

String serializeTime(DateTime dateTime) {
  return dateTime.toUtc().toIso8601String();
}

class ParameterChanger {
  bridge.Parameters init;
  ParameterChanger({required this.init});
  bridge.Parameters current() {
    return init;
  }

  bridge.Parameters changeSpeed(String speed) {
    bridge.Parameters ret = bridge.Parameters(
      speed: speed,
      startTime: init.startTime,
      segmentLength: init.segmentLength,
      segmentOverlap: init.segmentOverlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }

  bridge.Parameters changeTimeAxis(bridge.TimeAxis timeAxis) {
    bridge.Parameters ret = bridge.Parameters(
      speed: init.speed,
      startTime: init.startTime,
      segmentLength: init.segmentLength,
      segmentOverlap: init.segmentOverlap,
      smoothWidth: init.smoothWidth,
      profileOptions: bridge.ProfileOptions(timeAxis: timeAxis),
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }

  bridge.Parameters changeStartTime(DateTime time) {
    String rfc3339time = serializeTime(time);
    developer.log("time = $rfc3339time");
    bridge.Parameters ret = bridge.Parameters(
      speed: init.speed,
      startTime: rfc3339time,
      segmentLength: init.segmentLength,
      segmentOverlap: init.segmentOverlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }

  bridge.Parameters changeSegmentLength(double length) {
    bridge.Parameters ret = bridge.Parameters(
      speed: init.speed,
      startTime: init.startTime,
      segmentLength: length,
      segmentOverlap: init.segmentOverlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }

  bridge.Parameters changeSegmentOverlap(double overlap) {
    bridge.Parameters ret = bridge.Parameters(
      speed: init.speed,
      startTime: init.startTime,
      segmentLength: init.segmentLength,
      segmentOverlap: overlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: init.userStepsOptions,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }

  bridge.Parameters changeUserStepsOptions(bridge.UserStepsOptions options) {
    bridge.Parameters ret = bridge.Parameters(
      speed: init.speed,
      startTime: init.startTime,
      segmentLength: init.segmentLength,
      segmentOverlap: init.segmentOverlap,
      smoothWidth: init.smoothWidth,
      profileOptions: init.profileOptions,
      mapOptions: init.mapOptions,
      userStepsOptions: options,
      debug: init.debug,
      controlGpxNameFormat: init.controlGpxNameFormat,
    );
    init = ret;
    return ret;
  }
}

DateTime parseDateTime(String data) {
  return DateTime.parse(data).toLocal();
}

String joinNonEmpty(List<String> parts) {
  return parts.where((s) => s.isNotEmpty).join(', ');
}

bridge.Bridge getBackend(BuildContext context) {
  RootModel root = context.read<RootModel>();
  return root.getBackend();
}

String getPacingPointText(bridge.Parameters parameters) {
  String pacingPointsText = "none";
  if (parameters.userStepsOptions.stepElevationGain != null) {
    double hm = parameters.userStepsOptions.stepElevationGain!;
    pacingPointsText = "every ${hm.toStringAsFixed(0)} m elevation gain";
  } else if (parameters.userStepsOptions.stepDistance != null) {
    double km = parameters.userStepsOptions.stepDistance! / 1000;
    pacingPointsText = "every ${km.toStringAsFixed(0)} km";
  } else {
    pacingPointsText = "none";
  }
  return pacingPointsText;
}

String statisticsString(bridge.SegmentStatistics statistics) {
  double k1 = statistics.distanceStart / 1000;
  double k2 = statistics.distanceEnd / 1000;
  return "${k1.toStringAsFixed(1)} - ${k2.toStringAsFixed(1)}";
}

String speedSpecFromMPS(double mps) {
  double kmh = mps * 3600 / 1000;
  return "KMH-$kmh";
}

String formatKmh(double kmh, int n) {
  String result = kmh.toStringAsFixed(n);
  if (!result.contains('.')) {
    result = '$result.0';
  }
  while (result.endsWith('0') && !result.endsWith('.0')) {
    result = result.substring(0, result.length - 1);
  }

  return result;
}

DateTime _roundToMinute(DateTime dt) {
  if (dt.second >= 30 || dt.millisecond >= 500) {
    return dt
        .copyWith(second: 0, millisecond: 0, microsecond: 0)
        .add(const Duration(minutes: 1));
  }
  return dt.copyWith(second: 0, millisecond: 0, microsecond: 0);
}

String formatTime(DateTime t) {
  // rounding to the minute (as opposed to truncating the seconds part)
  // allows to display 12:34:59.999 as 12:35:00.
  return DateFormat('HH:mm').format(_roundToMinute(t));
}

String formatDate(DateTime t) {
  return DateFormat('dd/MM').format(t);
}

String formatDuration(Duration duration) {
  // https://stackoverflow.com/questions/54775097/formatting-a-duration-as-hhmmss
  String negativeSign = duration.isNegative ? '-' : '';
  String twoDigits(int n) => n.toString().padLeft(2, "0");
  String twoDigitMinutes = twoDigits(duration.inMinutes.remainder(60).abs());
  if (duration.inHours == 0) {
    return "$negativeSign$twoDigitMinutes min";
  }
  return "$negativeSign${twoDigits(duration.inHours)} h $twoDigitMinutes min";
}

DateTime bestEndTime(
  DateTime? min,
  DateTime init,
  DateTime? max,
  int endHour,
  int endMinute,
) {
  DateTime? best;
  double bestDiff = double.infinity;

  for (int dayOffset = -10; dayOffset < 10; dayOffset++) {
    final candidate = DateTime(
      init.year,
      init.month,
      init.day + dayOffset,
      endHour,
      endMinute,
    );

    if (min != null &&
        candidate.microsecondsSinceEpoch < min.microsecondsSinceEpoch) {
      continue;
    }

    if (max != null &&
        candidate.microsecondsSinceEpoch > max.microsecondsSinceEpoch) {
      continue;
    }

    final diffmicrosec =
        (init.microsecondsSinceEpoch - candidate.microsecondsSinceEpoch).abs();

    debugPrint("offset: $dayOffset diff:${diffmicrosec / 1000 / 3600}");
    if (diffmicrosec < bestDiff) {
      bestDiff = diffmicrosec.toDouble();
      best = candidate;
    }
  }
  if (best != null) {
    return best;
  }
  return DateTime(init.year, init.month, init.day, endHour, endMinute);
}
