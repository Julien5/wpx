import 'dart:developer' as developer;
import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:intl/intl.dart';
import 'package:provider/provider.dart';
import 'package:ui/src/backendmodel.dart';
import 'package:ui/src/routes.dart';
import 'package:ui/src/rust/api/bridge.dart' as bridge;
import 'package:ui/src/statistics_widget.dart';

class Selector extends StatelessWidget {
  final String text;
  final double min;
  final double max;
  final double value;
  final Function(double) onChanged;
  const Selector({
    super.key,
    required this.min,
    required this.max,
    required this.text,
    required this.value,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext ctx) {
    developer.log("[selector/build] text=$text value=$value");
    return Center(
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Text(text),
          Slider(
            min: min,
            max: max,
            divisions: math.max(
              5,
              (((min - max) / 20).floor() * 20),
            ), // not good yet.
            value: value.clamp(min, max),
            label: text,
            onChanged: onChanged,
          ),
        ],
      ),
    );
  }
}

class SegmentsSettings extends StatefulWidget {
  const SegmentsSettings({super.key});

  @override
  State<SegmentsSettings> createState() => _SegmentsSettingsState();
}

class _SegmentsSettingsState extends State<SegmentsSettings> {
  DateTime startTime = DateTime.now();
  double speed = 15 * 1000.0 / 3600;
  double segmentLength = 100000;
  double maxStepSize = 5000;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      readModel();
      setState(() {});
    });
  }

  void readModel() {
    SegmentsProvider provider = Provider.of<SegmentsProvider>(
      context,
      listen: false,
    );
    bridge.Parameters parameters = provider.parameters();
    startTime = DateTime.parse(parameters.startTime);
    speed = parameters.speed;
    segmentLength = parameters.segmentLength;
    maxStepSize = parameters.maxStepSize;
  }

  void writeModel(BuildContext context) {
    SegmentsProvider provider = Provider.of<SegmentsProvider>(
      context,
      listen: false,
    );
    bridge.Parameters oldParameters = provider.parameters();
    String rfc3339time = startTime.toIso8601String();
    if (!rfc3339time.endsWith("Z")) {
      rfc3339time = "${rfc3339time}Z";
    }
    bridge.Parameters newParameters = bridge.Parameters(
      speed: speed,
      startTime: rfc3339time,
      segmentLength: segmentLength,
      maxStepSize: maxStepSize,
      smoothWidth: oldParameters.smoothWidth,
      epsilon: oldParameters.epsilon,
      debug: oldParameters.debug,
    );
    provider.setParameters(newParameters);
    Navigator.of(context).pushNamed(RouteManager.segmentsView);
  }

  Future<void> _selectTime(BuildContext context) async {
    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay(hour: startTime.hour, minute: startTime.minute),
    );

    if (picked != null) {
      DateTime dateTime = DateTime(
        startTime.year,
        startTime.month,
        startTime.day,
        picked.hour,
        picked.minute,
      );
      setState(() {
        startTime = dateTime;
      });
    }
  }

  Future<void> _selectDate(BuildContext context) async {
    final DateTime? picked = await showDatePicker(
      context: context,
      firstDate: DateTime.now(),
      lastDate: DateTime.now().add(Duration(days: 30)),
      initialDate: startTime,
    );

    if (picked != null) {
      DateTime pickedDateTime = DateTime(
        picked.year,
        picked.month,
        picked.day,
        startTime.hour,
        startTime.minute,
      );
      setState(() {
        startTime = pickedDateTime;
      });
    }
  }

  String timeAsString() {
    return DateFormat('HH:mm').format(startTime);
  }

  String dateAsString() {
    return DateFormat('dd.MM.yyyy').format(startTime);
  }

  String speedAsString() {
    double kmh = speed * 3.6;
    return "Speed: ${kmh.toStringAsFixed(1)} kmh";
  }

  String segmentLengthAsString() {
    double km = segmentLength / 1000;
    return "Page length: ${km.toStringAsFixed(1)} km";
  }

  String maxStepSizeAsString() {
    double km = maxStepSize / 1000;
    return "Max step size: ${km.toStringAsFixed(1)} km";
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log(
          "[SegmentsConsumer] length=${segmentsProvider.segments().length}",
        );
        Table table1 = Table(
          columnWidths: const {
            0: FlexColumnWidth(), // Fixed width for the first column
            1: FlexColumnWidth(), // Flexible width for the second column
          },
          children: [
            TableRow(
              children: [
                Container(
                  height: 50,
                  alignment: Alignment.center,
                  child: const Text("Start Date:"),
                ),
                Container(
                  height: 50,
                  alignment: Alignment.center,
                  child: ElevatedButton(
                    onPressed: () => _selectDate(context),
                    child: Text(dateAsString()),
                  ),
                ),
              ],
            ),
            TableRow(
              children: [
                Container(
                  height: 50,
                  alignment: Alignment.center,
                  child: const Text("Start Time:"),
                ),
                Container(
                  height: 50,
                  alignment: Alignment.center,
                  child: ElevatedButton(
                    onPressed: () => _selectTime(context),
                    child: Text(timeAsString()),
                  ),
                ),
              ],
            ),
          ],
        );
        Card card1 = Card(
          elevation: 4, // Add shadow to the card
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8), // Rounded corners
          ),
          child: Padding(
            padding: const EdgeInsets.all(16), // Add padding inside the card
            child: table1,
          ),
        );
        Table table2 = Table(
          columnWidths: const {
            0: FlexColumnWidth(1), // Fixed width for the first column
            1: FlexColumnWidth(2), // Flexible width for the second column
          },
          children: [
            TableRow(
              children: [
                Container(
                  height: 60,
                  alignment: Alignment.centerLeft,
                  child: Text(speedAsString()),
                ),
                Container(
                  height: 60,
                  alignment: Alignment.centerLeft,
                  child: Selector(
                    min: 8.0,
                    max: 30.0,
                    text: "",
                    value: speed * 3.6,
                    onChanged: (value) {
                      setState(() {
                        speed = value * 1000 / 3600;
                      });
                    },
                  ),
                ),
              ],
            ),
            TableRow(
              children: [
                Container(
                  height: 60,
                  alignment: Alignment.centerLeft,
                  child: Text(segmentLengthAsString()),
                ),
                Container(
                  height: 60,
                  alignment: Alignment.centerLeft,
                  child: Selector(
                    min: 50.0,
                    max: 150.0,
                    text: "",
                    value: segmentLength / 1000,
                    onChanged: (value) {
                      setState(() {
                        segmentLength = value * 1000;
                      });
                    },
                  ),
                ),
              ],
            ),
            TableRow(
              children: [
                Container(
                  height: 60,
                  alignment: Alignment.centerLeft,
                  child: Text(maxStepSizeAsString()),
                ),
                Container(
                  height: 60,
                  alignment: Alignment.centerLeft,
                  child: Selector(
                    min: 5.0,
                    max: 30.0,
                    text: "",
                    value: maxStepSize / 1000,
                    onChanged: (value) {
                      setState(() {
                        maxStepSize = value * 1000;
                      });
                    },
                  ),
                ),
              ],
            ),
          ],
        );
        Card card2 = Card(
          elevation: 4, // Add shadow to the card
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8), // Rounded corners
          ),
          child: Padding(
            padding: const EdgeInsets.all(16), // Add padding inside the card
            child: table2,
          ),
        );
        return Column(
          children: [
            card1,
            card2,
            SizedBox(height: 10),
            ElevatedButton(
              onPressed: () => writeModel(context),
              child: const Text("Preview"),
            ),
          ],
        );
      },
    );
  }
}

