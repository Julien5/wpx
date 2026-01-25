import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/widgets/small.dart';

import 'model.dart';

class EventWidget extends StatefulWidget {
  final LoadScreenModel screenModel;
  final Job target;
  final String? forcedString;
  const EventWidget({
    super.key,
    required this.screenModel,
    required this.target,
    this.forcedString,
  });

  @override
  State<EventWidget> createState() => _EventWidgetState();
}

class _EventWidgetState extends State<EventWidget> {
  EventModel? model;

  @override
  Widget build(BuildContext context) {
    if (widget.forcedString != null) {
      return SmallText(text: widget.forcedString!);
    }
    EventModel model = Provider.of<EventModel>(context, listen: false);
    return StreamBuilder<String>(
      stream: model.broadcastStream,
      builder: (context, snap) {
        final error = snap.error;
        String event = "....";
        if (error != null) {
          event = error.toString();
          developer.log("error: ${error.toString()}");
        }
        final data = snap.data;
        if (data != null) {
          event = data;
        }
        return SmallText(
          text: filterEvent(event, widget.target, widget.screenModel),
        );
      },
    );
  }
}

String safeLast(String? event) {
  if (event == null) {
    return "...";
  }
  //developer.log("====> ${event.events.last} ($n)");
  return event;
}

String errorString(bridge.Error e) {
  if (e is bridge.Error_MissingElevation) {
    //var index = e.index;
    return "The track misses elevation data.";
  }
  if (e is bridge.Error_GPXHasNoSegment) {
    return "no segment in gpx";
  }
  if (e is bridge.Error_GPXInvalid) {
    return "invalid gpx file";
  }
  if (e is bridge.Error_OSMDownloadFailed) {
    return "download failed";
  }
  return "";
}

String filterEvent(String? event, Job targetJob, LoadScreenModel screenModel) {
  if (screenModel.errors.containsKey(targetJob)) {
    //return "error: [${errorString(screenModel.errors[targetJob]!)}]";
    return errorString(screenModel.errors[targetJob]!);
  }
  if (screenModel.running != null && screenModel.running! == targetJob) {
    //return "event: [${safeLast(eventModel)}]";
    return safeLast(event);
  }
  if (screenModel.hasDone(targetJob)) {
    return "done";
  }
  return "..";
}
