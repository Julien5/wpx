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
      setState((){});
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
    String rfc3339time=startTime.toIso8601String();
    if (!rfc3339time.endsWith("Z")) {
      rfc3339time="${rfc3339time}Z";
    }
    bridge.Parameters newParameters = bridge.Parameters(
      speed: speed,
      startTime: rfc3339time,
      segmentLength: segmentLength,
      maxStepSize: maxStepSize,
      smoothWidth: oldParameters.smoothWidth,
      epsilon: oldParameters.epsilon,
    );
    provider.setParameters(newParameters);
    Navigator.of(context).pushNamed(RouteManager.segmentsView);
  }

  Future<void> _selectTime(BuildContext context) async {
    final TimeOfDay? picked = await showTimePicker(
      context: context,
      initialTime: TimeOfDay.now(),
    );

    if (picked != null) {
      DateTime now = DateTime.now();
      DateTime dateTime = DateTime(
        now.year,
        now.month,
        now.day,
        picked.hour,
        picked.minute,
      );
      setState(() {
        startTime = dateTime;
      });
    }
  }

  String timeAsString() {
    return DateFormat('dd.MM HH:mm').format(startTime);
  }

  String speedAsString() {
    double kmh = speed * 3.6;
    return "Speed: ${kmh.toStringAsFixed(1)} kmh";
  }

  String segmentLengthAsString() {
    double km = segmentLength / 1000;
    return "Segment length: ${km.toStringAsFixed(1)} km";
  }

  String maxStepSizeAsString() {
    double km = maxStepSize / 1000;
    return "max step size: ${km.toStringAsFixed(1)} km";
  }

  @override
  Widget build(BuildContext ctx) {
    return Consumer<SegmentsProvider>(
      builder: (context, segmentsProvider, child) {
        developer.log(
          "[SegmentsConsumer] length=${segmentsProvider.segments().length}",
        );
        return Center(
          child: Container(
            constraints: const BoxConstraints(maxWidth: 1500),
            child: Column(
              children: [
                ElevatedButton(
                  onPressed: () => _selectTime(context),
                  child: Text(timeAsString()),
                ),
                Selector(
                  min: 8.0,
                  max: 30.0,
                  text: speedAsString(),
                  value: speed * 3.6,
                  onChanged: (value) {
                    setState(() {
                      speed = value * 1000 / 3600;
                    });
                  },
                ),
                Selector(
                  min: 50.0,
                  max: 150.0,
                  text: segmentLengthAsString(),
                  value: segmentLength / 1000,
                  onChanged: (value) {
                    setState(() {
                      segmentLength = value * 1000;
                    });
                  },
                ),
                Selector(
                  min: 5.0,
                  max: 30.0,
                  text: maxStepSizeAsString(),
                  value: maxStepSize / 1000,
                  onChanged: (value) {
                    setState(() {
                      maxStepSize = value * 1000;
                    });
                  },
                ),
                ElevatedButton(
                  onPressed: () => writeModel(context),
                  child: const Text("Preview"),
                ),
              ],
            ),
          ),
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
        return ChangeNotifierProvider.value(
          value: rootModel.provider(),
          builder: (context, child) {
            return Scaffold(
              appBar: AppBar(title: const Text('Settings')),
              body: Column(children: [StatisticsWidget(),  SegmentsSettings()])
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
            constraints: const BoxConstraints(maxWidth: 300),
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