class SettingsWidget extends StatelessWidget {
  const SettingsWidget({super.key});

  Widget wait() {
    return Scaffold(
      appBar: AppBar(title: const Text('Segments')),
      body: Center(child: Column(children: [Text("loading...")])),
    );
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<RootModel>(
      builder: (context, rootModel, child) {
        if (rootModel.provider() == null) {
          return wait();
        }
        developer.log(
          "[SegmentsProviderWidget] ${rootModel.provider()?.filename()} length=${rootModel.provider()?.segments().length}",
        );

        Card card = Card(
          elevation: 4, // Add shadow to the card
          margin: const EdgeInsets.all(1), // Add margin around the card
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(8), // Rounded corners
          ),
          child: Padding(
            padding: const EdgeInsets.all(16), // Add padding inside the card
            child: StatisticsWidget(),
          ),
        );

        return ChangeNotifierProvider.value(
          value: rootModel.provider(),
          builder: (context, child) {
            return Scaffold(
              appBar: AppBar(title: const Text('Settings')),
              body: Center(
                child: Container(
                  constraints: const BoxConstraints(maxWidth: 500),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    crossAxisAlignment: CrossAxisAlignment.center,
                    children: [card, SizedBox(height: 15), SegmentsSettings()],
                  ),
                ),
              ),
            );
          },
        );
      },
    );
  }
}

class WidthSettings extends StatefulWidget {
  const WidthSettings({super.key});

  @override
  State<WidthSettings> createState() => _WidthSettingsState();
}

class _WidthSettingsState extends State<WidthSettings> {
  double width = 200;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      SegmentsProvider provider = Provider.of<SegmentsProvider>(
        context,
        listen: false,
      );
      bridge.Parameters parameters = provider.parameters();
      setState(() {
        width = parameters.smoothWidth;
      });
    });
  }

  void writeModel(BuildContext context) {
    SegmentsProvider provider = Provider.of<SegmentsProvider>(
      context,
      listen: false,
    );
    bridge.Parameters oldParameters = provider.parameters();
    bridge.Parameters newParameters = bridge.Parameters(
      speed: oldParameters.speed,
      startTime: oldParameters.startTime,
      segmentLength: oldParameters.segmentLength,
      maxStepSize: oldParameters.maxStepSize,
      smoothWidth: width,
      epsilon: oldParameters.epsilon,
      debug: oldParameters.debug,
    );
    provider.setParameters(newParameters);
  }

  String widthAsString() {
    return "$width m";
  }

  void onChanged(double value, SegmentsProvider provider) {
    setState(() {
      width = value;
    });
    writeModel(context);
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log(
          "[_WidthSettingsState] length=${segmentsProvider.segments().length}",
        );
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 500),
            child: Column(
              children: [
                Selector(
                  min: 10.0,
                  max: 1000.0,
                  text: widthAsString(),
                  value: width,
                  onChanged: (value) => onChanged(value, segmentsProvider),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
