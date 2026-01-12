import 'dart:developer' as developer;

import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/models/segmentmodel.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/widgets/slidervalues.dart';

class StatisticsWidget extends StatefulWidget {
  const StatisticsWidget({super.key});

  @override
  State<StatisticsWidget> createState() => _StatisticsWidgetState();
}

List<double> fromKmh(List<double> list) {
  List<double> ret = list;
  for (int k = 0; k < list.length; ++k) {
    ret[k] = list[k] * 1000 / 3600;
  }
  return ret;
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
    showDialog(
      context: context,
      builder: (BuildContext context) {
        return StatefulBuilder(
          builder: (context, setDialogState) {
            String kmh = "none";
            int index = 0;
            if (speed != null) {
              kmh = "${(speed! * 3600 / 1000).toStringAsFixed(1)} km/h";
              index = getClosestIndex(speedSliderValues(), speed!);
            }
            return SimpleDialog(
              title: Text('Speed', textAlign: TextAlign.center),
              children: [
                SliderValuesWidget(
                  values: speedSliderValues(),
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

    EdgeInsets valuePadding = const EdgeInsets.fromLTRB(15, 0, 15, 0);
    return Container(
      constraints: const BoxConstraints(maxWidth: 300), // Set max width
      child: Table(
        columnWidths: const {0: IntrinsicColumnWidth(), 1: FlexColumnWidth()},
        children: [
          TableRow(
            children: [
              const Padding(
                padding: EdgeInsets.all(8.0),
                child: Text("Start time"),
              ),

              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    ElevatedButton(
                      onPressed: () => _selectTime(context),
                      style: ElevatedButton.styleFrom(
                        padding: valuePadding,
                        minimumSize: Size.zero,
                        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      ),
                      child: Text(startTimeText, textAlign: TextAlign.right),
                    ),
                  ],
                ),
              ),
            ],
          ),
          TableRow(
            children: [
              const Padding(
                padding: EdgeInsets.all(8.0),
                child: Text("Minimal average speed"),
              ),
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    ElevatedButton(
                      onPressed: openSpeedDialog,
                      style: ElevatedButton.styleFrom(
                        padding: valuePadding,
                        minimumSize: Size.zero,
                        tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                      ),
                      child: Text(
                        "${kmh.toStringAsFixed(1)} kmh",
                        textAlign: TextAlign.right,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
          TableRow(
            children: [
              const Padding(
                padding: EdgeInsets.all(8.0),
                child: Text("End time"),
              ),
              Padding(
                padding: const EdgeInsets.all(8.0).add(valuePadding),
                child: Text(endTimeText, textAlign: TextAlign.right),
              ),
            ],
          ),
          TableRow(
            children: [
              const Padding(
                padding: EdgeInsets.all(8.0),
                child: Text("Distance"),
              ),
              Padding(
                padding: const EdgeInsets.all(8.0).add(valuePadding),
                child: Text(
                  "${km.toStringAsFixed(0)} km",
                  textAlign: TextAlign.right,
                ),
              ),
            ],
          ),
          TableRow(
            children: [
              const Padding(
                padding: EdgeInsets.all(8.0),
                child: Text("Elevation"),
              ),
              Padding(
                padding: const EdgeInsets.all(8.0).add(valuePadding),
                child: Text(
                  "${hm.toStringAsFixed(0)} m",
                  textAlign: TextAlign.right,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
