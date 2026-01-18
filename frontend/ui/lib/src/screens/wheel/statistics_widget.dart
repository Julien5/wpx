import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/rust/api/bridge.dart';
import 'package:ui/src/widgets/slidervalues.dart';
import 'package:ui/utils.dart';

class SmallButton extends StatelessWidget {
  final VoidCallback? callback;
  final String text;
  const SmallButton({super.key, this.callback, required this.text});

  @override
  Widget build(BuildContext context) {
    EdgeInsets valuePadding = const EdgeInsets.fromLTRB(15, 0, 15, 0);
    return Padding(
      padding: const EdgeInsets.all(8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.start,
        children: [
          OutlinedButton(
            onPressed: callback,
            style: ElevatedButton.styleFrom(
              padding: valuePadding,
              minimumSize: Size.zero,
              tapTargetSize: MaterialTapTargetSize.shrinkWrap,
            ),
            child: Text(text, textAlign: TextAlign.center),
          ),
        ],
      ),
    );
  }
}

class SmallText extends StatelessWidget {
  final String text;
  const SmallText({super.key, required this.text});

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(8.0),
      child: Text(text, textAlign: TextAlign.left),
    );
  }
}

class StatisticsWidget extends StatefulWidget {
  final void Function() onPacingPointPressed;
  final void Function() onControlsPointPressed;
  final void Function() onPagesPressed;
  const StatisticsWidget({
    super.key,
    required this.onPacingPointPressed,
    required this.onControlsPointPressed,
    required this.onPagesPressed,
  });

  @override
  State<StatisticsWidget> createState() => _StatisticsWidgetState();
}

List<double> speedSliderValues() {
  return fromKmh([5, 10, 12.5, 13.5, 15, 18.0, 20, 25, 28]);
}

class _StatisticsWidgetState extends State<StatisticsWidget> {
  DateTime? startTime;
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
    SegmentModel segmentModel = Provider.of<SegmentModel>(
      context,
      listen: false,
    );
    bridge.Parameters parameters = segmentModel.parameters();
    setState(() {
      startTime = DateTime.parse(parameters.startTime);
      speed = parameters.speed;
    });
  }

  void writeModel() {
    if (!mounted) return;
    SegmentModel segmentModel = Provider.of<SegmentModel>(
      context,
      listen: false,
    );
    bridge.Parameters oldParameters = segmentModel.parameters();
    ParameterChanger changer = ParameterChanger(init: oldParameters);
    changer.changeSpeed(speed!);
    changer.changeStartTime(startTime!);
    bridge.Parameters parameters = changer.current();
    segmentModel.setParameters(parameters);
    setState(() {
      startTime = DateTime.parse(parameters.startTime);
      speed = parameters.speed;
    });
  }

  Future<void> _selectTime(BuildContext context) async {
    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay(hour: startTime!.hour, minute: startTime!.minute),
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

  void onSpeedChanged(double newSpeed) {
    developer.log("new speed: $newSpeed");
    speed = newSpeed;
    setState(() {});
    writeModel();
  }

  void openSpeedDialog() {
    List<double> values = speedSliderValues();
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

  @override
  Widget build(BuildContext ctx) {
    SegmentModel segmentModel = Provider.of<SegmentModel>(ctx);
    bridge.Parameters parameters = segmentModel.parameters();
    bridge.SegmentStatistics statistics = segmentModel.statistics();
    double km = statistics.distanceEnd / 1000;
    double hm = statistics.elevationGain;
    double kmh = segmentModel.parameters().speed * 3600 / 1000;
    String startTimeText = "?";
    String endTimeText = "?";
    if (startTime != null) {
      startTimeText = DateFormat('HH:mm').format(startTime!);
      Duration duration = Duration(
        seconds: (statistics.distanceEnd / parameters.speed).round(),
      );
      DateTime endTime = startTime!.add(duration);
      endTimeText = DateFormat('HH:mm').format(endTime);
    }

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

    List<Waypoint> controlPoints = segmentModel.someWaypoints({
      InputType.control,
    });
    String controlPointsText = "${controlPoints.length}";

    RootModel root = Provider.of<RootModel>(context);
    List<Segment> segments = root.segments();
    String pagesCountText =
        "${(segments.length / 2).ceil().toString().padLeft(2)} pages";
    Widget table2 = Table(
      columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
      children: [
        TableRow(
          children: [
            SmallText(text: "Start time"),
            SmallButton(
              text: startTimeText,
              callback: () => _selectTime(context),
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
          children: [SmallText(text: "End time"), SmallText(text: endTimeText)],
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
        TableRow(
          children: [
            SmallText(text: "Pacing points"),
            SmallButton(
              callback: widget.onPacingPointPressed,
              text: pacingPointsText,
            ),
          ],
        ),
        TableRow(
          children: [
            SmallText(text: "PDF"),
            SmallButton(callback: widget.onPagesPressed, text: pagesCountText),
          ],
        ),
      ],
    );
    return Column(
      mainAxisAlignment: MainAxisAlignment.center,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [Card(elevation: 4, child: table2)],
    );
  }
}
