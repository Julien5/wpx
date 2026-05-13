import 'dart:developer' as developer;
import 'dart:math';

import 'package:flutter/material.dart';
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

  bridge.Parameters changeStartTime(DateTime time) {
    String rfc3339time = time.toUtc().toIso8601String();
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
}

DateTime parseDateTime(String data) {
  return DateTime.parse(data).toLocal();
}

String joinNonEmpty(List<String> parts) {
  return parts.where((s) => s.isNotEmpty).join(', ');
}

bridge.Bridge getBackend(BuildContext context) {
  RootModel root = Provider.of<RootModel>(context, listen: false);
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

// convert from kmh to mps
double parseSpeedMps(String speed) {
  return double.parse(speed) * 1000 / 3600;
}

String speedString(double mps) {
  double kmh = mps * 3600 / 1000;
  return kmh.toStringAsFixed(1);
}
