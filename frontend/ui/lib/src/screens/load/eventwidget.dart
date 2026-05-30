import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/events.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/widgets/small.dart';

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
  if (e is bridge.TrackError_OSMDownloadRunning) {
    return "download running";
  }
  if (e is bridge.TrackError_OSMDownloadCancelled) {
    return "download cancelled";
  }
  if (e is bridge.TrackError_Unknown) {
    return "unknown error";
  }
  debugPrint(e.toString());
  return e.toString();
}

String filterEvent(String? event, Job targetJob, LoadScreenModel model) {
  if (model.hasFailed(targetJob) != null) {
    return errorString(model.hasFailed(targetJob)!);
  }
  if (model.runningFuture != null && model.runningFuture!.job == targetJob) {
    return safeLast(event);
  }
  if (model.hasDone(targetJob)) {
    return "done";
  }
  return "..";
}

class OsmEventWidget extends StatefulWidget {
  final Job target;
  final String? forcedString;
  const OsmEventWidget({super.key, required this.target, this.forcedString});

  @override
  State<OsmEventWidget> createState() => _OsmEventWidgetState();
}

class _OsmEventWidgetState extends State<OsmEventWidget> {
  @override
  Widget build(BuildContext context) {
    EventModel event = Provider.of<EventModel>(context);
    OsmStatus progressInfo = event.getOsmEvent();

    if (progressInfo.niceTaskName().isNotEmpty && !progressInfo.done()) {
      Widget? bar;
      if (progressInfo.total > 0) {
        bar = Padding(
          padding: EdgeInsetsGeometry.fromLTRB(15, 0, 15, 0),
          child: LinearProgressIndicator(
            value:
                progressInfo.current /
                progressInfo
                    .total, // Value between 0.0 and 1.0 indicating progress
            semanticsLabel: progressInfo.niceTaskName(),
          ),
        );
      }
      Widget label = Text(
        progressInfo.niceTaskName(),
        textAlign: TextAlign.center,
        style: TextStyle(fontSize: 11, fontWeight: FontWeight.w500),
      );
      return Column(
        children: [
          SizedBox(height: 10),
          label,
          SizedBox(height: 10),
          if (bar != null) bar,
        ],
      );
    }
    return SmallText(text: progressInfo.niceTaskName());
  }
}
