import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/widgets/small.dart';

import 'model.dart';

class ScreenEventWidget extends StatefulWidget {
  final Job target;
  final String? forcedString;
  const ScreenEventWidget({super.key, required this.target, this.forcedString});

  @override
  State<ScreenEventWidget> createState() => _ScreenEventWidgetState();
}

class _ScreenEventWidgetState extends State<ScreenEventWidget> {
  @override
  Widget build(BuildContext context) {
    LoadScreenModel screenModel = Provider.of<LoadScreenModel>(context);
    if (widget.forcedString != null) {
      return SmallText(text: widget.forcedString!);
    }
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
  if (screenModel.error(targetJob) != null) {
    return errorString(screenModel.error(targetJob)!);
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
