import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/widgets/small.dart';

import 'model.dart';

class EventWidget extends StatefulWidget {
  final Job target;
  final String? forcedString;
  const EventWidget({super.key, required this.target, this.forcedString});

  @override
  State<EventWidget> createState() => _EventWidgetState();
}

class _EventWidgetState extends State<EventWidget> {
  @override
  Widget build(BuildContext context) {
    if (widget.forcedString != null) {
      return SmallText(text: widget.forcedString!);
    }
    LoadScreenModel screenModel = Provider.of<LoadScreenModel>(context);
    return SmallText(
      text: filterEvent(screenModel.lastEvent(), widget.target, screenModel),
    );
  }
}

String safeLast(String? event) {
  if (event == null) {
    return "...";
  }
  return event;
}

String errorString(Object o) {
  if (o is! bridge.TrackError) {
    return o.toString();
  }
  bridge.TrackError e = o;
  if (e is bridge.TrackError_MissingElevation) {
    //var index = e.index;
    return "The track misses elevation data.";
  }
  if (e is bridge.TrackError_GPXHasNoSegment) {
    return "no segment in gpx";
  }
  if (e is bridge.TrackError_GPXInvalid) {
    return "invalid gpx file";
  }
  if (e is bridge.TrackError_OSMDownloadFailed) {
    return "download failed";
  }
  if (e is bridge.TrackError_OSMDownloadTimeout) {
    return "download timed out";
  }
  if (e is bridge.TrackError_Unknown) {
    return "unknown error";
  }
  debugPrint(e.toString());
  return e.toString();
}

String filterEvent(String? event, Job targetJob, LoadScreenModel screenModel) {
  if (screenModel.hasFailed(targetJob) != null) {
    return errorString(screenModel.hasFailed(targetJob)!);
  }
  if (screenModel.runningFuture != null &&
      screenModel.runningFuture!.job == targetJob) {
    //return "event: [${safeLast(eventModel)}]";
    return safeLast(event);
  }
  if (screenModel.hasDone(targetJob)) {
    return "done";
  }
  return "..";
}
