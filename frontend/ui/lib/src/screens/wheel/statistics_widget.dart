import 'dart:developer' as developer;
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/utils/print.dart';
import 'package:wpx/src/widgets/slidervalues.dart';
import 'package:wpx/src/widgets/small.dart';
import 'package:wpx/src/utils/utils.dart';

class OverviewWidget extends StatefulWidget {
  final void Function()? onPacingPointPressed;
  final void Function() onControlsPointPressed;
  final void Function()? onPDFPressed;
  const OverviewWidget({
    super.key,
    required this.onPacingPointPressed,
    required this.onControlsPointPressed,
    required this.onPDFPressed,
  });

  @override
  State<OverviewWidget> createState() => _OverviewWidgetState();
}

List<double> speedSliderValues() {
  return fromKmh([5, 10, 12.5, 13.5, 15, 18.0, 20, 25, 28]);
}

class _OverviewWidgetState extends State<OverviewWidget> {
  DateTime? startTime;
  DateTime? endTime;
  double? speed;
  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      readModel();
    });
  }

  void readModel() {
    developer.log("read model");
    ParameterModel parametersModel = Provider.of<ParameterModel>(
      context,
      listen: false,
    );
    bridge.Parameters parameters = parametersModel.parameters();
    setState(() {
      startTime = parseDateTime(parameters.startTime);
      speed = parameters.speed;
    });
  }

  void writeModel() {
    if (!mounted) return;
    ParameterModel parametersModel = Provider.of<ParameterModel>(
      context,
      listen: false,
    );
    bridge.Parameters oldParameters = parametersModel.parameters();
    ParameterChanger changer = ParameterChanger(init: oldParameters);
    changer.changeSpeed(speed!);
    changer.changeStartTime(startTime!);
    bridge.Parameters parameters = changer.current();
    parametersModel.setParameters(parameters);
    setState(() {
      startTime = parseDateTime(parameters.startTime);
      speed = parameters.speed;
    });
  }

  Future<void> _selectStartDate(BuildContext context) async {
    final picked = await showDatePicker(
      context: context,
      initialDate: DateTime.now(),
      firstDate: DateTime(2000),
      lastDate: DateTime(2100),
    );

    // Guard against using the BuildContext after an async gap
    if (picked != null) {
      startTime = DateTime(
        picked.year,
        picked.month,
        picked.day,
        startTime!.hour,
        startTime!.minute,
      );
      writeModel();
    }
  }

  Future<void> _selectStartTime(BuildContext context) async {
    // see here
    // https://stackoverflow.com/questions/66023387/flutter-how-to-use-timepickerthemedata-properly
    // to change the colors of the time picker.
    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay(hour: startTime!.hour, minute: startTime!.minute),
      builder: (context, child) {
        return MediaQuery(
          data: MediaQuery.of(context).copyWith(alwaysUse24HourFormat: true),
          child: child!,
        );
      },
    );

    // Guard against using the BuildContext after an async gap
    if (picked != null) {
      startTime = DateTime(
        startTime!.year,
        startTime!.month,
        startTime!.day,
        picked.hour,
        picked.minute,
      );
      writeModel();
    }
  }

  DateTime bestEndTime(
    DateTime start,
    double distance,
    double initSpeed,
    int hour,
    int minute,
  ) {
    DateTime? best;
    double bestDiff = double.infinity;

    // Search a reasonable range of days around the start date
    for (int dayOffset = 0; dayOffset < 30; dayOffset++) {
      final candidate = DateTime(
        start.year,
        start.month,
        start.day + dayOffset,
        hour,
        minute,
      );
      final seconds = candidate.difference(start).inSeconds;
      if (seconds <= 0) continue;

      final speed = distance / seconds;
      final diff = (speed - initSpeed).abs();
      debugPrint("speed candidate:$candidate => speed=$speed => diff=$diff");

      if (diff < bestDiff) {
        bestDiff = diff;
        best = candidate;
      }
    }
    debugPrint("speed best:$best");

    return best ?? DateTime(start.year, start.month, start.day, hour, minute);
  }

  Future<void> _selectEndTime(BuildContext context, DateTime init) async {
    // see here
    // https://stackoverflow.com/questions/66023387/flutter-how-to-use-timepickerthemedata-properly
    // to change the colors of the time picker.
    SegmentModel segmentModel = Provider.of(context, listen: false);

    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay(hour: init.hour, minute: init.minute),
      builder: (context, child) {
        return MediaQuery(
          data: MediaQuery.of(context).copyWith(alwaysUse24HourFormat: true),
          child: child!,
        );
      },
    );

    // Guard against using the BuildContext after an async gap
    if (picked != null) {
      SegmentStatistics stat = segmentModel.statistics();
      double distance = stat.distanceEnd - stat.distanceStart;
      DateTime endTime = bestEndTime(
        startTime!,
        distance,
        speed!,
        picked.hour,
        picked.minute,
      );

      if (!mounted) {
        return;
      }

      int seconds = endTime.difference(startTime!).inSeconds;
      if (seconds <= 0) {
        return;
      }
      speed = distance / seconds;
      // max at 50kmh
      speed = min(speed!, 50000 / 3600);
      writeModel();
    }
  }

  void onSpeedChanged(double newSpeed) {
    developer.log("new speed: $newSpeed");
    speed = newSpeed;
    setState(() {});
    writeModel();
  }

  void openSpeedDialog() {
    List<double> stdValues = speedSliderValues();
    stdValues.add(speed!);
    List<double> values = stdValues.toSet().toList()..sort();
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return StatefulBuilder(
          builder: (context, setDialogState) {
            String kmh = "none";
            int index = 0;
            if (speed != null) {
              kmh = "${(speed! * 3600 / 1000).toStringAsFixed(1)} km/h";
              index = getClosestIndex(values, speed!);
            }
            return SimpleDialog(
              title: Text('Speed', textAlign: TextAlign.center),
              children: [
                SliderValuesWidget(
                  values: values,
                  initIndex: index,
                  formatLabel:
                      (value) =>
                          "${(value * 3600 / 1000).toStringAsFixed(1)} km/h",
                  onValueChanged: (newSpeed) {
                    setDialogState(() {
                      speed = newSpeed;
                    });
                    writeModel();
                  },
                  enabled: true,
                ),
                Padding(
                  padding: const EdgeInsets.all(8.0),
                  child: Text(kmh, textAlign: TextAlign.right),
                ),
                Padding(
                  padding: const EdgeInsets.all(
                    8.0,
                  ), // Add padding to the right
                  child: ElevatedButton(
                    onPressed: () {
                      Navigator.of(context).pop();
                      // already called when the slider changed
                      // writeModel();
                    },
                    child: Text('OK', textAlign: TextAlign.right),
                  ),
                ),
              ],
            );
          },
        );
      },
    );
  }

  DateTime roundToMinute(DateTime dt) {
    if (dt.second >= 30 || dt.millisecond >= 500) {
      return dt
          .copyWith(second: 0, millisecond: 0, microsecond: 0)
          .add(const Duration(minutes: 1));
    }
    return dt.copyWith(second: 0, millisecond: 0, microsecond: 0);
  }

  @override
  Widget build(BuildContext ctx) {
    SegmentModel segmentModel = Provider.of<SegmentModel>(ctx);
    ParameterModel parameterModel = Provider.of<ParameterModel>(context);
    Parameters parameters = parameterModel.parameters();
    bridge.SegmentStatistics statistics = segmentModel.statistics();
    double km = statistics.distanceEnd / 1000;
    double hm = statistics.elevationGain;
    double kmh = parameterModel.parameters().speed * 3600 / 1000;

    if (startTime == null) {
      return Text("loading..");
    }

    String startDateText = DateFormat('dd/MM').format(startTime!);
    String startTimeText = DateFormat('HH:mm').format(startTime!);
    Duration duration = Duration(
      seconds: (statistics.distanceEnd / parameters.speed).round(),
    );
    DateTime endTime = startTime!.add(duration);
    String endTimeText = DateFormat('HH:mm').format(roundToMinute(endTime));

    String pacingPointsText = getPacingPointText(parameters);

    List<Waypoint> controlPoints = segmentModel.someWaypoints({Kind.controls});
    String controlPointsText = "${controlPoints.length}";

    bridge.Bridge backend = getBackend(ctx);
    List<Segment> segments = backend.segments();
    String pagesCountText = PageCountInfo.getPagesCountString(segments.length);
    Widget table = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(
          children: [
            SmallText(text: "Start time"),
            Row(
              children: [
                SmallButton(
                  text: startDateText,
                  callback: () => _selectStartDate(context),
                ),
                SmallButton(
                  text: startTimeText,
                  callback: () => _selectStartTime(context),
                ),
              ],
            ),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "Average speed"),
            SmallButton(
              callback: openSpeedDialog,
              text: "${kmh.toStringAsFixed(1)} kmh",
            ),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "End time"),
            SmallButton(
              text: endTimeText,
              callback: () => _selectEndTime(context, endTime),
            ),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "Distance"),
            SmallText(text: "${km.toStringAsFixed(0)} km"),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "Elevation"),
            SmallText(text: "${hm.toStringAsFixed(0)} m"),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "Control points"),
            SmallText(text: controlPointsText),
          ],
        ),
        if (widget.onPacingPointPressed != null)
          TableRow(
            children: [
              SmallText(text: "Cutoff points"),
              SmallButton(
                callback: widget.onPacingPointPressed,
                text: pacingPointsText,
              ),
            ],
          ),
        if (widget.onPDFPressed != null)
          TableRow(
            children: [
              SmallText(text: "PDF"),
              SmallButton(callback: widget.onPDFPressed, text: pagesCountText),
            ],
          ),
      ],
    );
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [Card(elevation: 4, child: table)],
    );
  }
}
