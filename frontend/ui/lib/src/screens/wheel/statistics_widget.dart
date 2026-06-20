import 'dart:developer' as developer;
import 'dart:math';

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:wpx/src/models/segmentmodel.dart';
import 'package:wpx/src/rust/api/bridge.dart' as bridge;
import 'package:wpx/src/rust/api/bridge.dart';
import 'package:wpx/src/screens/wheel/speed_dialog.dart';
import 'package:wpx/src/utils/print.dart';
import 'package:wpx/src/widgets/small.dart';
import 'package:wpx/src/utils/utils.dart';

class OverviewWidget extends StatefulWidget {
  final void Function()? onPacingPointPressed;
  final void Function()? onControlsPointPressed;
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

class _OverviewWidgetState extends State<OverviewWidget> {
  DateTime? startTime;
  DateTime? endTime;
  String? speed;
  String? lastConstantSpeed;
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
      debugPrint("parameter speed: ${parameters.speed}");
      // Initialize lastConstantSpeed if current speed is constant
      SpeedMode speedMode = parseSpeedMode(parameters.speed);
      if (speedMode == SpeedMode.kmh) {
        lastConstantSpeed = parameters.speed;
      }
    });
  }

  void persistParameters(Bridge backend) async {
    if (!mounted) return;
    // There seem to be a bug in flutter_rust_bridge async handling:
    // the `await backend.persistSmallParameters();` freezes the application.
    // This one-second delay is a workaround. I hope it efficiently prevents the freeze.
    await Future.delayed(Duration(milliseconds: 1));
    await backend.persistSmallParameters();
  }

  void writeModel() {
    if (!mounted) return;
    ParameterModel parametersModel = Provider.of<ParameterModel>(
      context,
      listen: false,
    );
    bridge.Parameters oldParameters = parametersModel.parameters();
    ParameterChanger changer = ParameterChanger(init: oldParameters);
    SpeedMode speedMode = parseSpeedMode(speed!);
    if (speedMode == SpeedMode.kmh) {
      lastConstantSpeed = speed;
    }
    changer.changeSpeed(speed!);
    changer.changeStartTime(startTime!);
    bridge.Parameters parameters = changer.current();
    parametersModel.setParameters(parameters);
    debugPrint("write speed:${parameters.speed}");
    setState(() {
      startTime = parseDateTime(parameters.startTime);
      speed = parameters.speed;
      WidgetsBinding.instance.addPostFrameCallback((_) {
        persistParameters(getBackend(context));
      });
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
        startTime,
        init,
        null,
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
      double mps = distance / seconds;
      // max at 50kmh
      mps = min(mps, 50000 / 3600);
      speed = speedSpecFromMPS(mps);
      writeModel();
    }
  }

  Widget buildWorker(BuildContext ctx) {
    SegmentModel segmentModel = Provider.of<SegmentModel>(ctx);
    ParameterModel parameterModel = Provider.of<ParameterModel>(context);
    Parameters parameters = parameterModel.parameters();
    bridge.SegmentStatistics statistics = segmentModel.statistics();
    double km = statistics.distanceEnd / 1000;
    double hm = statistics.elevationGain;
    speed = parameterModel.parameters().speed;

    if (startTime == null) {
      return Text("loading..");
    }

    String startDateText = DateFormat('dd/MM').format(startTime!);
    String startTimeText = formatTime(startTime!);
    DateTime endTime = parseDateTime(statistics.endTime);
    String endTimeText = formatTime(endTime);

    String pacingPointsText = getPacingPointText(parameters);

    List<Waypoint> controlPoints = segmentModel.someWaypoints([Kind.controls]);
    String controlPointsText = "${controlPoints.length}";

    bridge.Bridge backend = getBackend(ctx);
    List<Segment> segments = backend.segments();
    String pagesCountText = PageCountInfo.getPagesCountString(segments.length);

    String speedText = prettySpeed(parameters.speed);
    Widget endTimeWidget = SmallText(text: endTimeText);
    if (parseSpeedMode(parameters.speed) == SpeedMode.kmh) {
      endTimeWidget = SmallButton(
        text: endTimeText,
        callback: () => _selectEndTime(context, endTime),
      );
    }
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
                  callback: () => _selectStartDate(ctx),
                ),
                SmallButton(
                  text: startTimeText,
                  callback: () => _selectStartTime(ctx),
                ),
              ],
            ),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "Average speed"),
            SmallButton(
              callback:
                  () => openSpeedDialog(
                    outerContext: ctx,
                    speed: speed!,
                    allowedSpeeds: backend.allowedSpeeds(),
                    initialConstantSpeed: lastConstantSpeed,
                    onConfirm: (newSpeed) {
                      debugPrint("confirm:$newSpeed");
                      setState(() {
                        speed = newSpeed;
                      });
                      writeModel();
                    },
                    onSpeedChanged: (newSpeed) {
                      debugPrint("changed:$newSpeed");
                    },
                    onCancel: (newSpeed) {
                      debugPrint("cancel:$newSpeed");
                    },
                  ),
              text: speedText,
            ),
          ],
        ),
        TableRow(children: [SmallText(text: "End time"), endTimeWidget]),
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
            if (widget.onControlsPointPressed != null)
              SmallButton(
                text: controlPointsText,
                callback: widget.onControlsPointPressed,
              )
            else
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

  @override
  Widget build(BuildContext ctx) {
    try {
      return buildWorker(ctx);
    } catch (e, stack) {
      debugPrint("FOUND THE ERROR: $e");
      debugPrint("FOUND THE STACK: $stack");
      rethrow;
    }
  }
}
