import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/models/root.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;

class StatisticsWidget extends StatefulWidget {
  const StatisticsWidget({super.key});

  @override
  State<StatisticsWidget> createState() => _StatisticsWidgetState();
}

class _StatisticsWidgetState extends State<StatisticsWidget> {
  DateTime? startTime;

  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      readModel();
    });
  }

  void readModel() {
    RootModel rootModel = Provider.of<RootModel>(context, listen: false);
    bridge.Parameters parameters = rootModel.parameters();
    setState(() {
      startTime = DateTime.parse(parameters.startTime);
    });
  }

  Future<void> _selectTime(BuildContext context) async {
    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay(hour: startTime!.hour, minute: startTime!.minute),
    );

    if (picked != null) {
      DateTime dateTime = DateTime(
        startTime!.year,
        startTime!.month,
        startTime!.day,
        picked.hour,
        picked.minute,
      );
      setState(() {
        startTime = dateTime;
      });
    }
  }

  void openTimeBottomSheet() {
    showModalBottomSheet(
      context: context,
      builder: (BuildContext context) {
        return SizedBox(
          height: 200, // Set your desired height
          child: Center(child: Text('This is a Modal Bottom Sheet')),
        );
      },
    );
  }

  @override
  Widget build(BuildContext ctx) {
    RootModel rootModel = Provider.of<RootModel>(ctx);
    bridge.SegmentStatistics statistics = rootModel.statistics();
    double km = statistics.distanceEnd / 1000;
    double hm = statistics.elevationGain;
    double kmh = rootModel.parameters().speed * 3600 / 1000;
    String startTimeText = "?";
    if (startTime != null) {
      startTimeText = DateFormat('HH:mm').format(startTime!);
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
                child: Text("start time"),
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
              const Padding(padding: EdgeInsets.all(8.0), child: Text("speed")),
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.end,
                  children: [
                    ElevatedButton(
                      onPressed: openTimeBottomSheet,
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
                child: Text("distance"),
              ),
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Padding(
                  padding: valuePadding,
                  child: Text(
                    "${km.toStringAsFixed(0)} km",
                    textAlign: TextAlign.right,
                  ),
                ),
              ),
            ],
          ),
          TableRow(
            children: [
              const Padding(
                padding: EdgeInsets.all(8.0),
                child: Text("elevation"),
              ),
              Padding(
                padding: const EdgeInsets.all(8.0),
                child: Padding(
                  padding: valuePadding,
                  child: Text(
                    "${hm.toStringAsFixed(0)} m",
                    textAlign: TextAlign.right,
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
